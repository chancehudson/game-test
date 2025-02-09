use macroquad::prelude::*;

trait Actor {
    fn move_x(&mut self, dx: f32, solids: &Vec<Rect>);
    fn move_y(&mut self, dx: f32, solids: &Vec<Rect>);
}

pub struct Player {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
}

impl Actor for Player {
    fn move_x(&mut self, dx: f32, solids: &Vec<Rect>) {
        let to_pos = self.position.x + dx;
        let sign = dx.signum();
        let dx_abs = dx.abs();
        let mut moved = 0.;

        self.position.x += dx;
    }

    fn move_y(&mut self, dy: f32, solids: &Vec<Rect>) {
        let to_pos = self.position.y + dy;
        let sign = dy.signum();
        let dy_abs = dy.abs();
        let mut moved = 0.;

        // if the character is jumping we don't care about collisions
        if dy.is_sign_negative() {
            self.position.y += dy;
            return;
        }
        while moved < dy_abs + 1. {
            let new_player_rect = Rect::new(self.position.x, self.position.y + sign * moved, self.size.x, self.size.y);
            for solid in solids {
                if let Some(overlap) = solid.intersect(new_player_rect) {
                    // only collide if we're at the top of the platform
                    if overlap.h < 1. && overlap.y == solid.y {
                        // we've collided, stop
                        self.velocity.y = 0.0;
                        self.position.y = new_player_rect.y - sign;
                        return;
                    }
                }
            }
            moved += 1.;
        }
        self.position.y += dy;
    }
}

/// We'll separate solids and visuals
pub struct Map {
    solids: Vec<Rect>,
}

pub struct GameState {
    player: Player,
    active_map: Map,
    last_step: f64,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player: Player { position: Vec2::new(0., 0.), velocity: Vec2::new(0., 0.), size: Vec2::new(30., 30.) },
            active_map: Map { solids: vec![
                Rect::new(0., 50., 100., 100.),
                Rect::new(0., 250., 1000., 100.),
                Rect::new(0., 400., 1000., 100.),
                Rect::new(0., 5000., 100000., 100.),
            ] },
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

    pub fn render_player(&mut self) {
        draw_circle(self.player.position.x + self.player.size.x / 2., self.player.position.y + self.player.size.y /2., self.player.size.x/2., GREEN);
    }

    pub fn input(&mut self, step_len: f32) {
        let accel_rate = 9.0 * step_len;
        let decel_rate = 13.0 * step_len;
        if is_key_down(KeyCode::Right) {
            self.player.velocity.x += accel_rate;
        } else if is_key_down(KeyCode::Left) {
            self.player.velocity.x -= accel_rate;
        } else if self.player.velocity.x.abs() > 0.0 {
            self.player.velocity.x = self.player.velocity.move_towards(Vec2::ZERO, decel_rate).x;
        }

        if is_key_pressed(KeyCode::Space) {
            // check if we're standing on a platform first
            self.player.velocity.y = -6.0;
        }
        self.player.velocity = self.player.velocity.clamp(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0));
    }

    pub fn step_physics(&mut self, step_len: f32) {
        // in meters per second per second
        const GRAVITY_ACCEL: f32 = 5.0;
        self.player.velocity.y += GRAVITY_ACCEL * step_len;
        self.player.move_x(self.player.velocity.x, &self.active_map.solids);
        self.player.move_y(self.player.velocity.y, &self.active_map.solids);
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

        self.step_physics(step_len);
        self.render_camera();
        self.render_map();
        self.render_player();
        self.input(step_len);
    }
}
