use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "server")]
use rand::Rng;

#[cfg(feature = "server")]
use super::timestamp;
use super::Actor;
use super::MapData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub id: u64,
    pub mob_type: u64,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub max_velocity: f32,

    pub moving_to: Option<Vec2>,
    pub move_start: f32,
}

impl Actor for Mob {
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
        // simple logic to control the mob
        let accel_rate = 700.0;
        if self.moving_to.is_none() {
            #[cfg(feature = "server")]
            if rand::rng().random_bool(0.00001) {
                self.move_start = timestamp();
                self.moving_to = Some(Vec2::new(
                    rand::rng().random_range(0.0..map.size.x),
                    rand::rng().random_range(0.0..map.size.y),
                ));
            }
            self.velocity.x = self
                .velocity
                .move_towards(Vec2::ZERO, accel_rate * step_len)
                .x;
            self.step_physics_default(step_len, map);
            return;
        }
        let moving_to = self.moving_to.clone().unwrap();
        let move_left = self.position.x > moving_to.x;
        let move_right = self.position.x < moving_to.x;
        if move_right {
            self.velocity_mut().x += accel_rate * step_len;
            self.velocity.x = self.velocity.x.clamp(-self.max_velocity, self.max_velocity);
        } else if move_left {
            self.velocity_mut().x -= accel_rate * step_len;
            self.velocity.x = self.velocity.x.clamp(-self.max_velocity, self.max_velocity);
        } else if self.velocity.x.abs() > 0.0 {
            self.velocity.x = self
                .velocity
                .move_towards(Vec2::ZERO, accel_rate * step_len)
                .x;
        }
        if (self.position.x - moving_to.x).abs() < 10.0 {
            self.moving_to = None;
        }
        #[cfg(feature = "server")]
        if timestamp() - self.move_start > 10.0 {
            self.moving_to = None;
        }
        self.step_physics_default(step_len, map);
    }
}
