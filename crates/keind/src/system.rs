use std::any::Any;

use crate::prelude::*;

/// A system that is attached by pointer to an entity in the engine.
/// Systems determine when they "step", which involves copying their
/// state and returning a new mutated instance.
///
/// This allows entities to be constant size in memory (vector of pointers)
/// with granular copy on change behavior.
pub trait EEntitySystem<G: GameLogic>: Any {
    /// Readonly access to entity. Determine if write access
    /// is needed.
    /// Return true to mutate self or entity
    /// Engine events may be sent here.
    fn prestep(&self, _engine: &GameEngine<G>, _entity: &G::Entity) -> bool {
        true
    }

    /// Step the system, provided the engine, and the entity the system
    /// is attached to. Underlying entity type may be extracted with, for example,
    /// `let player_entity = entity.extract_ref_mut::<PlayerEntity>()`
    ///
    /// the system may freely mutate the entity. The entity step logic executes _after_
    /// all system steps. Oldest systems execute first (e.g.) system added at step 1 executes
    /// before system added at step 5.
    fn step(
        &self,
        _engine: &GameEngine<G>,
        _entity: &G::Entity,
        _next_entity: &mut G::Entity,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
