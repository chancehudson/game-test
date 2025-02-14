use macroquad::prelude::*;

use super::Actor;
use super::Renderable;
use super::Sprite;

/// Represents an item on the ground
pub struct Item {
    sprite: Sprite,
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
}

impl Item {
    pub fn new(texture_name: &str, position: Vec2, velocity: Vec2) -> Self {
        Self {
            sprite: Sprite::new(texture_name, 52.0, 52.0),
            position,
            velocity,
            size: Vec2::new(52., 52.),
        }
    }
}

impl Renderable for Item {
    fn render(&mut self, _step_len: f32) {
        let time = get_time() as f32;
        // the bobbing/floating animation
        let y_offset = (time * 4.).sin() * 3.0;
        self.sprite
            .draw_frame(0, self.position.x, self.position.y + y_offset, false);
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
