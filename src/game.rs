use macroquad::prelude::*;

use super::Player;

// in pixels per second per second
const GRAVITY_ACCEL: f32 = 700.0;

pub trait Actor {
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn rect(&self) -> Rect;

    fn render(&self);
    fn step_physics(&mut self, step_len: f32, solids: &Vec<Rect>) {
        self.velocity_mut().y += GRAVITY_ACCEL * step_len;
        let dx = self.velocity_mut().x * step_len;
        let dy = self.velocity_mut().y * step_len;
        self.move_x(dx, solids);
        self.move_y(dy, solids);
    }

    fn move_x(&mut self, dx: f32, _solids: &Vec<Rect>) {
        self.position_mut().x += dx;
    }

    fn move_y(&mut self, dy: f32, solids: &Vec<Rect>) {
        let sign = dy.signum();
        let dy_abs = dy.abs();
        let mut moved = 0.;

        // if the character is jumping we don't care about collisions
        if dy.is_sign_negative() {
            let position = self.position_mut();
            position.y += dy;
            return;
        }
        while moved < dy_abs + 1. {
            let mut new_player_rect = self.rect();
            new_player_rect.y += sign * moved;

            for solid in solids {
                if let Some(overlap) = solid.intersect(new_player_rect) {
                    // only collide if we're at the top of the platform
                    if overlap.h < 1. && overlap.y == solid.y {
                        // we've collided, stop
                        let position = self.position_mut();
                        position.y = new_player_rect.y - sign;
                        let velocity = self.velocity_mut();
                        velocity.y = 0.0;
                        return;
                    }
                }
            }
            moved += 1.;
        }
        let position = self.position_mut();
        position.y += dy;
    }
}

pub struct Item {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
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

    fn render(&self) {
        draw_rectangle(self.position.x, self.position.y, self.size.x, self.size.y, PINK);
    }
}

/// We'll separate solids and visuals
pub struct Map {
    solids: Vec<Rect>,
}

pub struct GameState {
    player: Player,
    active_map: Map,
    actors: Vec<Box<dyn Actor>>,
    last_step: f64,
}

impl GameState {
    pub fn new() -> Self {
        let player =
             Player { position: Vec2::new(0., 0.), velocity: Vec2::new(0., 0.), size: Vec2::new(30., 30.) };
        GameState {
            player,
            active_map: Map { solids: vec![
                Rect::new(0., 50., 100., 100.),
                Rect::new(0., 250., 1000., 100.),
                Rect::new(0., 400., 1000., 100.),
                Rect::new(0., 5000., 100000., 100.),
            ] },
            actors: vec![],
            last_step: 0.0,
        }
    }

    // center on the player, except if we're at the edge of a map
    // then lock the camera viewport edge to the edge of the map
    pub fn render_camera(&mut self) {
        let zoom = 0.002;
        set_camera(&Camera2D {
            target: self.player.position,
            zoom: vec2(zoom, zoom * screen_width() / screen_height()),
            ..Default::default()
        });
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
            self.actors.push(Box::new(Item { position: self.player.position.clone(), velocity: Vec2::new(0., -1.), size: Vec2::new(10., 10.) }));
        }
        self.player.velocity = self.player.velocity.clamp(Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY), Vec2::new(MAX_VELOCITY, MAX_VELOCITY));
    }

    pub fn render_map(&self) {
        for solid in &self.active_map.solids {
            draw_rectangle(solid.x, solid.y, solid.w, solid.h, BLUE);
        }
    }

    pub fn render(&mut self) {
        let time = get_time();
        let step_len = (time - self.last_step) as f32;
        self.last_step = time;

        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.active_map.solids);
        }
        self.player.step_physics(step_len, &self.active_map.solids);

        // begin rendering
        self.render_camera();
        self.render_map();
        self.input(step_len);
        self.player.render();
        for actor in &self.actors {
            actor.render();
        }
    }
}
