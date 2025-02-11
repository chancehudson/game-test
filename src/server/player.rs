use macroquad::prelude::Vec2;
use macroquad::prelude::Rect;

use super::Actor;
use super::MapData;

const MAX_VELOCITY: f32 = 500.0;

pub struct Player {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(100., 0.),
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

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        self.step_physics_default(step_len, map);
        self.velocity = self.velocity.clamp(Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY), Vec2::new(MAX_VELOCITY, MAX_VELOCITY));
    }
}
