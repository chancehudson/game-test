use bevy::math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::actor::move_x;
use crate::actor::move_y;
use crate::MapData;

use super::entity::{Entity, EntityInput};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerEntity {
    pub id: u128,
    pub position: Vec2,
    pub size: Vec2,
}

impl Entity for PlayerEntity {
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
        let mut next_self = self.clone();
        let mut velocity = Vec2::new(0., -200.);
        if let Some(input) = inputs {
            if input.move_left {
                velocity.x -= 100.;
            }
            if input.move_right {
                velocity.x += 100.;
            }
        }
        let (x_pos, _x_vel) = move_x(self.rect(), velocity, velocity.x / 60., map);
        let (y_pos, _y_vel) = move_y(self.rect(), velocity, velocity.y / 60., map);
        next_self.position.x = x_pos;
        next_self.position.y = y_pos;
        next_self
    }
}
