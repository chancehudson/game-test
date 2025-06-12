use bevy_math::Rect;
use bevy_math::Vec2;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Deserialize;
use serde::Serialize;

use super::emoji::EmojiEntity;
use super::mob::MobEntity;
use super::mob_spawner::MobSpawnEntity;
use super::platform::PlatformEntity;
use super::player::PlayerEntity;
use super::portal::PortalEntity;
use super::rect::RectEntity;
use crate::engine::GameEngine;
use crate::STEP_DELAY;

/// Inputs that may be applied to any entity.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EntityInput {
    pub jump: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub crouch: bool,
    pub attack: bool,
    pub enter_portal: bool,
    pub admin_enable_debug_markers: bool,
    pub show_emoji: bool,
}

/// An entity that exists inside the engine.
pub trait EEntity {
    fn id(&self) -> u128;
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn size(&self) -> Vec2;
    fn velocity(&self) -> Vec2;

    // can the entity be simulated using public information
    fn pure(&self) -> bool;

    /// deterministic rng for entities, safe for replay
    fn rng(&self, step_index: &u64) -> StdRng {
        let id = self.id();
        let first_half = (id >> 64) as u64; // Upper 64 bits
        let second_half = id as u64; // Lower 64 bits (cast truncates)

        let seed = first_half ^ second_half ^ step_index;
        StdRng::seed_from_u64(seed)
    }

    /// Get an rng for the current state of the server
    fn rng_client(&self, step_index: &u64) -> StdRng {
        self.rng(&(step_index + STEP_DELAY))
    }

    fn center(&self) -> Vec2 {
        let mut out = self.position();
        let size = self.size();
        out.x += size.x / 2.0;
        out.y += size.y / 2.0;
        out
    }

    fn rect(&self) -> Rect {
        let pos = self.position();
        let size = self.size();
        Rect::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y)
    }

    fn equal(&self, other: &Self) -> bool {
        self.position() == other.position()
            && self.velocity() == other.velocity()
            && self.size() == other.velocity()
    }
}

pub trait SEEntity: EEntity + Clone {
    fn step(&self, _engine: &mut GameEngine, _step_index: &u64) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
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

        impl SEEntity for $enum_name {
            fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
                match self {
                    $(
                        $enum_name::$variant(entity) => $enum_name::$variant(entity.step(engine, step_index)),
                    )*
                }
            }
        }

        impl EEntity for $enum_name {
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

            fn velocity(&self) -> Vec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.velocity(),
                    )*
                }
            }

            fn pure(&self) -> bool {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.pure(),
                    )*
                }
            }
        }
    };
}

engine_entity_enum! {
    EngineEntity {
        Rect(RectEntity),
        Player(PlayerEntity),
        Mob(MobEntity),
        MobSpawner(MobSpawnEntity),
        Platform(PlatformEntity),
        Portal(PortalEntity),
        Emoji(EmojiEntity),
        // Item(ItemEntity),  // Uncomment when ready
    }
}

/// Macro to generate an entity struct with EEntity trait implementation
///
/// Usage:
/// ```
/// // Basic entity with default trait implementation
/// entity_struct! {
///     pub struct MyEntity {
///         pub custom_field: f32,
///         private_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! entity_struct {
    // Pattern for struct with additional fields and trait implementation
    (
        $(#[$struct_attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_attr])*
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
        $vis struct $name {
            #[serde(default)]
            pub id: u128,
            #[serde(default)]
            pub position: bevy_math::Vec2,
            #[serde(default)]
            pub size: bevy_math::Vec2,
            #[serde(default)]
            pub velocity: bevy_math::Vec2,
            #[serde(default)]
            pub pure: bool,
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $name {
            pub fn new(id: u128, position: bevy_math::Vec2, size: bevy_math::Vec2) -> Self {
                Self {
                    id,
                    position,
                    size,
                    ..Default::default()
                }
            }
        }

        impl EEntity for $name {
            fn id(&self) -> u128 {
                self.id
            }

            fn position(&self) -> bevy_math::Vec2 {
                self.position
            }

            fn position_mut(&mut self) -> &mut bevy_math::Vec2 {
                &mut self.position
            }

            fn size(&self) -> bevy_math::Vec2 {
                self.size
            }

            fn velocity(&self) -> bevy_math::Vec2 {
                self.velocity
            }

            fn pure(&self) -> bool {
                self.pure
            }
        }
    };
}
