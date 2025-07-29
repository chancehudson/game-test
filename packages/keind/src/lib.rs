/// keind is a deterministic game engine optimized for execution
/// in zkvm environments. As a result it's quick in most other
/// environments.
///
mod engine;
mod entity;
mod event;
pub mod prelude;
mod system;

/// In a zkvm we are truly single threaded, and have no use for atomics.
/// We try to remove all atomics and thread support to improve performance.
#[cfg(feature = "zk")]
pub use std::rc::Rc as RefPointer;
#[cfg(not(feature = "zk"))]
pub use std::sync::Arc as RefPointer;

/// A helper trait for polymorphism in the keind game engine.
pub trait KPoly {
    /// Retrieve a runtime TypeId for an instance.
    fn type_id(&self) -> std::any::TypeId;

    fn as_any(&self) -> &dyn std::any::Any;

    fn get_ref<T: 'static>(&self) -> Option<&T>;

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

use serde::Deserialize;
use serde::Serialize;

pub trait GameLogic: Clone + Serialize + for<'de> Deserialize<'de> + 'static {
    type Entity: Clone + entity::SEEntity<Self> + KPoly + Serialize + for<'de> Deserialize<'de>; // Enum wrapping all possible entities
    type System: Clone + KPoly + Serialize + for<'de> Deserialize<'de>; // Enum wrapping all possible systems
    type Input: Default + Clone + Serialize + for<'de> Deserialize<'de>; // User input
    type Event: Clone + Serialize + for<'de> Deserialize<'de>; // Game event, distinct from Engine event, which is internal to keind
}
