use bevy_math::IRect;
use bevy_math::IVec2;
use bevy_math::Vec2;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro64StarStar;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::GameEngine;

pub mod emoji;
pub mod item;
pub mod message;
pub mod mob;
pub mod mob_damage;
pub mod mob_spawn;
pub mod npc;
pub mod platform;
pub mod player;
pub mod portal;
pub mod rect;
pub mod text;

/// Inputs that may be applied to any entity.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EntityInput {
    pub jump: bool,
    pub jump_down: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub crouch: bool,
    pub attack: bool,
    pub enter_portal: bool,
    pub show_emoji: bool,
    pub respawn: bool,
    pub pick_up: bool,
}

/// An entity that exists inside the engine.
pub trait EEntity {
    fn id(&self) -> u128;
    fn position(&self) -> IVec2;
    fn position_f32(&self) -> Vec2 {
        let p = self.position();
        Vec2::new(p.x as f32, p.y as f32)
    }
    fn size(&self) -> IVec2;
    fn size_f32(&self) -> Vec2 {
        let s = self.size();
        Vec2::new(s.x as f32, s.y as f32)
    }
    fn velocity(&self) -> IVec2;

    fn player_creator_id(&self) -> Option<u128>;

    /// deterministic rng for entities, safe for replay
    fn rng(&self, step_index: &u64) -> Xoroshiro64StarStar {
        Xoroshiro64StarStar::seed_from_u64((self.id() as u64) + *step_index)
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
            && self.size() == other.size()
    }
}

pub trait SEEntity: EEntity + Clone {
    fn step<T: GameEngine>(&self, _engine: &T) -> Self
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
                $variant:ident($type:ty) = $id:expr
            ),* $(,)?
        }
    ) => {
        /// Get the u32 type ID for a concrete type T
        pub fn type_id_of<T: 'static>() -> Option<u32> {
            use std::any::TypeId;
            $(
                if TypeId::of::<T>() == TypeId::of::<$type>() {
                    return Some($id);
                }
            )*
            None
        }

        /// Type IDs for each entity variant
        pub mod entity_type_ids {
            $(
                #[allow(non_upper_case_globals)]
                pub const $variant: u32 = $id;
            )*
        }

        /// Enum to wrap all possible entity types
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub enum $enum_name {
            $(
                $variant($type),
            )*
        }

        impl $enum_name {
            /// Get the u32 type ID for this entity variant
            pub fn type_id(&self) -> u32 {
                match self {
                    $(
                        $enum_name::$variant(_) => entity_type_ids::$variant,
                    )*
                }
            }

            /// Get the TypeId for this entity variant
            pub fn runtime_type_id(&self) -> std::any::TypeId {
                match self {
                    $(
                        $enum_name::$variant(_) => std::any::TypeId::of::<$type>(),
                    )*
                }
            }

            /// Extract reference to inner value
            pub fn extract_ref<T: 'static>(&self) -> Option<&T> {
                use std::any::{Any, TypeId};
                $(
                    if TypeId::of::<T>() == TypeId::of::<$type>() {
                        if let $enum_name::$variant(inner) = self {
                            return (inner as &dyn Any).downcast_ref::<T>();
                        }
                    }
                )*
                None
            }

            /// Extract a mutable reference
            pub fn extract_ref_mut<T: 'static>(&mut self) -> Option<&mut T> {
                use std::any::{Any, TypeId};
                $(
                    if TypeId::of::<T>() == TypeId::of::<$type>() {
                        if let $enum_name::$variant(inner) = self {
                            return (inner as &mut dyn Any).downcast_mut::<T>();
                        }
                    }
                )*
                None
            }
        }

        impl SEEntity for $enum_name {
            fn step<T: GameEngine>(&self, engine: &T) -> Self {
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

            fn position_f32(&self) -> Vec2 {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.position_f32(),
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

            fn player_creator_id(&self) -> Option<u128> {
                match self {
                    $(
                        $enum_name::$variant(entity) => entity.player_creator_id(),
                    )*
                }
            }
        }
    };
}

engine_entity_enum! {
    EngineEntity {
        MobDamage(mob_damage::MobDamageEntity) = 0,
        Rect(rect::RectEntity) = 1,
        Player(player::PlayerEntity) = 2,
        Mob(mob::MobEntity) = 3,
        MobSpawner(mob_spawn::MobSpawnEntity) = 4,
        Platform(platform::PlatformEntity) = 5,
        Portal(portal::PortalEntity) = 6,
        Emoji(emoji::EmojiEntity) = 7,
        Text(text::TextEntity) = 8,
        Item(item::ItemEntity) = 9,
        Npc(npc::NpcEntity) = 10,
        Message(message::MessageEntity) = 11,
    }
}

/// Macro to generate an entity struct with EEntity trait implementation
///
/// Usage:
/// ```
/// use game_common::entity_struct;
/// use game_common::engine::entity::EEntity;
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
            pub player_creator_id: Option<u128>,
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
        }

        impl EEntity for $name {
            fn id(&self) -> u128 {
                self.id
            }

            fn position(&self) -> bevy_math::IVec2 {
                self.position
            }

            fn size(&self) -> bevy_math::IVec2 {
                self.size
            }

            fn velocity(&self) -> bevy_math::IVec2 {
                self.velocity
            }

            fn player_creator_id(&self) -> Option<u128> {
                self.player_creator_id
            }
        }
    };
}
