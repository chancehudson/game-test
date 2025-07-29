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
        false
    }

    /// Step the system, provided the engine, and the entity the system
    /// is attached to. Underlying entity type may be extracted with, for example,
    /// `let player_entity = entity.extract_ref_mut::<PlayerEntity>()`
    ///
    /// the system may freely mutate the entity. The entity step logic executes _after_
    /// all system steps. Oldest systems execute first (e.g.) system added at step 1 executes
    /// before system added at step 5.
    fn step(&self, _engine: &GameEngine<G>, _entity: &mut G::Entity) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

/// Creater a wrapper enum for EntitySystem polymorphism.
#[macro_export]
macro_rules! engine_entity_system_enum {
    (
        $game_logic:ident,
        $vis:vis enum $name:ident {
            $(
                $variant_name:ident($variant_type:ty)
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        $vis enum $name {
            $(
                $variant_name($variant_type),
            )*
        }

        impl $name {
            /// Retrieve a runtime TypeId for an instance.
            fn type_id(&self) -> std::any::TypeId {
                match self {
                    $(
                        $name::$variant_name(_) => std::any::TypeId::of::<$variant_type>(),
                    )*
                }
            }

            fn as_any(&self) -> &dyn Any {
                match self {
                    $(
                        $name::$variant_name(entity) => entity,
                    )*
                }
            }
        }

        $(
            impl From<$variant_type> for $name {
                fn from(value: $variant_type) -> Self {
                    $name::$variant_name(value)
                }
            }
        )*

        impl keind::prelude::EEntitySystem<$game_logic> for $name {
            fn step(&self, engine: &keind::prelude::GameEngine<$game_logic>, entity: &mut <$game_logic as keind::prelude::GameLogic>::Entity) -> Option<Self> {
                match self {
                    $(
                        $name::$variant_name(system) => system.step(engine, entity).map(|v| $name::from(v)),
                    )*
                }
            }
        }
    };
}
