use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use game_test::action::PlayerState;
use game_test::engine::entity::EEntity;
use game_test::engine::entity::EngineEntity;
use game_test::engine::game_event::GameEvent;
use game_test::engine::player::PlayerEntity;
use game_test::engine::{GameEngine, TRAILING_STATE_COUNT};

use game_test::action::Response;
use game_test::map::MapData;
use game_test::STEP_DELAY;

use crate::network;

struct RemotePlayerEngine {
    pub entity_id: Option<u128>,
    pub engine_id: u128,
    pub is_inited: bool,
    pub player_id: String,
    pub last_step_index: u64,
}

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub map: MapData,
    pub engine: GameEngine,

    player_engines: HashMap<String, RemotePlayerEngine>,

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
        }
    }

    /// TODO: move synchronization management into a structure ?
    pub fn send_event_sync(&mut self, player_id: String) -> anyhow::Result<()> {
        if let Some(player) = self.player_engines.get_mut(&player_id) {
            // if the initial game state has not been sent we don't want to send updates
            if !player.is_inited {
                return Ok(());
            }
            let to_step = self.engine.step_index - STEP_DELAY;
            let events = self
                .engine
                .universal_events_since_step(&player.last_step_index, Some(to_step));
            player.last_step_index = to_step;
            if events.is_empty() {
                return Ok(());
            }

            // discard input events for this user (they already know about them)
            // TODO: generalized event filtering ???
            let events = events
                .iter()
                .filter_map(|(step_index, events)| {
                    let events = events
                        .iter()
                        .filter(|(_, event)| match event {
                            GameEvent::Input { entity_id: id, .. } => {
                                // don't send input events back to the original engine
                                if let Some(player_entity_id) = player.entity_id {
                                    return &player_entity_id != id;
                                }
                                true
                            }
                            _ => true,
                        })
                        .map(|(id, event)| (*id, event.clone()))
                        .collect::<HashMap<_, _>>();
                    if events.is_empty() {
                        None
                    } else {
                        Some((*step_index, events))
                    }
                })
                .collect::<BTreeMap<_, _>>();
            if events.is_empty() {
                return Ok(());
            }

            let response = Response::EngineEvents(player.engine_id, events);
            let network_server = self.network_server.clone();
            tokio::spawn(async move {
                network_server.send_to_player(&player_id, response).await;
            });
        }
        Ok(())
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
                last_step_index: 0,
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
        let step_index = step_index + STEP_DELAY;
        if step_index < self.engine.step_index
            && self.engine.step_index - step_index >= TRAILING_STATE_COUNT
        {
            anyhow::bail!("event too far in the past, discarding");
        }

        // player action validity checks/logic
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
                        self.engine.say_to(
                            &entity_id,
                            format!("stop trying to spawn entities, {}", player.player_id),
                        );
                        anyhow::bail!("player attempted to spawn second self");
                    }
                    match entity {
                        EngineEntity::Player(p) => {
                            if p.player_id != player_id {
                                anyhow::bail!(
                                    "attempted to spawn player entity with incorrect player id"
                                );
                            }
                            player.entity_id = Some(p.id);
                            let mut entity = p.clone();
                            entity.is_active = false;
                            self.engine
                                .spawn_entity(EngineEntity::Player(p.clone()), None, true);
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
                            self.engine.say_to(&id, format!("hello you tried to move a character that is not you. probably don't try that again"));
                            anyhow::bail!("player tried to input for wrong entity");
                        }
                        println!("integrating input event at {step_index}");
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
                            self.engine.say_to(&id, format!("hello you tried to remove a character that is not you. probably don't try that again"));
                            anyhow::bail!("player tried to remove non-self entity");
                        }
                        player.entity_id = None;
                        self.engine.integrate_event(step_index, event);
                    } else {
                        println!("attempting to send removal with no spawned entity");
                    }
                }
            }
        } else {
            anyhow::bail!("unknown player id, discarding game events");
        }
        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.engine.tick();

        let player_ids = self.player_engines.keys().cloned().collect::<Vec<_>>();
        for id in player_ids {
            if let Some(player) = self.player_engines.get_mut(&id) {
                if player.is_inited {
                    self.send_event_sync(id)?;
                } else {
                    // the state confirmation delay. Participants must come to consensus in STEP_DELAY
                    // steps
                    let client_step_index = self.engine.step_index - STEP_DELAY;

                    // reverse the engine by 30 frames, insert the player, and step 30 frames forward
                    // to allow 30 frames of replay
                    const ENGINE_HISTORY_STEPS: u64 = 30;
                    let mut engine = self
                        .engine
                        .engine_at_step(&(client_step_index - ENGINE_HISTORY_STEPS))?;
                    engine.id = rand::random();
                    engine.step_to(&client_step_index);

                    player.last_step_index = client_step_index;
                    player.is_inited = true;
                    player.engine_id = engine.id;

                    let response = Response::EngineState(engine, self.engine.step_index);

                    let player_id = id.clone();
                    let network_server = self.network_server.clone();
                    tokio::spawn(async move {
                        network_server.send_to_player(&player_id, response).await;
                    });
                }
            }
        }
        Ok(())
    }
}
