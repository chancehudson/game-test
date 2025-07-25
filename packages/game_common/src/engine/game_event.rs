/// Engine events are events shared between engines to keep
/// the state in sync. Game events are things that occur in the game
/// that need to be displayed to the user or recorded in the db
///
/// engine event = engine changes, game event = state information
///
use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use db::Ability;

use crate::entity::EngineEntity;
use crate::entity::EntityInput;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerEnterPortal {
        player_id: String,
        entity_id: u128,
        // look at portals in the destination map and select the one farthest
        // to left or right automatically?
        from_map: String,
        to_map: String,
        requested_spawn_pos: Option<IVec2>,
    },
    // player entity id, ability
    PlayerAbilityExp(u128, Ability, u64),
    PlayerHealth(String, u64), // player health has changed through damage or healing
    Message(u128, String),     // message sent by an entity (npc or player)
    // player entity id
    PlayerPickUpRequest(u128),
    // player entity id, item type, count
    PlayerPickUp(String, u64, u32),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineEvent {
    RemoveEntity {
        entity_id: u128,
        universal: bool,
    },
    SpawnEntity {
        entity: EngineEntity,
        universal: bool,
    },
    Input {
        input: EntityInput,
        entity_id: u128,
        universal: bool,
    },
    Message {
        text: String,
        entity_id: u128, // sender id
        entity_type_id: u32,
        universal: bool,
    },
}

pub trait HasUniversal {
    fn is_universal(&self) -> bool;
}

impl HasUniversal for EngineEvent {
    fn is_universal(&self) -> bool {
        match self {
            EngineEvent::Message { universal, .. } => *universal,
            EngineEvent::RemoveEntity { universal, .. } => *universal,
            EngineEvent::SpawnEntity { universal, .. } => *universal,
            EngineEvent::Input { universal, .. } => *universal,
        }
    }
}
