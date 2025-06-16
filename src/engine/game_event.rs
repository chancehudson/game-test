use serde::Deserialize;
use serde::Serialize;

use crate::engine::entity::EngineEntity;
use crate::engine::entity::EntityInput;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerEnterPortal {
        player_id: String,
        entity_id: u128,
        // look at portals in the destination map and select the one farthest
        // to left or right automatically?
        from_map: String,
        to_map: String,
    },
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
    fn universal(&self) -> bool;
}

impl HasUniversal for EngineEvent {
    fn universal(&self) -> bool {
        match self {
            EngineEvent::RemoveEntity { universal, .. } => *universal,
            EngineEvent::SpawnEntity { universal, .. } => *universal,
            EngineEvent::Input { universal, .. } => *universal,
        }
    }
}
