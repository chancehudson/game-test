use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::GameEngine;
use crate::engine::game_event::EngineEvent;

/// Types of messages that can be sent to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // provide a username
    CreatePlayer(String),
    // provide a username
    LoginPlayer(String),
    LogoutPlayer,
    // engine id, engine event, step_index
    RemoteEngineEvent(u128, EngineEvent, u64),
    // engine id, divergent step index
    RequestEngineReload(u128, u64),
    Ping,
}

/// Types of messages the client can receive from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    PlayerLoggedIn(PlayerState),
    PlayerRemoved(String),
    // engine, entity id the player controls, server step
    EngineState(GameEngine, Option<u128>, u64),
    EngineStats(u128, u64, (u64, blake3::Hash)),
    // engine id, game events <step_index, <event_id, event>>
    RemoteEngineEvents(u128, BTreeMap<u64, HashMap<u128, EngineEvent>>),
    PlayerState(PlayerState),
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
