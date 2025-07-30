use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent<G: GameLogic> {
    RemoveEntity {
        entity_id: u128,
        is_non_determinism: bool,
    },
    SpawnEntity {
        entity: RefPointer<G::Entity>,
        is_non_determinism: bool,
    },
    Input {
        input: G::Input,
        entity_id: u128,
        is_non_determinism: bool,
    },
    SpawnSystem {
        entity_id: u128,
        system_ptr: RefPointer<G::System>,
        is_non_determinism: bool,
    },
    RemoveSystem {
        entity_id: u128,
        system_ptr: RefPointer<G::System>,
        is_non_determinism: bool,
    },
}

pub trait EventNonDeterminism {
    fn is_non_determinism(&self) -> bool;
}

impl<G: GameLogic> EventNonDeterminism for EngineEvent<G> {
    fn is_non_determinism(&self) -> bool {
        match self {
            EngineEvent::RemoveEntity {
                is_non_determinism, ..
            } => *is_non_determinism,
            EngineEvent::SpawnEntity {
                is_non_determinism, ..
            } => *is_non_determinism,
            EngineEvent::Input {
                is_non_determinism, ..
            } => *is_non_determinism,
            EngineEvent::SpawnSystem {
                is_non_determinism, ..
            } => *is_non_determinism,
            EngineEvent::RemoveSystem {
                is_non_determinism, ..
            } => *is_non_determinism,
        }
    }
}
