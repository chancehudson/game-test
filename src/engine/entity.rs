use bevy_math::Rect;
use bevy_math::Vec2;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Deserialize;
use serde::Serialize;

use super::mob::MobEntity;
use super::mob_spawner::MobSpawnEntity;
use super::platform::PlatformEntity;
use super::player::PlayerEntity;
use super::portal::PortalEntity;
use crate::engine::GameEngine;

/// Inputs that may be applied to any entity.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EntityInput {
    pub jump: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub crouch: bool,
    pub attack: bool,
    pub enter_portal: bool,
}

/// An entity that exists inside the engine.
pub trait Entity {
    fn id(&self) -> u128;
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn size(&self) -> Vec2;
    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self;

    /// deterministic rng for entities, safe for replay
    fn rng(&self, step_index: &u64) -> StdRng {
        let id = self.id();
        let first_half = (id >> 64) as u64; // Upper 64 bits
        let second_half = id as u64; // Lower 64 bits (cast truncates)

        let seed = first_half ^ second_half ^ step_index;
        StdRng::seed_from_u64(seed)
    }

    fn rect(&self) -> Rect {
        let pos = self.position();
        let size = self.size();
        Rect::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y)
    }
}

macro_rules! engine_entity_enum {
    (
        $enum_name:ident {
            $(
                $variant:ident($type:ty)
            ),* $(,)?
        }
    ) => {
        /// Enum to wrap all possible entity types
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum $enum_name {
            $(
                $variant($type),
            )*
        }

        impl Entity for $enum_name {
            fn id(&self) -> u128 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.id(),
                    )*
                }
            }

            fn size(&self) -> Vec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.size(),
                    )*
                }
            }

            fn position(&self) -> Vec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.position(),
                    )*
                }
            }

            fn position_mut(&mut self) -> &mut Vec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.position_mut(),
                    )*
                }
            }

            fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
                match self {
                    $(
                        $enum_name::$variant(entity) => $enum_name::$variant(entity.step(engine, step_index)),
                    )*
                }
            }
        }
    };
}

engine_entity_enum! {
    EngineEntity {
        Player(PlayerEntity),
        Mob(MobEntity),
        MobSpawner(MobSpawnEntity),
        Platform(PlatformEntity),
        Portal(PortalEntity),
        // Item(ItemEntity),  // Uncomment when ready
    }
}
