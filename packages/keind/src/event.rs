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
    /// Request that an entity trigger a copy with a proposed
    /// new version of self.
    RequestCopy {
        mutated_entity: G::Entity, // new version of the entity
        is_non_determinism: bool,
    },
    Input {
        input: G::Input,
        entity_id: u128,
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
            EngineEvent::RequestCopy {
                is_non_determinism, ..
            } => *is_non_determinism,
        }
    }
}
