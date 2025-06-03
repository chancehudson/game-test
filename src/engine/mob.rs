use bevy::math::Vec2;

use crate::MapData;

use super::entity::Entity;
use super::entity::EntityInput;

#[derive(Clone, Debug)]
pub struct MobEntity {
    pub id: u64,
    pub position: Vec2,
    pub size: Vec2,
    pub mob_type: u64,
}

impl Entity for MobEntity {
    fn id(&self) -> u64 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn step(&mut self, inputs: Option<&EntityInput>, map: &MapData) -> Self {
        let next_self = self.clone();
        let mut velocity = Vec2::new(0., -800.);

        if let Some(input) = inputs {
            if input.move_left {
                velocity.x -= 100.;
            }
            if input.move_right {
                velocity.x += 100.;
            }
        }
        next_self
    }
}
