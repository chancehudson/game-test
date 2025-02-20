use bevy::math::Rect;
use bevy::math::Vec2;

use super::MapData;

// in pixels per second per second
pub const GRAVITY_ACCEL: f32 = 1200.0;
pub const MAX_VELOCITY: Vec2 = Vec2::new(300.0, 400.0);

pub fn move_x(body: Rect, velocity: Vec2, dx: f32, map: &MapData) -> (f32, f32) {
    if dx == 0. {
        return (body.min.x, velocity.x);
    }
    let mut body = body.clone();
    let mut velocity = velocity.clone();
    body.min.x += dx;
    body.max.x += dx;
    if body.max.x > map.size.x {
        body.min.x = map.size.x - body.width();
        body.max.x = map.size.x;
        velocity.x = 0.;
    } else if body.min.x < 0. {
        body.max.x = body.width();
        body.min.x = 0.;
        velocity.x = 0.;
    }
    (body.min.x, velocity.x)
}

pub fn move_y(
    body: bevy::math::Rect,
    velocity: bevy::math::Vec2,
    dy: f32,
    map: &MapData,
) -> (f32, f32) {
    if dy == 0. {
        return (body.min.y, velocity.y);
    }
    let sign = dy.signum();
    let dy_abs = dy.abs();
    let mut moved = 0.;
    let min_y = 0.0;
    let max_y = map.size.y - body.height();
    let mut position = body.min.clone();

    // if the character is jumping we don't care about collisions
    if dy.is_sign_positive() {
        position.y = (position.y + dy).clamp(min_y, max_y);
        return (position.y, velocity.y);
    }
    while moved < dy_abs + 1. {
        let mut new_player_rect = body.clone();
        new_player_rect.min.y += sign * moved;
        new_player_rect.max.y += sign * moved;

        for solid in &map.platforms {
            let solid_rect = bevy::math::Rect::new(
                solid.position.x,
                solid.position.y,
                solid.position.x + solid.size.x,
                solid.position.y + solid.size.y,
            );
            let overlap = solid_rect.intersect(new_player_rect);
            if overlap.is_empty() {
                continue;
            }
            // only collide if we're at the top of the platform
            if overlap.height() < 1. && (overlap.min.y - solid_rect.max.y).abs() < 1. {
                // we've collided, stop
                return ((new_player_rect.min.y - sign).clamp(min_y, max_y), 0.0);
            }
        }
        moved += 1.;
    }
    // position.y += dy;
    ((position.y + dy).clamp(min_y, max_y), velocity.y)
}

pub trait Actor {
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn rect(&self) -> Rect;

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        self.step_physics_default(step_len, map);
    }

    fn step_physics_default(&mut self, step_len: f32, map: &MapData) {
        self.velocity_mut().y += -GRAVITY_ACCEL * step_len;
        let dx = self.velocity_mut().x * step_len;
        let dy = self.velocity_mut().y * step_len;
        self.move_x(dx, &map);
        self.move_y(dy, &map);
    }

    fn move_x(&mut self, dx: f32, map: &MapData) {
        let (new_x, new_vel_x) = move_x(self.rect(), self.velocity_mut().clone(), dx, map);
        self.position_mut().x = new_x;
        self.velocity_mut().x = new_vel_x;
    }

    fn move_y(&mut self, dy: f32, map: &MapData) {
        let (new_y, new_vel_y) = move_y(self.rect(), self.velocity_mut().clone(), dy, map);
        self.position_mut().y = new_y;
        self.velocity_mut().y = new_vel_y;
    }
}
