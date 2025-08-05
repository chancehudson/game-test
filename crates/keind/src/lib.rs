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
//#[cfg(feature = "zk")]
pub use std::rc::Rc as RefPointer;
//#[cfg(not(feature = "zk"))]
//pub use std::sync::Arc as RefPointer;

use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

pub trait KPoly {
    /// Retrieve a runtime TypeId for an instance.
    fn type_id(&self) -> std::any::TypeId;

    fn as_any(&self) -> &dyn std::any::Any;
    fn extract_ref<T: 'static>(&self) -> Option<&T>;
    fn extract_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

pub trait GameLogic: Clone + Serialize + for<'de> Deserialize<'de> + 'static {
    type Entity: entity::SEEntity<Self>
        + KPoly
        + Debug
        + Clone
        // + Send //
        // + Sync
        + Serialize
        + for<'de> Deserialize<'de>; // Enum wrapping all possible entities
    type System: KPoly + Debug + Clone + Serialize + for<'de> Deserialize<'de>; // Enum wrapping all possible systems
    type Input: Default + Clone + Serialize + for<'de> Deserialize<'de>; // User ninput
    type Event: Clone + Serialize + for<'de> Deserialize<'de>; // Game event, distinct from Engine event, which is internal to keind

    fn handle_game_events(
        engine: &engine::GameEngine<Self>,
        game_events: &Vec<RefPointer<Self::Event>>,
    );
}
