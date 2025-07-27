use std::fmt::Debug;

use bevy_math::IRect;
use bevy_math::IVec2;
use bevy_math::Vec2;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro64StarStar;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

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

pub trait EEntityManager: EEntity {
    fn systems_by_type<T: EEntitySystem + 'static>(&self) -> Vec<Rc<T>> {
        self.systems()
            .iter()
            .filter_map(|system| (system.clone() as Rc<dyn Any>).downcast::<T>().ok())
            .collect()
    }

    fn has_system<T: EEntitySystem + 'static>(&self) -> bool {
        !self.systems_by_type::<T>().is_empty()
    }
}

/// An entity that exists inside the engine.
#[typetag::serde(tag = "type")]
pub trait EEntity: Debug + Any {
    fn systems(&self) -> &Vec<Rc<dyn EEntitySystem>>;

    fn state(&self) -> &BaseEntityState;
    fn state_mut(&mut self) -> &mut BaseEntityState;

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

    /// Get a unique owned reference to an EEntity.
    /// Convert to an `Rc<dyn EEntity>` for use with an engine.
    /// e.g. get cloned box, mutate box, returnRc
    fn clone_box(&self) -> Box<dyn EEntity>;
    fn clone_se_box(&self) -> Box<dyn SEEntity>;

    fn clone_arc(&self) -> Rc<dyn EEntity> {
        Rc::from(self.clone_box())
    }

    fn step_systems(&self, engine: &GameEngine, next_self_maybe: &mut Option<Box<dyn SEEntity>>);
}

impl<T> EEntityManager for T where T: SEEntity {}

#[typetag::serde(tag = "type")]
pub trait SEEntity: EEntity {
    fn step(&self, _engine: &GameEngine) -> Option<Box<dyn SEEntity>> {
        None
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BaseEntityState {
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
}

#[derive(Default, Serialize, Deserialize)]
pub struct EntitySystemsVec(Vec<Rc<dyn EEntitySystem>>);

impl Debug for EntitySystemsVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        Ok(())
    }
}

impl Clone for EntitySystemsVec {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|v| v.clone_arc()).collect::<Vec<_>>())
    }
}

/// Properties that all engine entities have. This macro is optional, you may
/// implement EEntity explicitly elsewhere. Check entity/item.rs for an example.
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
        #[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            #[serde(default)]
            pub state: BaseEntityState,
            pub systems: EntitySystemsVec,
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }


        impl $name {
            pub fn new(state: BaseEntityState, systems: Vec<std::rc::Rc<dyn EEntitySystem>>) -> Self {
                Self {
                    state,
                    systems: crate::entity::EntitySystemsVec(systems),
                    ..Default::default()
                }
            }
        }

        #[typetag::serde]
        impl EEntity for $name {
            fn systems(&self) -> &Vec<std::rc::Rc<dyn EEntitySystem>> {
                &self.systems.0
            }

            fn state(&self) -> &BaseEntityState {
                &self.state
            }

            fn state_mut(&mut self) -> &mut BaseEntityState {
                &mut self.state
            }

            fn step_systems(&self, engine: &GameEngine, next_self_maybe: &mut Option<Box<dyn SEEntity>>) {
                let mut next_systems: Vec<std::rc::Rc<dyn EEntitySystem>> = Vec::new();
                for system in &self.systems.0 {
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
                        *next_self_maybe = Some(self.clone_se_box());
                    }
                    let next_self = next_self_maybe.as_mut().unwrap();
                    // systems determine whether a clone is necessary
                    if let Some(next_system) = system.step(engine, &mut **next_self) {
                        next_systems.push(std::rc::Rc::from(next_system));
                    } else {
                        next_systems.push(system.clone());
                    }
                }
                // if we did a clone, insert next_systems into clone
                if let Some(next_self) = next_self_maybe.as_mut() {
                    let any_ref: &mut dyn std::any::Any = &mut **next_self;
                    let next_self_concrete = any_ref.downcast_mut::<Self>().expect("downcast into self failed");
                    next_self_concrete.systems = crate::entity::EntitySystemsVec(next_systems);
                }
            }

            fn clone_se_box(&self) -> Box<dyn SEEntity> {
                Box::new(self.clone())
            }

            fn clone_box(&self) -> Box<dyn EEntity> {
                Box::new(self.clone())
            }
        }
    };
}
