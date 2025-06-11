use std::collections::HashMap;
use std::sync::Arc;

use game_test::action::PlayerState;
use game_test::engine::entity::{EngineEntity, Entity, EntityInput};
use game_test::engine::GameEngine;

use game_test::action::Response;
use game_test::map::MapData;
use game_test::STEP_DELAY;

use crate::network;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub map: MapData,
    pub engine: GameEngine,
    pub player_id_to_entity_id: HashMap<String, u128>,
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
            player_id_to_entity_id: HashMap::new(),
            engine: GameEngine::new(map.clone()),
            network_server,
            map,
        }
    }

    /// When we send the state to the client we always send
    /// a state that is in the past. In the future STEP_DELAY will be
    /// variable depending on latency between individual clients
    /// and the server
    pub fn send_full_state(&self, player_id: String) -> anyhow::Result<()> {
        let target_step = self.engine.step_index - STEP_DELAY;
        let response = Response::EngineState(
            self.engine.engine_at_step(&target_step)?,
            self.engine.step_index,
        );
        let network_server = self.network_server.clone();
        // self.last_sent_step_player_id
        //     .insert(player_id.clone(), target_step);
        tokio::spawn(async move {
            network_server.send_to_player(&player_id, response).await;
        });
        Ok(())
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(&mut self, player_state: &PlayerState) -> anyhow::Result<()> {
        if self.player_id_to_entity_id.contains_key(&player_state.id) {
            anyhow::bail!(
                "attempting to add player {} to map_instance {} twice",
                player_state.username,
                self.map.name
            );
        }
        let entity = self.engine.spawn_player_entity(
            player_state.id.clone(),
            None,
            Some(self.engine.step_index - STEP_DELAY - 1),
        );
        self.player_id_to_entity_id
            .insert(player_state.id.clone(), entity.id());

        {
            let network_server = self.network_server.clone();
            let player_id = player_state.id.clone();
            let response =
                Response::PlayerEntityId(entity.id(), self.engine.clone(), player_state.clone());
            tokio::spawn(async move {
                network_server.send_to_player(&player_id, response).await;
            });
        }
        self.engine.tick();
        self.send_full_state(player_state.id.clone())?;
        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        if let Some(entity_id) = self.player_id_to_entity_id.get(player_id) {
            self.engine.remove_entity(entity_id);
        }
        self.player_id_to_entity_id.remove(player_id);
    }

    pub async fn update_player_input(
        &mut self,
        player_id: &str,
        step_index: u64,
        entity: EngineEntity,
        input: EntityInput,
    ) -> anyhow::Result<()> {
        // the client is behind the server. We take our inputs as
        // happening _now_, which is offset by STEP_DELAY
        // let step_index = self.engine.expected_step_index();
        // use the expected index in case the tick rate is low
        let current_step = self.engine.expected_step_index();
        if step_index > current_step {
            println!(
                "WARNING: client is {} steps ahead of server, discarding input",
                step_index - current_step
            );
            return Ok(());
        }
        if current_step - step_index > 2 * STEP_DELAY {
            println!("WARNING: client input is too far in the past, max {STEP_DELAY} behind, received {} behind", current_step - step_index);
            return Ok(());
        }

        if !matches!(entity, EngineEntity::Player(_)) {
            anyhow::bail!("received incorrect entity type");
        }

        // we have the constant STEP_DELAY, but each player also has a packet RTT that must be
        // less than STEP_DELAY
        //
        // approximation of packet RTT ??
        // let offset_step = current_step - STEP_DELAY - (current_step - step_index);
        if let Some(entity_id) = self.player_id_to_entity_id.get(player_id) {
            if &entity.id() != entity_id {
                anyhow::bail!("received incorrect entity id");
            }
            self.engine
                .register_input(Some(step_index + STEP_DELAY), *entity_id, input);
            // self.engine
            //     .reposition_entity(entity, &(step_index + STEP_DELAY))?;
            // if step_index < current_step {
            //     // replay with the new position
            //     self.engine.reposition_entity(entity, &step_index)?;
            // }
        } else {
            anyhow::bail!("received player position update for player with no entity");
        }
        self.engine.tick();
        self.send_full_state(player_id.to_string())?;
        Ok(())
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.engine.tick();
        for player_id in self.player_id_to_entity_id.keys() {
            self.send_full_state(player_id.clone())?;
        }
        Ok(())
    }
}
