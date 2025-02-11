use macroquad::prelude::Vec2;
use macroquad::prelude::Rect;

use super::Actor;

/// Represents an item on the ground
pub struct Item {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
}

impl Item {
    pub fn new(texture_name: &str, position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            size: Vec2::new(52., 52.),
        }
    }
}

impl Actor for Item {
    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }
    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }

}
