use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::entity::EngineEntity;
use crate::engine::entity::EntityInput;
use crate::engine::GameEngine;

/// Types of messages that can be sent to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // provide a username
    CreatePlayer(String),
    // provide a username
    LoginPlayer(String),
    LogoutPlayer,
    // step index, position, input
    PlayerInput(u64, EngineEntity, EntityInput),
    Ping,
}

/// Types of messages the client can receive from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    PlayerLoggedIn(PlayerState),
    PlayerRemoved(String),
    // step index, position, input
    PlayerInput(u64, EngineEntity, EntityInput),
    // send the entity id, the engine, and the position the player will spawn
    PlayerEntityId(u128, GameEngine, Vec2, PlayerState),
    // engine, server_step_index
    EngineState(GameEngine, u64),
    // from_map
    PlayerExitMap(String),
    LoginError(String),
    Pong,
    Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub experience: u64,
    pub max_health: u64,
    pub health: u64,
}
