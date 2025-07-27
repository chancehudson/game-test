/// Systems that are applied to entities
///
use crate::prelude::*;

pub mod attach;
pub mod disappear;
pub mod gravity;

#[typetag::serde(tag = "type")]
pub trait EEntitySystem: Any {
    /// Step the system, provided the engine, and the entity the system
    /// is attached to. Underlying entity type may be extracted with, for example,
    /// `let player_entity = entity.extract_ref_mut::<PlayerEntity>()`
    ///
    /// the system may freely mutate the entity. The entity step logic executes _after_
    /// all system steps. Oldest systems execute first (e.g.) system added at step 1 executes
    /// before system added at step 5.
    fn step(
        &self,
        _engine: &GameEngine,
        _entity: &mut dyn SEEntity,
    ) -> Option<Box<dyn EEntitySystem>> {
        None
    }

    /// Readonly access to entity. Determine if write access
    /// is needed.
    /// Return true to mutate self or entity
    /// Engine events may be sent here
    fn prestep(&self, _engine: &GameEngine, _entity: &Rc<dyn SEEntity>) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn EEntitySystem>;
    fn clone_arc(&self) -> Rc<dyn EEntitySystem> {
        Rc::from(self.clone_box())
    }
}
