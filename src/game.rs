use macroquad::prelude::*;

use super::Player;
use super::Map;
use super::Item;

// in pixels per second per second
const GRAVITY_ACCEL: f32 = 1200.0;

pub trait Actor {
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn rect(&self) -> Rect;

    fn render(&mut self);
    fn step_physics(&mut self, step_len: f32, map: &Map) {
        self.velocity_mut().y += GRAVITY_ACCEL * step_len;
        let dx = self.velocity_mut().x * step_len;
        let dy = self.velocity_mut().y * step_len;
        self.move_x(dx, &map);
        self.move_y(dy, &map);
    }

    fn move_x(&mut self, dx: f32, map: &Map) {
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

    fn move_y(&mut self, dy: f32, map: &Map) {
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

            for solid in &map.solids {
                if let Some(overlap) = solid.intersect(new_player_rect) {
                    // only collide if we're at the top of the platform
                    if overlap.h < 1. && overlap.y == solid.y {
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

pub struct GameState {
    player: Player,
    active_map: Map,
    actors: Vec<Box<dyn Actor>>,
    last_step: f64,
}

impl GameState {
    pub async fn new() -> Self {
        let player = Player::new();
        GameState {
            player,
            active_map: Map::new().await,
            actors: vec![],
            last_step: 0.0,
        }
    }

    // center on the player, except if we're at the edge of a map
    // then lock the camera viewport edge to the edge of the map
    pub fn render_camera(&mut self) {
        let half_screen = Vec2::new(screen_width()/2., screen_height()/2.);
        let camera = Camera2D::from_display_rect(
            Rect::new(
                (self.player.position.x - half_screen.x).clamp(0., self.active_map.size.x - screen_width()),
                (self.player.position.y + half_screen.y).clamp(0., self.active_map.size.y + 40.), // 40 is the padding at the bottom
                screen_width(), -screen_height()));
        set_camera(&camera);
    }

    pub fn input(&mut self, step_len: f32) {
        const ACCEL_RATE: f32 = 700.0;
        const DECEL_RATE: f32 = 800.0;
        const MAX_VELOCITY: f32 = 500.0;
        if is_key_down(KeyCode::Right) {
            self.player.velocity.x += ACCEL_RATE * step_len;
            if self.player.velocity.x < 0.0 {
                self.player.velocity.x += DECEL_RATE * step_len;
            }
        } else if is_key_down(KeyCode::Left) {
            self.player.velocity.x -= ACCEL_RATE * step_len;
            if self.player.velocity.x > 0.0 {
                self.player.velocity.x -= DECEL_RATE * step_len;
            }
        } else if self.player.velocity.x.abs() > 0.0 {
            self.player.velocity.x = self.player.velocity.move_towards(Vec2::ZERO, DECEL_RATE * step_len).x;
        }

        if is_key_down(KeyCode::Down) && is_key_pressed(KeyCode::Space) && self.player.velocity.y == 0. {
            self.player.position.y += 2.0;
        } else if is_key_pressed(KeyCode::Space) {
            // TODO: check if we're standing on a platform first
            self.player.velocity.y = -300.0;
        }

        if is_key_pressed(KeyCode::Z) {
            // drop an item
            self.actors.push(
                Box::new(
                    Item::new("assets/stick.png", self.player.position.clone(), Vec2::new(0., -200.))
                )
            );
        }
        self.player.velocity = self.player.velocity.clamp(Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY), Vec2::new(MAX_VELOCITY, MAX_VELOCITY));
    }

    pub fn render(&mut self) {
        let time = get_time();
        let step_len = (time - self.last_step) as f32;
        self.last_step = time;

        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.active_map);
        }
        self.player.step_physics(step_len, &self.active_map);

        // begin rendering
        self.render_camera();
        self.active_map.render(step_len, self.player.position);
        self.input(step_len);
        self.player.render();
        for actor in &mut self.actors {
            actor.render();
        }
    }
}
