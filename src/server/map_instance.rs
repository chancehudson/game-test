use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use game_test::action::{Action, PlayerState};
use game_test::engine::entity::{EEntity, EngineEntity, EntityInput};
use game_test::engine::game_event::GameEvent;
use game_test::engine::{GameEngine, TRAILING_STATE_COUNT};

use game_test::action::Response;
use game_test::map::MapData;
use game_test::STEP_DELAY;

use crate::network;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub map: MapData,
    pub engine: GameEngine,

    // player id keyed to (entity_id, last_sync_step_index, inited)
    pub player_entity: HashMap<String, (u128, u64, bool)>,

    network_server: Arc<network::Server>,
    pending_game_events: Vec<GameEvent>,
}

/// A MapInstance handles communication with the player.
/// Assumes all communication is reliable/tcp
///
/// When a player connects the map state is sent. This includes
/// all present entities and
impl MapInstance {
    pub fn new(map: MapData, network_server: Arc<network::Server>) -> Self {
        Self {
            player_entity: HashMap::new(),
            engine: GameEngine::new(map.clone()),
            network_server,
            map,
            pending_game_events: vec![],
        }
    }

    pub fn send_event_sync(&mut self, player_id: String) -> anyhow::Result<()> {
        if let Some((entity_id, last_sync_step_index, _)) = self.player_entity.get_mut(&player_id) {
            let to_step = self.engine.step_index - STEP_DELAY;
            let events = self
                .engine
                .universal_events_since_step(last_sync_step_index, Some(to_step));
            *last_sync_step_index = to_step;
            if events.is_empty() {
                return Ok(());
            }

            // discard input events for this user (they already know about them)
            let events = events
                .iter()
                .filter_map(|(step_index, events)| {
                    let events = events
                        .iter()
                        .filter(|(_, event)| match event {
                            GameEvent::Input { entity_id: id, .. } => entity_id != id,
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

            let response = Response::EngineEvents(self.engine.id, events);
            let network_server = self.network_server.clone();
            tokio::spawn(async move {
                network_server.send_to_player(&player_id, response).await;
            });
        }
        Ok(())
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(&mut self, player_state: &PlayerState) -> anyhow::Result<()> {
        if let Some((entity_id, _, _)) = self.player_entity.get(&player_state.id) {
            self.engine.remove_entity(*entity_id, true);
        }
        let entity = self
            .engine
            .spawn_player_entity(player_state.id.clone(), None, None);

        self.player_entity
            .insert(player_state.id.clone(), (entity.id(), 0, false));
        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        if let Some((entity_id, _, _)) = self.player_entity.get(player_id) {
            self.engine.remove_entity(*entity_id, true);
        }
        self.player_entity.remove(player_id);
    }

    pub async fn integrate_client_event(
        &mut self,
        player_id: &str,
        engine_id: &u32,
        event: GameEvent,
        step_index: u64,
    ) -> anyhow::Result<()> {
        if self.engine.id != *engine_id {
            anyhow::bail!("engine id mismatch in client event, discarding");
        }
        let step_index = step_index + STEP_DELAY;
        if step_index < self.engine.step_index
            && self.engine.step_index - step_index >= TRAILING_STATE_COUNT
        {
            anyhow::bail!("event too far in the past, discarding");
        }
        if let Some((_entity_id, _last_sync_step_index, _)) = self.player_entity.get(player_id) {
            // TOOO: validity checks of events
            // e.g. moving own character and not someone elses
            println!("integrating event at {step_index}");
            self.engine.integrate_event(step_index, event);
        } else {
            anyhow::bail!("unknown player id, discarding game events");
        }
        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.engine.tick();

        let player_ids = self.player_entity.keys().cloned().collect::<Vec<_>>();
        for id in player_ids {
            if let Some((entity_id, sync_step_index, inited)) = self.player_entity.get_mut(&id) {
                if *inited {
                    self.send_event_sync(id)?;
                } else {
                    let client_step_index = self.engine.step_index - STEP_DELAY;
                    if self
                        .engine
                        .entities_by_step
                        .get(&client_step_index)
                        .unwrap()
                        .contains_key(entity_id)
                    {
                        *inited = true;
                        *sync_step_index = client_step_index;

                        let mut engine = self.engine.engine_at_step(&(client_step_index - 30))?;
                        engine.step_to(&client_step_index);
                        let response =
                            Response::EngineState(engine, self.engine.step_index, Some(*entity_id));

                        let player_id = id.clone();
                        let network_server = self.network_server.clone();
                        // wait for the player to spawn and then send the engine state
                        tokio::spawn(async move {
                            network_server.send_to_player(&player_id, response).await;
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
