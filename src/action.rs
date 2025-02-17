use macroquad::prelude::Vec2;
use serde::Deserialize;
use serde::Serialize;

use super::Mob;
use crate::Actor;

const ACCEL_RATE: f32 = 700.0;
const DECEL_RATE: f32 = 800.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // provide a username
    CreatePlayer(String),
    // provide a username
    LoginPlayer(String),
    SetPlayerAction(PlayerAction),
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // current_map_id, experience
    PlayerLoggedIn(PlayerState),
    MapState(Vec<Mob>),
    MobChange(u64, Option<Vec2>), // id, new moving_to
    PlayerRemoved(String),
    PlayerChange(PlayerBody),
    PlayerData(PlayerState), // data about another player
    ChangeMap(String),
    LoginError(String),
    Tick(),
    Log(String),
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub experience: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBody {
    pub id: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub action: Option<PlayerAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub position: Option<Vec2>,
    pub velocity: Option<Vec2>,
    pub move_left: bool,
    pub move_right: bool,
    pub enter_portal: bool,
    pub jump: bool,
    pub pickup: bool, // attempt to pick up an item
    pub downward_jump: bool,
}

impl PartialEq for PlayerAction {
    fn eq(&self, other: &Self) -> bool {
        self.move_left == other.move_left
            && self.move_right == other.move_right
            && self.enter_portal == other.enter_portal
            && self.jump == other.jump
            && self.pickup == other.pickup
            && self.downward_jump == other.downward_jump
    }
}

impl Default for PlayerAction {
    fn default() -> Self {
        Self {
            velocity: None,
            position: None,
            move_left: false,
            move_right: false,
            enter_portal: false,
            jump: false,
            pickup: false,
            downward_jump: false,
        }
    }
}

impl PlayerAction {
    pub fn update(&mut self, other_new: Self) {
        self.velocity = other_new.velocity;
        self.position = other_new.position;
        self.move_left = other_new.move_left;
        self.move_right = other_new.move_right;
        if other_new.enter_portal {
            self.enter_portal = true;
        }
        if other_new.jump {
            self.jump = true;
        }
        if other_new.downward_jump {
            self.downward_jump = true;
        }
        if other_new.pickup {
            self.pickup = true;
        }
    }

    pub fn step_action(&self, actor: &mut dyn Actor, step_len: f32) -> Self {
        let mut out = self.clone();
        out.position = None;
        out.velocity = None;
        if self.move_right {
            let velocity = actor.velocity_mut();
            velocity.x += ACCEL_RATE * step_len;
            if velocity.x < 0.0 {
                velocity.x += DECEL_RATE * step_len;
            }
        } else if self.move_left {
            let velocity = actor.velocity_mut();
            velocity.x -= ACCEL_RATE * step_len;
            if velocity.x > 0.0 {
                velocity.x -= DECEL_RATE * step_len;
            }
        } else if actor.velocity_mut().x.abs() > 0.0 {
            let velocity = actor.velocity_mut();
            velocity.x = velocity.move_towards(Vec2::ZERO, DECEL_RATE * step_len).x;
        }

        if self.downward_jump && actor.velocity_mut().y == 0. {
            out.downward_jump = false;
            let position = actor.position_mut();
            position.y += 2.0;
        } else if self.jump {
            out.jump = false;
            let velocity = actor.velocity_mut();
            // TODO: check if we're standing on a platform first
            velocity.y = -300.0;
        }
        out

        // if is_key_pressed(KeyCode::Z) {
        //     // drop an item
        //     state.actors.push(Box::new(Item::new(
        //         "assets/stick.png",
        //         state.player.position.clone(),
        //         Vec2::new(0., -200.),
        //     )));
        // }
    }
}
