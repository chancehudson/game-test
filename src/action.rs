use bevy::math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use super::actor::MAX_VELOCITY;
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
    LogoutPlayer,
    // action and a current position + velocity
    SetPlayerAction(PlayerAction, Vec2, Vec2),
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // current_map_id, experience
    PlayerLoggedIn(PlayerState, PlayerBody),
    MapState(Vec<Mob>),
    MobChange(u64, Option<Vec2>), // id, new moving_to
    PlayerRemoved(String),
    PlayerChange(PlayerBody, Option<PlayerState>),
    PlayerData(PlayerState, PlayerBody), // data about another player
    ChangeMap(String),
    LoginError(String),
    Tick(),
    Log(String),
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub experience: u64,
    pub max_health: u64,
    pub health: u64,
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
    pub attack: bool,
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
            attack: false,
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
        self.move_left = other_new.move_left;
        self.move_right = other_new.move_right;
        if other_new.attack {
            self.attack = true;
        }
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

    pub fn step_action_raw(
        &self,
        position: Vec2,
        velocity: Vec2,
        step_len: f32,
    ) -> (Vec2, Vec2, Self) {
        let mut updated_action = self.clone();
        let mut velocity = velocity;
        let mut position = position;
        if self.move_right {
            velocity.x += ACCEL_RATE * step_len;
            if velocity.x < 0.0 {
                velocity.x += DECEL_RATE * step_len;
            }
        } else if self.move_left {
            velocity.x -= ACCEL_RATE * step_len;
            if velocity.x > 0.0 {
                velocity.x -= DECEL_RATE * step_len;
            }
        } else if velocity.x.abs() > 0.0 {
            velocity.x = velocity.move_towards(Vec2::ZERO, DECEL_RATE * step_len).x;
        }

        if self.downward_jump && velocity.y == 0. {
            updated_action.downward_jump = false;
            position.y -= 2.0;
        } else if self.jump {
            updated_action.jump = false;
            // TODO: check if we're standing on a platform first
            velocity.y = 400.0;
        }
        (
            position,
            velocity.clamp(-MAX_VELOCITY, MAX_VELOCITY),
            updated_action,
        )

        // if is_key_pressed(KeyCode::Z) {
        //     // drop an item
        //     state.actors.push(Box::new(Item::new(
        //         "assets/stick.png",
        //         state.player.position.clone(),
        //         Vec2::new(0., -200.),
        //     )));
        // }
    }

    pub fn step_action(&self, actor: &mut dyn Actor, step_len: f32) -> Self {
        let (position, velocity, out) = self.step_action_raw(
            actor.position_mut().clone(),
            actor.velocity_mut().clone(),
            step_len,
        );
        *actor.position_mut() = position;
        *actor.velocity_mut() = velocity;
        out
    }
}
