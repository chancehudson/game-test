use bevy_math::IRect;
use bevy_math::IVec2;
use bevy_math::Vec2;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::GameEngine;
use crate::STEP_DELAY;

pub mod emoji;
pub mod mob;
pub mod mob_spawn;
pub mod platform;
pub mod player;
pub mod portal;
pub mod rect;
pub mod text;

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
    fn position(&self) -> IVec2;
    fn position_f32(&self) -> Vec2 {
        let p = self.position();
        Vec2::new(p.x as f32, p.y as f32)
    }
    fn position_mut(&mut self) -> &mut IVec2;
    fn size(&self) -> IVec2;
    fn size_f32(&self) -> Vec2 {
        let s = self.size();
        Vec2::new(s.x as f32, s.y as f32)
    }
    fn velocity(&self) -> IVec2;

    // can the entity be simulated using public information
    fn pure(&self) -> bool;

    /// deterministic rng for entities, safe for replay
    fn rng(&self, step_index: &u64) -> ChaCha8Rng {
        let id = self.id();
        let first_half = (id >> 64) as u64; // Upper 64 bits
        let second_half = id as u64; // Lower 64 bits (cast truncates)

        let seed = first_half ^ second_half ^ step_index;
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Get an rng for the current state of the server
    fn rng_client(&self, step_index: &u64) -> ChaCha8Rng {
        self.rng(&(step_index + STEP_DELAY))
    }

    fn center(&self) -> IVec2 {
        let mut out = self.position();
        let size = self.size();
        out.x += size.x / 2;
        out.y += size.y / 2;
        out
    }

    fn rect(&self) -> IRect {
        let pos = self.position();
        let size = self.size();
        IRect::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y)
    }

    fn equal(&self, other: &Self) -> bool {
        self.position() == other.position()
            && self.velocity() == other.velocity()
            && self.size() == other.velocity()
    }
}

pub trait SEEntity: EEntity + Clone {
    fn step(&self, _engine: &mut GameEngine) -> Self
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
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub enum $enum_name {
            $(
                $variant($type),
            )*
        }

        impl SEEntity for $enum_name {
            fn step(&self, engine: &mut GameEngine) -> Self {
                match self {
                    $(
                        $enum_name::$variant(entity) => $enum_name::$variant(entity.step(engine)),
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

            fn size(&self) -> IVec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.size(),
                    )*
                }
            }

            fn position(&self) -> IVec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.position(),
                    )*
                }
            }

            fn position_mut(&mut self) -> &mut IVec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.position_mut(),
                    )*
                }
            }

            fn velocity(&self) -> IVec2 {
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
        Rect(rect::RectEntity),
        Player(player::PlayerEntity),
        Mob(mob::MobEntity),
        MobSpawner(mob_spawn::MobSpawnEntity),
        Platform(platform::PlatformEntity),
        Portal(portal::PortalEntity),
        Emoji(emoji::EmojiEntity),
        Text(text::TextEntity),
        // Item(ItemEntity),  // Uncomment when ready
    }
}

/// Macro to generate an entity struct with EEntity trait implementation
///
/// Usage:
/// ```
/// use game_test::entity_struct;
/// use game_test::engine::entity::EEntity;
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
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default, PartialEq)]
        $vis struct $name {
            #[serde(default)]
            pub id: u128,
            #[serde(default)]
            pub position: bevy_math::IVec2,
            #[serde(default)]
            pub size: bevy_math::IVec2,
            #[serde(default)]
            pub velocity: bevy_math::IVec2,
            #[serde(default)]
            pub pure: bool,
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $name {
            pub fn new(id: u128, position: bevy_math::IVec2, size: bevy_math::IVec2) -> Self {
                Self {
                    id,
                    position,
                    size,
                    ..Default::default()
                }
            }

            pub fn new_pure(position: bevy_math::IVec2, size: bevy_math::IVec2) -> Self {
                Self {
                    id: rand::random(), // use an unseeded random for entities that do not need to be communicated
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

            fn position(&self) -> bevy_math::IVec2 {
                self.position
            }

            fn position_mut(&mut self) -> &mut bevy_math::IVec2 {
                &mut self.position
            }

            fn size(&self) -> bevy_math::IVec2 {
                self.size
            }

            fn velocity(&self) -> bevy_math::IVec2 {
                self.velocity
            }

            fn pure(&self) -> bool {
                self.pure
            }
        }
    };
}
