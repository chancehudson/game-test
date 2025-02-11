use macroquad::prelude::*;

use super::Actor;
use super::AnimatedEntity;

pub struct Player {
    pub texture: AnimatedEntity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
}

impl Player {
    pub fn new() -> Self {
        Self {
            texture: AnimatedEntity::new("assets/robo.png", 97.0, 117.0, 17),
            position: Vec2::new(100., 100.),
            velocity: Vec2::new(0., 0.),
            size: Vec2::new(97., 117.),
        }
    }
}

impl Actor for Player {
    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }

    fn render(&mut self) {
        if is_key_down(KeyCode::Right) {
            self.texture.flip_x = true;
        } else if is_key_down(KeyCode::Left) {
            self.texture.flip_x = false;
        }
        self.texture.position = self.position;
        self.texture.set_animation(0); // Set to first animation (e.g., idle)
        self.texture.update();        // Update animation frame
        self.texture.draw();          // Draw current frame
        // draw_circle(self.position.x + self.size.x / 2., self.position.y + self.size.y /2., self.size.x/2., GREEN);
    }
}
