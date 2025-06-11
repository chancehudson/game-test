use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::player::PlayerEntity;
use crate::engine::GameEngine;

use super::entity::Entity;

fn default_size() -> Vec2 {
    Vec2::new(60., 80.)
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PortalEntity {
    #[serde(default)]
    pub id: u128,
    pub position: Vec2,
    #[serde(default = "default_size")]
    pub size: Vec2,
    #[serde(default)]
    velocity: Vec2,
    // destination map name
    pub to: String,
}

impl PortalEntity {
    pub fn can_enter(&self, player: &PlayerEntity) -> bool {
        !player.rect().intersect(self.rect()).is_empty()
    }
}

impl Entity for PortalEntity {
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
