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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

impl Default for BaseEntityState {
    fn default() -> Self {
        Self {
            id: 0,
            position: IVec2::default(),
            size: IVec2::default(),
            velocity: IVec2::default(),
            player_creator_id: None,
        }
    }
}

/// A _steppable_ entity that exists in the engine.
pub trait SEEntity<G: GameLogic + 'static>: EEntity<G> {
    /// Return a boolean indicating whether the entity needs to mutate.
    /// Returning false means `step` will not be called on the
    /// entity (though it may be called on attached systems).
    fn prestep(&self, _engine: &GameEngine<G>) -> bool {
        true
    }

    /// Mutate the next version of the entity.
    fn step(&self, _engine: &GameEngine<G>, _next_self: &mut Self) {}
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
            .filter_map(|system| system.extract_ref::<T>())
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
            pub state: $crate::prelude::BaseEntityState,
            #[serde(default)]
            pub systems: Vec<$crate::RefPointer<<$game_logic as $crate::prelude::GameLogic>::System>>,
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }


        impl $name {
            pub fn new(state: $crate::prelude::BaseEntityState, systems: Vec<$crate::RefPointer<<$game_logic as $crate::prelude::GameLogic>::System>>) -> Self {
                Self {
                    state,
                    systems,
                    ..Default::default()
                }
            }
        }

        impl $crate::prelude::EEntity<$game_logic> for $name {
            fn systems(&self) -> &Vec<$crate::RefPointer<<$game_logic as $crate::prelude::GameLogic>::System>> {
                &self.systems
            }

            fn systems_mut(&mut self) -> &mut Vec<$crate::RefPointer<<$game_logic as $crate::prelude::GameLogic>::System>> {
                &mut self.systems
            }

            fn state(&self) -> &$crate::prelude::BaseEntityState {
                &self.state
            }

            fn state_mut(&mut self) -> &mut $crate::prelude::BaseEntityState {
                &mut self.state
            }

            fn step_systems(&self, engine: &$crate::prelude::GameEngine<$game_logic>, next_self_maybe: &mut Option<EngineEntity>) {
                type EngineEntity = <$game_logic as $crate::prelude::GameLogic>::Entity;
                type EngineSystem = <$game_logic as $crate::prelude::GameLogic>::System;
                let mut next_systems: Vec<$crate::RefPointer<EngineSystem>> = Vec::new();
                let entity_ptr = engine
                    .entity_by_id_untyped(&self.id(), None)
                    .expect("entity being stepped but not in engine");
                for system in self.systems() {
                    if !system.prestep(engine, &entity_ptr) {
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
                    if let Some(next_system) = system.step(engine, entity_ptr, &mut *next_self) {
                        next_systems.push($crate::RefPointer::from(next_system));
                    } else {
                        // No new value was returned, the system is removed
                    }
                }
                // if we did a clone, insert next_systems into clone
                if let Some(next_self) = next_self_maybe.as_mut() {
                    *next_self.systems_mut() = next_systems;
                }
            }

        }
    };
}
