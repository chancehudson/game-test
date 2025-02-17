use std::collections::HashMap;

use macroquad::prelude::Vec2;
use serde::Deserialize;
use serde::Serialize;

use super::player::Player;
use super::Mob;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    // provide a username
    CreatePlayer(String),
    // provide a username
    LoginPlayer(String),
    PlayerMoved(u64, Vec2), // tick, new position
    Ping,
    TimeSync(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    PlayerLoggedIn(PlayerState),
    // tick, mobs, players
    MapState(u64, Vec<Mob>, HashMap<String, Player>),
    MobChange(u64, u64, Option<Vec2>), // tick, id, new moving_to
    PlayerRemoved(String),
    PlayerChange(u64, PlayerBody),
    LoginError(String),
    Tick(),
    Log(String),
    Pong,
    TimeSync(u32, u64, f64), // tick, milliseconds since tick
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
    pub tick: u64,
    pub move_left: bool,
    pub move_right: bool,
    pub enter_portal: bool,
    pub jump: bool,
    pub pickup: bool, // attempt to pick up an item
    pub downward_jump: bool,
}

impl Default for PlayerAction {
    fn default() -> Self {
        Self {
            tick: 0,
            move_left: false,
            move_right: false,
            enter_portal: false,
            jump: false,
            pickup: false,
            downward_jump: false,
        }
    }
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

impl PlayerAction {
    pub fn update(&mut self, other_new: &Self) {
        self.tick = other_new.tick;
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
}
