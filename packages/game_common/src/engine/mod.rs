use bevy_math::IVec2;
use rand_xoshiro::Xoroshiro64StarStar;

use crate::prelude::*;

pub mod actor;
pub mod damage_calc;
pub mod game_event;
pub mod rewindable;
pub mod simple;

pub mod constants {
    pub const STEP_LEN_S: f64 = 1. / 60.;
    pub const STEP_LEN_S_F32: f32 = 1. / 60.;
    pub const STEPS_PER_SECOND: u64 = (1.0 / STEP_LEN_S_F32) as u64;
    pub const STEPS_PER_SECOND_I32: i32 = (1.0 / STEP_LEN_S_F32) as i32;
    pub const TRAILING_STATE_COUNT: u64 = 1;
}

/// Trait for initializing a game engine.
pub trait EngineInit {
    /// Directly mutate a game engine instance arbitrarily.
    /// TODO: require `engine.step_index == 0` ?
    fn init(&self, game_data: &GameData, engine: &mut GameEngine) -> anyhow::Result<()>;
}

pub type GameEngine = RewindableGameEngine;

// pub trait GameEngine {
//     /// A distinct identifier for the engine. Deterministic based on `seed`.
//     fn id(&self) -> &u128;
//     /// Engine seed for rng.
//     fn seed(&self) -> &u64;
//     /// Retrieve a random number generator that is reseeded each step.
//     fn rng(&mut self) -> &mut Xoroshiro64StarStar;
//     /// Generate a new deterministic id.
//     fn generate_id(&mut self) -> u128;
//     /// The dimension of the engine.
//     fn size(&self) -> &IVec2;
//     fn step_index(&self) -> &u64;
//
//     /// Run a single step of the game engine.
//     fn step(&mut self) -> Vec<GameEvent>;
//     /// Move many steps into the future.
//     fn step_to(&mut self, to_step: &u64) -> Vec<GameEvent> {
//         assert!(to_step > self.step_index());
//         let mut out = vec![];
//         for _ in 0..(to_step - self.step_index()) {
//             out.append(&mut self.step());
//         }
//         out
//     }
//
//     /// Register a game event that will be propagated upward from the engine to
//     /// any structures above
//     /// TODO: reconsider this architecture
//     fn register_game_event(&self, event: GameEvent);
//     /// Mutate the engine in response to a game event
//     fn process_game_event(&mut self, event: &GameEvent);
//
//     /// Schedule an entity for removal. This occurs at the end of the
//     /// target step (current step by default).
//     fn remove_entity(&self, id: &u128, step_index: Option<u64>, is_non_determ: bool);
//     /// Schedule an entity for creation. This occurs at the end of the
//     /// target step (current step by default)n.
//     fn spawn_entity(&self, entity: EngineEntity, step_index: Option<u64>, is_non_determ: bool);
//     /// Schedule an engine event. This occurs at the end of the
//     /// target step (current step by default).
//     fn register_event(&self, step_index: Option<u64>, event: EngineEvent);
//
//     /// Receive an `EngineEntity` reference without knowing the underlying
//     /// entity type.
//     fn entity_by_id_untyped(&self, id: &u128, step_index: Option<u64>) -> Option<&EngineEntity>;
//     /// Extract a reference to an entity by type.
//     fn entity_by_id(&self, id: &u128, step_index: Option<u64>) -> Option<&dyn EEntity>;
//     /// Access all entities of a specific type currently in the engine.
//     fn entities_by_type(&self) -> Vec<&dyn EEntity>;
//
//     fn input_for_entity(&self, id: &u128) -> &EntityInput;
// }
