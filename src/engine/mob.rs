use bevy::math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::MapData;

use super::entity::Entity;
use super::entity::EntityInput;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobEntity {
    pub id: u128,
    pub position: Vec2,
    pub size: Vec2,
    pub mob_type: u64,
}

impl Entity for MobEntity {
    fn id(&self) -> u128 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
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
