use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use bevy_math::IVec2;

use db::PlayerInventory;
use db::PlayerRecord;
use db::PlayerStats;
use game_common::prelude::*;
use keind::prelude::*;

use crate::game::RemoteEngineEvent;
use crate::network;

pub struct RemotePlayerEngine {
    pub socket_id: String,
    pub entity_id: u128,
    pub engine_id: u128,
    pub is_inited: bool,
    pub player_id: String,
    pub last_input_step_index: u64,
}

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub engine: GameEngine<KeindGameLogic>,
    pub map: MapData,

    // actions received from players. These must be sanitized before
    // ingesting to engine
    pub pending_actions: (
        flume::Sender<RemoteEngineEvent>,
        flume::Receiver<RemoteEngineEvent>,
    ),
    pub pending_events: (
        flume::Sender<(u64, EngineEvent<KeindGameLogic>)>,
        flume::Receiver<(u64, EngineEvent<KeindGameLogic>)>,
    ),
    pub player_engines: HashMap<String, RemotePlayerEngine>,
    last_stats_broadcast: f64,

    network_server: Arc<network::Server>,
    db: Arc<redb::Database>,
    pub game_events: flume::Sender<GameEvent>,
    latest_processed_game_events: u64,
}

/// A MapInstance handles communication with the player.
/// Assumes all communication is reliable/tcp
///
/// When a player connects the map state is sent. This includes
/// all present entities and
impl MapInstance {
    pub fn new(
        map: MapData,
        network_server: Arc<network::Server>,
        db: Arc<redb::Database>,
        game_events: flume::Sender<GameEvent>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            pending_actions: flume::unbounded(),
            pending_events: flume::unbounded(),
            player_engines: HashMap::new(),
            engine: GameEngine::<KeindGameLogic>::new(map.size, rand::random()),
            map,
            network_server,
            last_stats_broadcast: 0.,
            db,
            game_events,
            latest_processed_game_events: 0,
        })
    }

    pub async fn spawn_item(&mut self, player_id: &str, item: (u64, u32)) -> anyhow::Result<()> {
        if let Some(player_engine) = self.player_engines.get(player_id) {
            if let Some(entity) = self
                .engine
                .entity_by_id_untyped(&player_engine.entity_id, None)
                .cloned()
            {
                let event = EngineEvent::SpawnEntity {
                    entity: RefPointer::new(
                        ItemEntity::new_item(
                            rand::random(),
                            entity.center(),
                            item.0,
                            item.1,
                            entity.id(),
                            self.engine.step_index,
                        )
                        .into(),
                    ),
                    is_non_determinism: true,
                };
                self.pending_events
                    .0
                    .send((self.engine.step_index, event.clone()))?;
                self.engine.register_event(None, event);
            }
        }
        Ok(())
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(
        &mut self,
        socket_id: String,
        player_record: &PlayerRecord,
        player_stats: &PlayerStats,
        requested_spawn_pos: Option<IVec2>,
    ) -> anyhow::Result<()> {
        let entity =
            PlayerEntity::new_with_ids(rand::random(), player_record.clone(), player_stats.clone());
        let player = RemotePlayerEngine {
            socket_id,
            engine_id: rand::random(),
            is_inited: false,
            player_id: player_record.id.clone(),
            entity_id: entity.id(),
            last_input_step_index: self.engine.step_index,
        };
        // we've inserted a new player, last_engine is the old player engine data, if it exists
        if let Some(last_engine) = self.player_engines.insert(player_record.id.clone(), player) {
            // cleanup previous engine connection
            let remove = EngineEvent::RemoveEntity {
                entity_id: last_engine.entity_id,
                is_non_determinism: true,
            };
            self.engine.register_event(None, remove.clone());
            self.pending_events
                .0
                .send((self.engine.step_index, remove))?;
        }
        let add_event = EngineEvent::SpawnEntity {
            entity: RefPointer::new(entity.into()),
            is_non_determinism: true,
        };
        self.engine.register_event(None, add_event.clone());
        self.pending_events
            .0
            .send((self.engine.step_index, add_event))?;
        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: &str) -> anyhow::Result<()> {
        if let Some(player) = self.player_engines.remove(player_id) {
            let event = EngineEvent::RemoveEntity {
                entity_id: player.entity_id,
                is_non_determinism: true,
            };
            self.engine.register_event(None, event.clone());
            self.pending_events
                .0
                .send((self.engine.step_index, event))?;
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
    ) -> anyhow::Result<Option<(u64, EngineEvent<KeindGameLogic>)>> {
        // discard events too far back
        if step_index < &self.engine.step_index
            && self.engine.step_index - step_index >= self.engine.trailing_state_len
        {
            anyhow::bail!("event too far in the past, discarding");
        }
        if step_index > &self.engine.expected_step_index() {
            anyhow::bail!("event too far in the future, discarding");
        }

        // player action validity checks/logic
        if let Some(player) = self.player_engines.get_mut(player_id) {
            // check that we're syncing with the correct engine
            if &player.engine_id != engine_id {
                // we discard without erroring
                return Ok(None);
            }
            // Structure for validity checks
            match event {
                EngineEvent::Input { entity_id, .. } => {
                    if entity_id != &player.entity_id {
                        anyhow::bail!("player tried to input for wrong entity");
                    }
                    println!("integrating input event at {step_index}");
                    player.last_input_step_index = *step_index;
                    return Ok(Some((*step_index, event.clone())));
                }
                _ => {} // disallow all others
            }
        } else {
            anyhow::bail!("unknown player id, discarding game event");
        }
        Ok(None)
    }

    /// ingest an event from a point in time in a another engine within 60 steps of this engine
    pub async fn process_remote_events(
        &mut self,
        remote_events: Vec<RemoteEngineEvent>,
    ) -> anyhow::Result<BTreeMap<u64, Vec<EngineEvent<KeindGameLogic>>>> {
        let mut events: BTreeMap<u64, Vec<_>> = BTreeMap::new();

        for remote_event in remote_events {
            if let Some((step_index, engine_event)) =
                self.process_remote_event(&remote_event).await?
            {
                events.entry(step_index).or_default().push(engine_event);
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
            new_events.entry(si).or_default().push(event);
        }
        if has_events {
            self.engine.integrate_events(new_events.clone());
        }

        // step as needed
        self.engine.tick();

        // process game events at a delayed rate to allow lagged user inputs
        let latest_step = self.engine.step_index - STEP_DELAY.min(self.engine.step_index);
        let game_events = self
            .engine
            .game_events(self.latest_processed_game_events, latest_step);
        self.latest_processed_game_events = latest_step;
        for game_event in game_events {
            // handle game events that occurred during a step
            match &*game_event {
                GameEvent::Message(_, _) => {}
                GameEvent::PlayerPickUpRequest(_) => {}
                GameEvent::PlayerPickUp(player_id, item_type, count) => {
                    let mut inventory = PlayerInventory::new(player_id.to_string());
                    match inventory.player_picked_up(self.db.clone(), *item_type, *count)? {
                        Some((slot_index, new_record)) => {
                            self.network_server
                                .send_to_player(
                                    &player_id,
                                    Response::PlayerInventoryRecord(slot_index, new_record),
                                )
                                .await;
                        }
                        None => {
                            println!("WARNING: player inventory is full. TODO: notify player");
                        }
                    }
                }
                GameEvent::PlayerEnterPortal {
                    player_id: _,
                    entity_id: _,
                    from_map: _,
                    to_map: _,
                    requested_spawn_pos: _,
                } => {
                    // we'll send this up to game.rs
                    self.game_events.send((*game_event).clone()).unwrap();
                }
                GameEvent::PlayerAbilityExp(player_entity_id, ability, amount) => {}
                GameEvent::PlayerHealth(player_id, new_health) => {
                    PlayerRecord::set_health(&self.db, &player_id, *new_health)?;
                }
            }
        }

        // build a checksum for a step in the recent past to
        // send to the client for detecting desync
        let engine_hash = if timestamp() - self.last_stats_broadcast > 2.0
            && self.engine.step_index >= 2 * STEPS_PER_SECOND as u64
        {
            self.last_stats_broadcast = timestamp();
            let target_step = self.engine.step_index - 2 * STEPS_PER_SECOND as u64;
            Some((target_step, self.engine.step_hash(&target_step)?))
        } else {
            None
        };
        let mut ids_to_remove = vec![];
        for (id, player) in self.player_engines.iter_mut() {
            let player_disconnected = if let Some(socket_id) = self
                .network_server
                .socket_by_player_id(&player.player_id)
                .await
            {
                socket_id != player.socket_id
            } else {
                true
            };
            if player_disconnected {
                let removal_event = EngineEvent::RemoveEntity {
                    entity_id: player.entity_id,
                    is_non_determinism: true,
                };
                self.engine.register_event(None, removal_event.clone());
                self.pending_events
                    .0
                    .send((self.engine.step_index, removal_event))?;
                ids_to_remove.push(id.clone());
                continue;
            }
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
                            #[cfg(debug_assertions)]
                            Some(self.engine.entities_at_step(engine_hash.0).clone()),
                            #[cfg(not(debug_assertions))]
                            None,
                        ),
                    )
                    .await;
            }
            if player.is_inited {
                if has_events {
                    let response = Response::RemoteEngineEvents(
                        player.engine_id,
                        new_events
                            .iter()
                            .map(|(step_index, events)| {
                                (
                                    *step_index,
                                    events
                                        .iter()
                                        .filter(|event| match event {
                                            EngineEvent::Input { entity_id, .. } => {
                                                entity_id != &player.entity_id
                                            }
                                            _ => true,
                                        })
                                        .cloned()
                                        .collect::<Vec<_>>(),
                                )
                            })
                            .filter(|(_step_index, events)| !events.is_empty())
                            .collect::<BTreeMap<_, Vec<_>>>(),
                        self.engine.expected_step_index(),
                    );
                    self.network_server.send_to_player(id, response).await;
                }
            } else {
                Self::init_remote_engine(self.network_server.clone(), &self.engine, &id, player)
                    .await;
            }
        }
        for id in ids_to_remove {
            self.player_engines.remove(&id);
        }
        // cleanup players that have disconnected
        let mut removal_events = vec![];
        for entity in self.engine.entities_by_type::<PlayerEntity>() {
            if let Some(player) = self.player_engines.get(&entity.player_id) {
                if player.entity_id == entity.id() {
                    // player still connected/active
                    continue;
                }
            }
            // otherwise remove the entity
            removal_events.push((
                entity.player_id.clone(),
                EngineEvent::RemoveEntity {
                    entity_id: entity.id(),
                    is_non_determinism: true,
                },
            ));
        }
        for (player_id, e) in removal_events {
            self.player_engines.remove(&player_id);
            self.pending_events
                .0
                .send((self.engine.step_index, e.clone()))?;
            self.engine.register_event(None, e);
        }

        Ok(())
    }

    pub async fn init_remote_engine(
        network_server: Arc<network::Server>,
        engine: &GameEngine<KeindGameLogic>,
        player_id: &str,
        player: &mut RemotePlayerEngine,
    ) {
        if engine.step_index < STEP_DELAY {
            return;
        }
        let client_engine = engine.engine_at_step(&(engine.step_index - STEP_DELAY), false);
        if client_engine.is_err() {
            // engine warming up, we'll try again next tick
            return;
        }
        let mut client_engine = client_engine.unwrap();
        client_engine.id = rand::random();

        player.is_inited = true;
        player.engine_id = *client_engine.id();

        let response = Response::EngineState(
            client_engine,
            player.entity_id,
            engine.expected_step_index(),
        );
        let player_id = player_id.to_string();
        network_server.send_to_player(&player_id, response).await;
    }
}
