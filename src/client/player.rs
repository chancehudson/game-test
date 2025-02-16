use macroquad::prelude::*;

use game_test::action::PlayerAction;

use super::Actor;
use super::AnimatedEntity;
use super::MapData;
use super::Renderable;

const MAX_VELOCITY: f32 = 500.0;

pub struct Player {
    pub id: String,
    pub texture: AnimatedEntity,
    pub experience: u64,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub action: Option<PlayerAction>,
    pub username: String,
}

impl Player {
    pub fn new(id: String) -> Self {
        Self {
            id,
            experience: 0,
            texture: AnimatedEntity::new("assets/banana.png", 52.0, 52.0, 2),
            position: Vec2::new(100., 100.),
            velocity: Vec2::new(0., 0.),
            size: Vec2::new(52., 52.),
            action: None,
            username: "".to_string(),
        }
    }
}

impl Renderable for Player {
    fn render(&mut self, _step_len: f32) {
        if let Some(action) = self.action.as_ref() {
            if action.move_right {
                self.texture.flip_x = false;
            } else if action.move_left {
                self.texture.flip_x = true;
            }
        }
        self.texture.position = self.position;
        self.texture.set_animation(0); // Set to first animation (e.g., idle)
        self.texture.update(); // Update animation frame
        self.texture.draw(); // Draw current frame
                             // draw_circle(self.position.x + self.size.x / 2., self.position.y + self.size.y /2., self.size.x/2., GREEN);
        {
            let username_font_size = 15;
            let username_size = measure_text(&self.username, None, username_font_size, 1.0);
            let padding = 10.;
            let x = self.position.x + self.size.x / 2. - username_size.width / 2.;
            let y = self.position.y + self.size.y + padding;
            draw_rectangle(
                x - padding / 2.,
                y - padding / 2.,
                username_size.width + padding,
                username_size.height + padding,
                BLACK,
            );
            draw_text(
                &self.username,
                x,
                y + username_size.height,
                username_font_size.into(),
                WHITE,
            );
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
        if let Some(action) = self.action.clone() {
            self.action = Some(action.step_action(self, step_len));
        }
        self.velocity = self.velocity.clamp(
            Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY),
            Vec2::new(MAX_VELOCITY, MAX_VELOCITY),
        );
    }
}
