use macroquad::prelude::*;

use super::GameState;
use super::Player;
use super::Item;

const ACCEL_RATE: f32 = 700.0;
const DECEL_RATE: f32 = 800.0;

pub struct InputHandler {}

impl InputHandler {
    pub fn step(step_len: f32, state: &mut GameState) {
        if is_key_down(KeyCode::Right) {
            state.player.velocity.x += ACCEL_RATE * step_len;
            if state.player.velocity.x < 0.0 {
                state.player.velocity.x += DECEL_RATE * step_len;
            }
        } else if is_key_down(KeyCode::Left) {
            state.player.velocity.x -= ACCEL_RATE * step_len;
            if state.player.velocity.x > 0.0 {
                state.player.velocity.x -= DECEL_RATE * step_len;
            }
        } else if state.player.velocity.x.abs() > 0.0 {
            state.player.velocity.x = state.player.velocity.move_towards(Vec2::ZERO, DECEL_RATE * step_len).x;
        }

        if is_key_down(KeyCode::Down) && is_key_pressed(KeyCode::Space) && state.player.velocity.y == 0. {
            state.player.position.y += 2.0;
        } else if is_key_pressed(KeyCode::Space) {
            // TODO: check if we're standing on a platform first
            state.player.velocity.y = -300.0;
        }

        if is_key_pressed(KeyCode::Z) {
            // drop an item
            state.actors.push(
                Box::new(
                    Item::new("assets/stick.png", state.player.position.clone(), Vec2::new(0., -200.))
                )
            );
        }
    }
}
