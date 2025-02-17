use macroquad::prelude::*;

use game_test::player::Player;

use super::Actor;
use super::AnimatedEntity;
use super::Renderable;

const MAX_VELOCITY: f32 = 500.0;

pub struct PlayerRenderable {
    pub texture: AnimatedEntity,
}

impl PlayerRenderable {
    pub fn new() -> Self {
        Self {
            texture: AnimatedEntity::new("assets/banana.png", 52.0, 52.0, 2),
        }
    }

    pub fn render(&mut self, player: &Player) {
        if player.current_action.move_right {
            self.texture.flip_x = false;
        } else if player.current_action.move_left {
            self.texture.flip_x = true;
        }
        self.texture.position = player.position();
        self.texture.set_animation(0); // Set to first animation (e.g., idle)
        self.texture.update(); // Update animation frame
        self.texture.draw(); // Draw current frame
                             // draw_circle(self.position.x + self.size.x / 2., self.position.y + self.size.y /2., self.size.x/2., GREEN);
        {
            let username_font_size = 15;
            let username_size = measure_text(&player.username, None, username_font_size, 1.0);
            let padding = 10.;
            let x = player.position().x + player.size.x / 2. - username_size.width / 2.;
            let y = player.position().y + player.size.y + padding;
            draw_rectangle(
                x - padding / 2.,
                y - padding / 2.,
                username_size.width + padding,
                username_size.height + padding,
                BLACK,
            );
            draw_text(
                &player.username,
                x,
                y + username_size.height,
                username_font_size.into(),
                WHITE,
            );
        }
    }
}
