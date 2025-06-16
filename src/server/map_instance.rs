use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use game_test::action::PlayerState;
use game_test::engine::STEPS_PER_SECOND;
use game_test::engine::entity::EngineEntity;
use game_test::engine::game_event::GameEvent;
use game_test::engine::game_event::HasId;
use game_test::engine::{GameEngine, TRAILING_STATE_COUNT};

use game_test::STEP_DELAY;
use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;

use crate::network;

pub struct RemotePlayerEngine {
    pub entity_id: Option<u128>,
    pub engine_id: u128,
    pub is_inited: bool,
    pub player_id: String,
    pub last_input_step_index: u64,
    pub pending_events: BTreeMap<u64, HashMap<u128, GameEvent>>,
}

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub map: MapData,
    pub engine: GameEngine,

    pub player_engines: HashMap<String, RemotePlayerEngine>,
    last_stats_broadcast: f64,

    network_server: Arc<network::Server>,
}

/// A MapInstance handles communication with the player.
/// Assumes all communication is reliable/tcp
///
/// When a player connects the map state is sent. This includes
/// all present entities and
impl MapInstance {
    pub fn new(map: MapData, network_server: Arc<network::Server>) -> Self {
        Self {
            player_engines: HashMap::new(),
            engine: GameEngine::new(map.clone()),
            network_server,
            map,
            last_stats_broadcast: 0.,
        }
    }

    pub fn init_player(
        network_server: Arc<network::Server>,
        engine: &GameEngine,
        player_id: &str,
        player: &mut RemotePlayerEngine,
    ) {
        // reverse the engine by 30 frames, insert the player, and step 30 frames forward
        // to allow 30 frames of replay
        const ENGINE_HISTORY_STEPS: u64 = 30;
        let mut client_engine = if let Ok(engine) =
            engine.engine_at_step(&(engine.step_index - ENGINE_HISTORY_STEPS))
        {
            engine
        } else {
            // engine needs to warm up, try init on next tick
            return;
        };
        client_engine.id = rand::random();
        client_engine.step_to(&engine.step_index);

        player.is_inited = true;
        player.engine_id = client_engine.id;

        let response = Response::EngineState(client_engine);
        let player_id = player_id.to_string();
        tokio::spawn(async move {
            network_server.send_to_player(&player_id, response).await;
        });
    }

    /// TODO: move synchronization management into a structure ?
    pub fn send_event_sync(
        network_server: Arc<network::Server>,
        engine: &GameEngine,
        player_id: &str,
        player: &mut RemotePlayerEngine,
    ) {
        // if the initial game state has not been sent we don't want to send updates
        if !player.is_inited {
            return;
        }
        // remove empty entries from pending events
        let start_count = player.pending_events.len();
        player
            .pending_events
            .retain(|_si, pending_hashmap| !pending_hashmap.is_empty());
        if start_count != player.pending_events.len() {
            println!("WARNING: removed empty RemotePlayerEngine pending_events btreemap entries");
        }
        // check if the structure has no entries
        if player.pending_events.is_empty() {
            return;
        }

        // discard input events for this user (they already know about them)
        // TODO: generalized event filtering ???
        let pending_events = std::mem::take(&mut player.pending_events);
        let response = Response::EngineEvents(player.engine_id, pending_events);
        let player_id = player_id.to_string();
        tokio::spawn(async move {
            network_server.send_to_player(&player_id, response).await;
        });
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(&mut self, player_state: &PlayerState) -> anyhow::Result<()> {
        // we've inserted a new player, last_engine is the old player engine data, if it exists
        if let Some(last_engine) = self.player_engines.insert(
            player_state.id.clone(),
            RemotePlayerEngine {
                engine_id: rand::random(),
                is_inited: false,
                player_id: player_state.id.clone(),
                entity_id: None,
                last_input_step_index: self.engine.step_index,
                pending_events: BTreeMap::default(),
            },
        ) {
            // cleanup previous engine connection
            if let Some(entity_id) = last_engine.entity_id {
                self.engine.remove_entity(entity_id, true);
            }
        }
        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        if let Some(player) = self.player_engines.remove(player_id) {
            if let Some(entity_id) = player.entity_id {
                // cleanup engine connection
                self.engine.remove_entity(entity_id, true);
            }
        } else {
            println!(
                "WARNING: attempted to remove {player_id} from {} instance",
                self.map.name
            );
        }
    }

    /// Reload fully reload the players engine instance without respawning them
    pub async fn reload_player_engine(&mut self, player_id: &str) -> anyhow::Result<()> {
        if let Some(player) = self.player_engines.get_mut(player_id) {
            println!("player {player_id} requested engine reload");
            // engine resync
            player.is_inited = false;
            player.engine_id = rand::random();
        } else {
            println!("WARNING: attempting engine reload for player not on instance");
        }
        Ok(())
    }

    /// ingest an event from a point in time in a another engine within 60 steps of this engine
    pub async fn integrate_client_event(
        &mut self,
        player_id: &str,
        engine_id: &u128,
        event: GameEvent,
        step_index: u64,
    ) -> anyhow::Result<()> {
        // discard events too far back
        if step_index < self.engine.step_index
            && self.engine.step_index - step_index >= TRAILING_STATE_COUNT
        {
            anyhow::bail!("event too far in the past, discarding");
        }

        // player action validity checks/logic
        let mut new_events = vec![];
        if let Some(player) = self.player_engines.get_mut(player_id) {
            if &player.engine_id != engine_id {
                anyhow::bail!("engine id mismatch in client event, discarding");
            }
            // check that we're syncing with the correct engine
            if &player.engine_id != engine_id {
                anyhow::bail!("event from incorrect engine_id for player");
            }
            // Structure for validity checks
            match &event {
                GameEvent::SpawnEntity {
                    universal: _, // player should not set this
                    entity,
                    id: _, // we'll generate the id from our engine seeded rng
                } => {
                    if let Some(entity_id) = player.entity_id {
                        anyhow::bail!("player attempted to spawn second self");
                    }
                    match entity {
                        EngineEntity::Player(p) => {
                            if p.player_id != player_id {
                                anyhow::bail!(
                                    "attempted to spawn player entity with incorrect player id"
                                );
                            }
                            if self.engine.entities.contains_key(&p.id) {
                                anyhow::bail!("attempted to spawn player with duplicate entity id");
                            }
                            player.entity_id = Some(p.id);
                            self.engine
                                .spawn_entity(EngineEntity::Player(p.clone()), None, true);
                            new_events.push(event.clone());
                        }
                        _ => {}
                    }
                }
                GameEvent::Input {
                    universal: _,
                    input: _,
                    id: _,
                    entity_id,
                } => {
                    if let Some(id) = player.entity_id {
                        if entity_id != &id {
                            anyhow::bail!("player tried to input for wrong entity");
                        }
                        println!("integrating input event at {step_index}");
                        player.last_input_step_index = step_index;
                        new_events.push(event.clone());
                        self.engine.integrate_event(step_index, event);
                    } else {
                        println!("attempting to send input with no spawned entity");
                    }
                }
                GameEvent::RemoveEntity {
                    id: _,
                    entity_id,
                    universal: _,
                } => {
                    if let Some(id) = player.entity_id {
                        if entity_id != &id {
                            anyhow::bail!("player tried to remove non-self entity");
                        }
                        player.entity_id = None;
                        new_events.push(event.clone());
                        self.engine.integrate_event(step_index, event);
                    } else {
                        println!("attempting to send removal with no spawned entity");
                    }
                }
            }
        } else {
            anyhow::bail!("unknown player id, discarding game events");
        }
        for event in new_events {
            for (_, player) in self.player_engines.iter_mut() {
                player
                    .pending_events
                    .entry(self.engine.step_index)
                    .or_default()
                    .insert(event.id(), event.clone());
            }
        }
        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.engine.tick();

        // let player_ids = self.player_engines.keys().cloned().collect::<Vec<_>>();
        // TODO: collect errors and finish syncing
        let engine_hash = if timestamp() - self.last_stats_broadcast > 2.0 {
            self.last_stats_broadcast = timestamp();
            let target_step = self.engine.step_index - 2 * STEPS_PER_SECOND;
            Some((target_step, self.engine.step_hash(&target_step)?))
        } else {
            None
        };
        for (id, player) in self.player_engines.iter_mut() {
            // send engine stats
            if let Some(engine_hash) = engine_hash {
                let network_server = self.network_server.clone();
                let step_index = self.engine.step_index;
                let id = id.to_string();
                let engine_hash = engine_hash.clone();
                tokio::spawn(async move {
                    network_server
                        .send_to_player(&id, Response::EngineStats(step_index, engine_hash))
                        .await;
                });
            }
            if player.is_inited {
                Self::send_event_sync(self.network_server.clone(), &self.engine, id, player);
            } else {
                Self::init_player(self.network_server.clone(), &self.engine, &id, player);
            }
        }
        Ok(())
    }
}
