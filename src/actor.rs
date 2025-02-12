use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;

use super::MapData;

// in pixels per second per second
const GRAVITY_ACCEL: f32 = 1200.0;

pub trait Actor {
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn rect(&self) -> Rect;

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        self.step_physics_default(step_len, map);
    }

    fn step_physics_default(&mut self, step_len: f32, map: &MapData) {
        self.velocity_mut().y += GRAVITY_ACCEL * step_len;
        let dx = self.velocity_mut().x * step_len;
        let dy = self.velocity_mut().y * step_len;
        self.move_x(dx, &map);
        self.move_y(dy, &map);
    }

    fn move_x(&mut self, dx: f32, map: &MapData) {
        let rect = self.rect();
        self.position_mut().x += dx;
        if self.position_mut().x + rect.w > map.size.x {
            self.position_mut().x = map.size.x - rect.w;
            self.velocity_mut().x = 0.;
        } else if self.position_mut().x < 0. {
            self.position_mut().x = 0.;
            self.velocity_mut().x = 0.;
        }
    }

    fn move_y(&mut self, dy: f32, map: &MapData) {
        let sign = dy.signum();
        let dy_abs = dy.abs();
        let mut moved = 0.;
        let min_y = -self.rect().h;
        let max_y = map.size.y - self.rect().h;

        // if the character is jumping we don't care about collisions
        if dy.is_sign_negative() {
            let position = self.position_mut();
            position.y = (position.y + dy).clamp(min_y, max_y);
            return;
        }
        while moved < dy_abs + 1. {
            let mut new_player_rect = self.rect();
            new_player_rect.y += sign * moved;

            for solid in &map.platforms {
                let solid_rect = Rect::new(
                    solid.position.x,
                    solid.position.y,
                    solid.size.x,
                    solid.size.y,
                );
                if let Some(overlap) = solid_rect.intersect(new_player_rect) {
                    // only collide if we're at the top of the platform
                    if overlap.h < 1. && overlap.y == solid_rect.y {
                        // we've collided, stop
                        let position = self.position_mut();
                        position.y = (new_player_rect.y - sign).clamp(min_y, max_y);
                        let velocity = self.velocity_mut();
                        velocity.y = 0.0;
                        return;
                    }
                }
            }
            moved += 1.;
        }
        let position = self.position_mut();
        // position.y += dy;
        position.y = (position.y + dy).clamp(min_y, max_y);
    }
}
