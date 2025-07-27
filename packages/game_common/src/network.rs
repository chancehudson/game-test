use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use db::PlayerRecord;

use crate::prelude::*;

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
    PlayerInventorySwap((u8, u8)),
    // slot index, count to drop
    PlayerInventoryDrop(u8, u32),
    Ping,
}

/// Types of messages the client can receive from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    PlayerLoggedIn(PlayerRecord),
    PlayerRemoved(String),
    // engine, entity id the player controls, server step
    EngineState(GameEngine, u128, u64),
    EngineStats(
        u128,
        u64,
        (u64, blake3::Hash),
        Option<BTreeMap<u128, Rc<dyn SEEntity>>>,
    ),
    // engine id, game events <step_index, events>, server step
    RemoteEngineEvents(u128, BTreeMap<u64, Vec<EngineEvent>>, u64),
    PlayerState(PlayerRecord),
    // when a record in the inventory table changes
    // the provided value _replaces_ the old value
    // inventory slot, (item type, count)
    PlayerInventoryRecord(u8, (u64, u32)),
    // from_map
    PlayerExitMap(String),
    LoginError(String),
    Pong,
    Tick,
}
