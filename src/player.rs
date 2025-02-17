use macroquad::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::TICK_LEN;

use super::action::PlayerAction;
use super::action::PlayerBody;
use super::action::PlayerState;
use super::timestamp;
use super::Actor;
use super::MapData;

const ACCEL_RATE: f64 = 700.0;
const DECEL_RATE: f64 = 800.0;
const MAX_VELOCITY: f32 = 500.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub username: String,
    pub next_action: PlayerAction,
    pub current_action: PlayerAction,
}

impl Player {
    pub fn new(id: String) -> Self {
        Self {
            id,
            position: Vec2::ZERO,
            velocity: Vec2::new(0., 0.),
            size: Vec2::new(52., 52.),
            username: "".to_string(),
            next_action: PlayerAction::default(),
            current_action: PlayerAction::default(),
        }
    }

    pub fn body(&self) -> PlayerBody {
        PlayerBody {
            id: self.id.clone(),
            position: self.position,
            velocity: self.velocity,
            size: self.size,
            action: None,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn step_action(&mut self) {
        if self.current_action.move_right {
            let velocity = self.velocity_mut();
            velocity.x += (ACCEL_RATE * TICK_LEN) as f32;
            if velocity.x < 0.0 {
                velocity.x += (DECEL_RATE * TICK_LEN) as f32;
            }
        } else if self.current_action.move_left {
            let velocity = self.velocity_mut();
            velocity.x -= (ACCEL_RATE * TICK_LEN) as f32;
            if velocity.x > 0.0 {
                velocity.x -= (DECEL_RATE * TICK_LEN) as f32;
            }
        } else if self.velocity_mut().x.abs() > 0.0 {
            let velocity = self.velocity_mut();
            velocity.x = velocity
                .move_towards(Vec2::ZERO, (DECEL_RATE * TICK_LEN) as f32)
                .x;
        }

        if self.current_action.downward_jump && self.velocity_mut().y == 0. {
            let position = self.position_mut();
            position.y += 2.0;
        } else if self.current_action.jump {
            let velocity = self.velocity_mut();
            // TODO: check if we're standing on a platform first
            velocity.y = -300.0;
        }

        // if is_key_pressed(KeyCode::Z) {
        //     // drop an item
        //     state.actors.push(Box::new(Item::new(
        //         "assets/stick.png",
        //         state.player.position.clone(),
        //         Vec2::new(0., -200.),
        //     )));
        // }
    }

    /// We execute the current action, which changes the velocities
    /// we step the physics, which moves the player position
    /// we set the next action to be the current action for the next frame
    pub fn tick(&mut self, map: &MapData) {
        self.step_action();
        self.step_physics(map);
        self.current_action = self.next_action.clone();
        self.next_action.jump = false;
        self.next_action.downward_jump = false;
        self.next_action.enter_portal = false;
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

    fn step_physics(&mut self, map: &MapData) {
        self.step_physics_default(map);
        self.velocity = self.velocity.clamp(
            Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY),
            Vec2::new(MAX_VELOCITY, MAX_VELOCITY),
        );
    }
}
