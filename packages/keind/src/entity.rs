use std::any::Any;
use std::fmt::Debug;

use bevy_math::IRect;
use bevy_math::IVec2;
use bevy_math::Vec2;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro64StarStar;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BaseEntityState {
    #[serde(default)]
    pub id: u128,
    #[serde(default)]
    pub position: IVec2,
    #[serde(default)]
    pub size: IVec2,
    #[serde(default)]
    pub velocity: IVec2,
    #[serde(default)]
    pub player_creator_id: Option<u128>,
}

/// A _steppable_ entity that exists in the engine.
pub trait SEEntity<G: GameLogic + 'static>: EEntity<G> {
    fn step(&self, _engine: &GameEngine<G>) -> Option<Self> {
        None
    }
}

/// An entity that exists inside the engine.
pub trait EEntity<G: GameLogic + 'static>: Debug + Any + Clone {
    fn systems(&self) -> &Vec<RefPointer<G::System>>;
    fn systems_mut(&mut self) -> &mut Vec<RefPointer<G::System>>;

    fn state(&self) -> &BaseEntityState;
    fn state_mut(&mut self) -> &mut BaseEntityState;

    fn systems_by_type<T: EEntitySystem<G> + 'static>(&self) -> Vec<&T> {
        self.systems()
            .iter()
            .filter_map(|system| (system as &dyn Any).downcast_ref::<T>())
            .collect()
    }

    fn has_system<T: EEntitySystem<G> + 'static>(&self) -> bool {
        !self.systems_by_type::<T>().is_empty()
    }

    fn id(&self) -> u128 {
        self.state().id
    }

    fn position(&self) -> IVec2 {
        self.state().position
    }

    fn position_f32(&self) -> Vec2 {
        let p = self.position();
        Vec2::new(p.x as f32, p.y as f32)
    }

    fn size(&self) -> IVec2 {
        self.state().size
    }

    fn size_f32(&self) -> Vec2 {
        let s = self.size();
        Vec2::new(s.x as f32, s.y as f32)
    }

    fn velocity(&self) -> IVec2 {
        self.state().velocity
    }

    fn player_creator_id(&self) -> Option<u128> {
        self.state().player_creator_id
    }

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

    fn step_systems(&self, engine: &GameEngine<G>, next_self_maybe: &mut Option<G::Entity>);
}

#[macro_export]
macro_rules! engine_entity_enum {
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

        impl $crate::prelude::KPoly for $name {
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

            fn get_ref<T: 'static>(&self) -> Option<&T> {
                self.as_any().downcast_ref::<T>()
            }

            fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
                match self {
                    $(
                        $name::$variant_name(entity) => {
                            let entity: &mut dyn Any = entity;
                            entity.downcast_mut::<T>()
                        },
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

        impl $crate::prelude::SEEntity<$game_logic> for $name {
            fn step(&self, engine: &$crate::prelude::GameEngine<$game_logic>) -> Option<Self> {
                match self {
                    $(
                        $name::$variant_name(entity) => entity.step(engine).map(|out| $name::$variant_name(out)),
                    )*
                }
            }
        }

        impl $crate::prelude::EEntity<$game_logic> for $name {
            fn systems(&self) -> &Vec<keind::prelude::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>> {
                match self {
                    $(
                        $name::$variant_name(entity) => entity.systems(),
                    )*
                }
            }

            fn systems_mut(&mut self) -> &mut Vec<keind::prelude::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>> {
                match self {
                    $(
                        $name::$variant_name(entity) => entity.systems_mut(),
                    )*
                }
            }

            fn state(&self) -> &keind::prelude::BaseEntityState {
                match self {
                    $(
                        $name::$variant_name(entity) => entity.state(),
                    )*
                }
            }

            fn state_mut(&mut self) -> &mut keind::prelude::BaseEntityState {
                match self {
                    $(
                        $name::$variant_name(entity) => entity.state_mut(),
                    )*
                }
            }

            fn step_systems(&self, engine: &keind::prelude::GameEngine<$game_logic>, next_self_maybe: &mut Option<$name>) {
                match self {
                    $(
                        $name::$variant_name(entity) => {
                            entity.step_systems(engine, next_self_maybe);
                        },
                    )*
                }
            }
        }
    };
}

/// Properties that all engine entities have. This macro is optional, you may
/// implement SEEntity explicitly elsewhere.
#[macro_export]
macro_rules! entity_struct {
    // Pattern for struct with additional fields and trait implementation
    (
        $game_logic:ident,
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),*
            $(,)?
        }
    ) => {

        #[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            #[serde(default)]
            pub state: keind::prelude::BaseEntityState,
            pub systems: Vec<$crate::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>>,
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }


        impl $name {
            pub fn new(state: keind::prelude::BaseEntityState, systems: Vec<$crate::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>>) -> Self {
                Self {
                    state,
                    systems,
                    ..Default::default()
                }
            }
        }

        impl keind::prelude::EEntity<$game_logic> for $name {
            fn systems(&self) -> &Vec<$crate::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>> {
                &self.systems
            }

            fn systems_mut(&mut self) -> &mut Vec<$crate::RefPointer<<$game_logic as keind::prelude::GameLogic>::System>> {
                &mut self.systems
            }

            fn state(&self) -> &keind::prelude::BaseEntityState {
                &self.state
            }

            fn state_mut(&mut self) -> &mut keind::prelude::BaseEntityState {
                &mut self.state
            }

            fn step_systems(&self, engine: &keind::prelude::GameEngine<$game_logic>, next_self_maybe: &mut Option<EngineEntity>) {
                type EngineEntity = <$game_logic as keind::prelude::GameLogic>::Entity;
                type EngineSystem = <$game_logic as keind::prelude::GameLogic>::System;
                let mut next_systems: Vec<$crate::RefPointer<EngineSystem>> = Vec::new();
                for system in self.systems() {
                    let entity_rc = engine
                        .entity_by_id_untyped(&self.id(), None)
                        .expect("entity being stepped but not in engine");
                    if !system.prestep(engine, &entity_rc) {
                        next_systems.push(system.clone());
                        continue;
                    }
                    // the system has requested a clone, we need to clone the parent entity
                    // as well
                    if next_self_maybe.is_none() {
                        *next_self_maybe = Some(EngineEntity::from(self.clone()));
                    }
                    let next_self = next_self_maybe.as_mut().unwrap();
                    // systems determine whether a clone is necessary
                    if let Some(next_system) = system.step(engine, &mut *next_self) {
                        next_systems.push($crate::RefPointer::from(next_system));
                    } else {
                        next_systems.push(system.clone());
                    }
                }
                // if we did a clone, insert next_systems into clone
                if let Some(next_self) = next_self_maybe.as_mut() {
                    let any_ref: &mut dyn std::any::Any = &mut *next_self;
                    let next_self_concrete = any_ref
                        .downcast_mut::<Self>()
                        .expect("downcast into self failed");
                    *next_self_concrete.systems_mut() = next_systems;
                }
            }

        }
    };
}
