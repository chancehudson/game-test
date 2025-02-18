use macroquad::prelude::*;

use game_test::action::PlayerAction;
use game_test::actor::MAX_VELOCITY;

use crate::AssetBuffer;

use super::Actor;
use super::AnimatedEntity;
use super::MapData;
use super::Renderable;

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
            texture: AnimatedEntity::new("assets/banana.png", 2),
            position: Vec2::ZERO,
            velocity: Vec2::new(0., 0.),
            size: Vec2::new(52., 52.),
            action: None,
            username: "".to_string(),
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
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
        self.texture.position = self.position();
        self.texture.set_animation(0); // Set to first animation (e.g., idle)
        self.texture.update(); // Update animation frame
        self.texture.draw(); // Draw current frame
                             // draw_circle(self.position.x + self.size.x / 2., self.position.y + self.size.y /2., self.size.x/2., GREEN);
        {
            let username_font_size = 12;
            let username_size = measure_text(
                &self.username,
                AssetBuffer::font("helvetica_light"),
                username_font_size,
                1.0,
            );
            let padding = 10.;
            let x = self.position().x + self.size.x / 2. - username_size.width / 2.;
            let y = self.position().y + self.size.y + padding;
            let mut color = BLACK;
            color.a = 0.5;
            draw_rectangle(
                x - padding / 2.,
                y - padding / 2.,
                username_size.width + padding,
                username_size.height + padding,
                color,
            );
            draw_text_ex(
                &self.username,
                x,
                y + username_size.height,
                TextParams {
                    font: AssetBuffer::font("helvetica_light"),
                    font_size: username_font_size,
                    color: WHITE,
                    ..Default::default()
                },
            );
            // draw_text(
            //     &self.username,
            //     x,
            //     y + username_size.height,
            //     username_font_size.into(),
            //     WHITE,
            // );
        }
    }
}

impl Actor for Player {
    fn rect(&self) -> Rect {
        Rect::new(
            self.position().x,
            self.position().y,
            self.size.x,
            self.size.y,
        )
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
        self.velocity = self.velocity.clamp(-1.0 * MAX_VELOCITY, MAX_VELOCITY);
    }
}
