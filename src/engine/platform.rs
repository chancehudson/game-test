use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::GameEngine;

use super::entity::Entity;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlatformEntity {
    pub id: u128,
    pub position: Vec2,
    pub size: Vec2,
    velocity: Vec2,
}

impl PlatformEntity {
    pub fn new(id: u128, position: Vec2, size: Vec2) -> Self {
        PlatformEntity {
            id,
            position,
            size,
            velocity: Vec2::new(0.0, 0.0),
        }
    }
}

impl Entity for PlatformEntity {
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

    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
        self.clone()
    }
}
