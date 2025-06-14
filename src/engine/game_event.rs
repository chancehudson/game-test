use serde::Deserialize;
use serde::Serialize;

use crate::engine::entity::EngineEntity;
use crate::engine::entity::EntityInput;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerEvent {
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
pub enum GameEventType {
    RemoveEntity,
    SpawnEntity,
    Input,
}

impl From<&GameEvent> for GameEventType {
    fn from(event: &GameEvent) -> Self {
        match event {
            GameEvent::RemoveEntity { .. } => GameEventType::RemoveEntity,
            GameEvent::SpawnEntity { .. } => GameEventType::SpawnEntity,
            GameEvent::Input { .. } => GameEventType::Input,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameEvent {
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

pub trait HasUniversal {
    fn universal(&self) -> bool;
}

impl HasUniversal for GameEvent {
    fn universal(&self) -> bool {
        match self {
            GameEvent::RemoveEntity { universal, .. } => *universal,
            GameEvent::SpawnEntity { universal, .. } => *universal,
            GameEvent::Input { universal, .. } => *universal,
        }
    }
}

// Add this trait to access the id field uniformly
pub trait HasId {
    fn id(&self) -> u128;
}

impl HasId for GameEvent {
    fn id(&self) -> u128 {
        match self {
            GameEvent::RemoveEntity { id, .. } => *id,
            GameEvent::SpawnEntity { id, .. } => *id,
            GameEvent::Input { id, .. } => *id,
        }
    }
}
