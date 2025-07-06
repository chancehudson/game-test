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

use super::entity::EngineEntity;
use super::entity::EntityInput;

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
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EngineEventType {
    RemoveEntity,
    SpawnEntity,
    Input,
}

impl From<&EngineEvent> for EngineEventType {
    fn from(event: &EngineEvent) -> Self {
        match event {
            EngineEvent::RemoveEntity { .. } => EngineEventType::RemoveEntity,
            EngineEvent::SpawnEntity { .. } => EngineEventType::SpawnEntity,
            EngineEvent::Input { .. } => EngineEventType::Input,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineEvent {
    RemoveEntity {
        id: u128,
        entity_id: u128,
        universal: bool,
    },
    SpawnEntity {
        id: u128,
        entity: EngineEntity,
        universal: bool,
    },
    Input {
        id: u128,
        input: EntityInput,
        entity_id: u128,
        universal: bool,
    },
}

pub trait HasId {
    fn id(&self) -> u128;
}

impl HasId for EngineEvent {
    fn id(&self) -> u128 {
        match self {
            EngineEvent::RemoveEntity { id, .. } => *id,
            EngineEvent::SpawnEntity { id, .. } => *id,
            EngineEvent::Input { id, .. } => *id,
        }
    }
}

pub trait HasUniversal {
    fn is_universal(&self) -> bool;
}

impl HasUniversal for EngineEvent {
    fn is_universal(&self) -> bool {
        match self {
            EngineEvent::RemoveEntity { universal, .. } => *universal,
            EngineEvent::SpawnEntity { universal, .. } => *universal,
            EngineEvent::Input { universal, .. } => *universal,
        }
    }
}
