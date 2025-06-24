use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::db::PlayerRecord;
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
    PlayerLoggedIn(PlayerRecord),
    PlayerRemoved(String),
    // engine, entity id the player controls, server step
    EngineState(GameEngine, u128, u64),
    EngineStats(u128, u64, (u64, blake3::Hash)),
    // engine id, game events <step_index, <event_id, event>>, server step
    RemoteEngineEvents(u128, BTreeMap<u64, HashMap<u128, EngineEvent>>, u64),
    PlayerState(PlayerRecord),
    // from_map
    PlayerExitMap(String),
    LoginError(String),
    Pong,
    Tick,
}
