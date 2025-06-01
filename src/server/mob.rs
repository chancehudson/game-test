use std::time::Instant;
use std::vec::Vec;

use bevy::math::Rect;
use bevy::math::Vec2;
use game_test::action::PlayerBody;
use game_test::actor::GRAVITY_ACCEL;
use game_test::mob::MobAnimationData;
use game_test::MapData;

use rand::Rng;

use crate::TICK_RATE_MS;

use super::timestamp;
use game_test::actor::move_x;
use game_test::actor::move_y;
use game_test::mob::MobData;
use game_test::mob::MOB_DATA;

const KNOCKBACK_DURATION: f32 = 0.5;

#[derive(Debug, Clone)]
pub struct ServerMob {
    pub id: u64,
    pub mob_type: u64,
    pub position: Vec2,
    pub next_position: Vec2,
    pub velocity: Vec2,
    pub max_health: u64,
    pub health: u64,
    pub level: u64,

    pub moving_dir: Option<f32>,
    pub move_start: f32,
    pub knockback_began: Option<Instant>,
    pub aggro_to: Option<String>,
    pub aggro_began: Option<f32>,

    pub data: &'static MobAnimationData,
}

impl Into<MobData> for ServerMob {
    fn into(self) -> MobData {
        MobData {
            id: self.id,
            mob_type: self.mob_type,
            position: self.position,
            next_position: self.next_position,
            moving_dir: self.moving_dir,
            max_health: self.max_health,
            health: self.health,
            level: self.level,
        }
    }
}

impl ServerMob {
    pub fn new(id: u64, mob_type: u64) -> Self {
        Self {
            id,
            mob_type,
            position: Vec2::ZERO,
            next_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            moving_dir: None,
            move_start: 0.0,
            max_health: 10,
            health: 10,
            level: 1,
            knockback_began: None,
            aggro_to: None,
            aggro_began: None,
            data: MOB_DATA.get(&mob_type).as_ref().unwrap(),
        }
    }

    pub fn center(&self) -> Vec2 {
        self.position + self.data.size / 2.0
    }

    pub fn rect(&self) -> Rect {
        let data = MOB_DATA.get(&self.mob_type).unwrap();
        Rect::new(
            self.position.x,
            self.position.y,
            self.position.x + data.size.x,
            self.position.y + data.size.y,
        )
    }

    pub fn next_rect(&self) -> Rect {
        let data = MOB_DATA.get(&self.mob_type).unwrap();
        Rect::new(
            self.next_position.x,
            self.next_position.y,
            self.next_position.x + data.size.x,
            self.next_position.y + data.size.y,
        )
    }

    // when a mob is hit
    pub fn hit(&mut self, from: &PlayerBody, damage: u64) {
        if damage >= self.health {
            self.health = 0;
            // mob is dead, drop items and despawn
        } else {
            self.health -= damage;
            self.knockback_began = Some(Instant::now());
            self.aggro_to = Some(from.id.clone());
            self.aggro_began = Some(timestamp());
        }
    }

    pub fn tick(&mut self, map: &MapData) {
        self.position = self.next_position;
        // determine a new position based on the velocity of the mob
        if self.moving_dir.is_none() && rand::rng().random_bool(0.05) {
            self.move_start = timestamp();
            self.moving_dir = if rand::rng().random_bool(0.5) {
                Some(1.)
            } else {
                Some(-1.)
            }
        }
        if self.moving_dir.is_some() && rand::rng().random_bool(0.05) && self.aggro_to.is_none() {
            self.moving_dir = None;
        }
        if self.aggro_to.is_some() && timestamp() - self.aggro_began.unwrap() > 10.0 {
            self.aggro_to = None;
            self.moving_dir = None;
        }
        // we'll do a simple move algo on a single dimension (x)
        // in the future we want to look at the map and potentially jump onto platforms
        // to move vertically
        let moving_dir = self.moving_dir.unwrap_or(0.);
        self.velocity.x = moving_dir * self.data.max_velocity;

        const STEP_LEN_MS: f32 = 16.666667;
        let step_count: usize = (TICK_RATE_MS / STEP_LEN_MS).round() as usize;
        for _ in 0..step_count {
            self.step(STEP_LEN_MS / 1000., map);
        }
    }

    pub fn step(&mut self, step_len: f32, map: &MapData) {
        self.velocity.y += -GRAVITY_ACCEL * step_len;
        let (new_x, vel_x) = move_x(
            self.next_rect(),
            self.velocity,
            step_len * self.velocity.x,
            map,
        );
        let (new_y, vel_y) = move_y(
            self.next_rect(),
            self.velocity,
            step_len * self.velocity.y,
            map,
        );
        self.next_position = Vec2::new(new_x, new_y);
        self.velocity = Vec2::new(vel_x, vel_y);
    }
}
