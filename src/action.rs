use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::game_event::GameEvent;
use crate::engine::GameEngine;

/// Types of messages that can be sent to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // provide a username
    CreatePlayer(String),
    // provide a username
    LoginPlayer(String),
    LogoutPlayer,
    // engine id, game event, step_index
    EngineEvent(u128, GameEvent, u64),
    // engine id
    RequestEngineReload(u128),
    Ping,
}

/// Types of messages the client can receive from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    PlayerLoggedIn(PlayerState),
    PlayerRemoved(String),
    // engine, server_step_index
    EngineState(GameEngine, u64),
    PlayerState(PlayerState),
    // engine id, game events <step_index, <event_id, event>>
    EngineEvents(u128, BTreeMap<u64, HashMap<u128, GameEvent>>),
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
