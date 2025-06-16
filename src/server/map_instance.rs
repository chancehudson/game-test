use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use bevy_math::IVec2;
use game_test::action::PlayerState;
use game_test::engine::STEPS_PER_SECOND;
use game_test::engine::entity::EEntity;
use game_test::engine::entity::EngineEntity;
use game_test::engine::entity::player::PlayerEntity;
use game_test::engine::game_event::EngineEvent;
use game_test::engine::game_event::HasId;
use game_test::engine::{GameEngine, TRAILING_STATE_COUNT};

use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;

use crate::game::RemoteEngineEvent;
use crate::network;

pub struct RemotePlayerEngine {
    pub entity_id: Option<u128>,
    pub engine_id: u128,
    pub is_inited: bool,
    pub player_id: String,
    pub last_input_step_index: u64,
}

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub map: MapData,
    pub engine: GameEngine,

    // actions received from players. These must be sanitized before
    // ingesting to engine
    pub pending_actions: (
        flume::Sender<RemoteEngineEvent>,
        flume::Receiver<RemoteEngineEvent>,
    ),
    pub pending_events: (
        flume::Sender<(u64, EngineEvent)>,
        flume::Receiver<(u64, EngineEvent)>,
    ),
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
            pending_actions: flume::unbounded(),
            pending_events: flume::unbounded(),
            player_engines: HashMap::new(),
            engine: GameEngine::new(map.clone()),
            network_server,
            map,
            last_stats_broadcast: 0.,
        }
    }

    pub async fn init_player(
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

        let response = Response::EngineState(client_engine, player.entity_id, engine.step_index);
        let player_id = player_id.to_string();
        network_server.send_to_player(&player_id, response).await;
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(
        &mut self,
        player_state: &PlayerState,
        requested_spawn_pos: Option<IVec2>,
    ) -> anyhow::Result<()> {
        let entity = EngineEntity::Player(PlayerEntity::new_with_ids(
            rand::random(),
            player_state.id.clone(),
        ));
        let player = RemotePlayerEngine {
            engine_id: rand::random(),
            is_inited: false,
            player_id: player_state.id.clone(),
            entity_id: Some(entity.id()),
            last_input_step_index: self.engine.step_index,
        };
        // we've inserted a new player, last_engine is the old player engine data, if it exists
        if let Some(last_engine) = self.player_engines.insert(player_state.id.clone(), player) {
            // cleanup previous engine connection
            if let Some(entity_id) = last_engine.entity_id {
                self.engine.remove_entity(entity_id, true);
                let remove = EngineEvent::RemoveEntity {
                    id: rand::random(),
                    entity_id,
                    universal: true,
                };
                self.engine.register_event(None, remove.clone());
                self.pending_events
                    .0
                    .send((self.engine.step_index, remove))?;
            }
        }
        let add_event = EngineEvent::SpawnEntity {
            id: rand::random(),
            entity,
            universal: true,
        };
        self.engine.register_event(None, add_event.clone());
        self.pending_events
            .0
            .send((self.engine.step_index, add_event))?;
        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: &str) -> anyhow::Result<()> {
        if let Some(player) = self.player_engines.remove(player_id) {
            if let Some(entity_id) = player.entity_id {
                let event = EngineEvent::RemoveEntity {
                    id: rand::random(),
                    entity_id,
                    universal: true,
                };
                self.engine.register_event(None, event.clone());
                self.pending_events
                    .0
                    .send((self.engine.step_index, event))?;
            }
        } else {
            println!(
                "WARNING: attempted to remove {player_id} from {} instance",
                self.map.name
            );
        }
        Ok(())
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

    pub async fn process_remote_event(
        &mut self,
        RemoteEngineEvent {
            player_id,
            engine_id,
            event,
            step_index,
        }: &RemoteEngineEvent,
    ) -> anyhow::Result<Option<(u64, EngineEvent)>> {
        // discard events too far back
        if step_index < &self.engine.step_index
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
                EngineEvent::SpawnEntity {
                    universal: _, // player should not set this
                    entity: _,
                    id: _, // we'll generate the id from our engine seeded rng
                } => {}
                EngineEvent::Input {
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
                        player.last_input_step_index = *step_index;
                        return Ok(Some((*step_index, event.clone())));
                    } else {
                        println!("WARNING: attempting to send input with no spawned entity");
                    }
                }
                EngineEvent::RemoveEntity {
                    id: _,
                    entity_id: _,
                    universal: _,
                } => {}
            }
        } else {
            anyhow::bail!("unknown player id, discarding game events");
        }
        Ok(None)
    }

    /// ingest an event from a point in time in a another engine within 60 steps of this engine
    pub async fn process_remote_events(
        &mut self,
        remote_events: Vec<RemoteEngineEvent>,
    ) -> anyhow::Result<BTreeMap<u64, HashMap<u128, EngineEvent>>> {
        let mut events: BTreeMap<u64, HashMap<_, _>> = BTreeMap::new();

        for remote_event in remote_events {
            if let Some((step_index, engine_event)) =
                self.process_remote_event(&remote_event).await?
            {
                if let Some(_) = events
                    .entry(step_index)
                    .or_default()
                    .insert(engine_event.id(), engine_event.clone())
                {
                    println!("WARNING: duplicate action/event detected!");
                }
            } else {
                println!("Error processing remote engine event: {:?}", remote_event);
            }
        }
        Ok(events)
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        // integrate any events we've received since last tick
        let pending_actions = self.pending_actions.1.drain().collect::<Vec<_>>();
        let pending_events = self.pending_events.1.drain().collect::<Vec<_>>();
        let has_events = !pending_events.is_empty() || !pending_actions.is_empty();
        let mut new_events = if pending_actions.len() > 0 {
            self.process_remote_events(pending_actions).await?
        } else {
            BTreeMap::new()
        };
        for (si, event) in pending_events {
            if let Some(_) = new_events.entry(si).or_default().insert(event.id(), event) {
                println!("WARNING: overwriting existing event");
            }
        }
        if has_events {
            self.engine.integrate_events(new_events.clone());
        }

        // step as needed
        self.engine.tick();

        // build a checksum for a step in the recent past to
        // send to the client for detecting desync
        let engine_hash = if timestamp() - self.last_stats_broadcast > 2.0
            && self.engine.step_index >= 2 * STEPS_PER_SECOND
        {
            self.last_stats_broadcast = timestamp();
            let target_step = self.engine.step_index - 2 * STEPS_PER_SECOND;
            Some((target_step, self.engine.step_hash(&target_step)?))
        } else {
            None
        };
        for (id, player) in self.player_engines.iter_mut() {
            // send engine stats
            if let Some(engine_hash) = engine_hash {
                let id = id.to_string();
                let engine_hash = engine_hash.clone();
                self.network_server
                    .send_to_player(
                        &id,
                        Response::EngineStats(
                            player.engine_id,
                            self.engine.step_index,
                            engine_hash,
                        ),
                    )
                    .await;
            }
            if player.is_inited {
                if has_events {
                    let response = Response::RemoteEngineEvents(
                        player.engine_id,
                        new_events.clone(),
                        self.engine.step_index,
                    );
                    self.network_server.send_to_player(id, response).await;
                }
            } else {
                Self::init_player(self.network_server.clone(), &self.engine, &id, player).await;
            }
        }
        Ok(())
    }
}
