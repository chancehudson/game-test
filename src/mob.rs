use std::collections::HashMap;

use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "server")]
use rand::Rng;
use walkdir::WalkDir;

#[cfg(feature = "server")]
use super::timestamp;
use super::Actor;
use super::MapData;

pub static MOB_DATA: Lazy<HashMap<u64, MobData>> = Lazy::new(|| {
    let mut mob_data = HashMap::new();
    for entry in WalkDir::new("mobs") {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if let Some(extension) = path.extension() {
            if extension != "json5" {
                continue;
            }
            if let Some(_file_name) = entry.file_name().to_str() {
                let data_str = std::fs::read_to_string(path_str).unwrap();
                let data = json5::from_str::<MobData>(&data_str).unwrap();
                mob_data.insert(data.id, data);
            }
        }
    }
    mob_data
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub frame_count: usize,
    pub fps: usize,
    pub sprite_sheet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobData {
    pub id: u64,
    pub name: String,
    pub size: Vec2,
    pub max_velocity: f32,
    pub standing: AnimationData,
    pub walking: AnimationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub id: u64,
    pub data: MobData,
    pub mob_type: u64,
    pub position: Vec2,
    pub velocity: Vec2,

    pub moving_to: Option<Vec2>,
    pub move_start: f32,
}

impl Mob {
    pub fn new(id: u64, mob_type: u64) -> Self {
        let data = MOB_DATA.get(&mob_type).unwrap().clone();
        Self {
            id,
            mob_type,
            data,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            moving_to: None,
            move_start: 0.0,
        }
    }
}

impl Actor for Mob {
    fn rect(&self) -> Rect {
        let data = MOB_DATA.get(&self.mob_type).unwrap();
        Rect::new(self.position.x, self.position.y, data.size.x, data.size.y)
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        let data = MOB_DATA.get(&self.mob_type).unwrap();
        // simple logic to control the mob
        let accel_rate = 700.0;
        if self.moving_to.is_none() {
            #[cfg(feature = "server")]
            if rand::rng().random_bool(0.001) {
                self.move_start = timestamp();
                self.moving_to = Some(Vec2::new(
                    rand::rng().random_range(0.0..map.size.x),
                    rand::rng().random_range(0.0..map.size.y),
                ));
            }
            self.velocity.x = self
                .velocity
                .move_towards(Vec2::ZERO, accel_rate * step_len)
                .x;
            self.step_physics_default(step_len, map);
            return;
        }
        let moving_to = self.moving_to.clone().unwrap();
        let move_left = self.position.x > moving_to.x;
        let move_right = self.position.x < moving_to.x;
        if move_right {
            self.velocity_mut().x += accel_rate * step_len;
            self.velocity.x = self.velocity.x.clamp(-data.max_velocity, data.max_velocity);
        } else if move_left {
            self.velocity_mut().x -= accel_rate * step_len;
            self.velocity.x = self.velocity.x.clamp(-data.max_velocity, data.max_velocity);
        } else if self.velocity.x.abs() > 0.0 {
            self.velocity.x = self
                .velocity
                .move_towards(Vec2::ZERO, accel_rate * step_len)
                .x;
        }
        if (self.position.x - moving_to.x).abs() < 10.0 {
            self.moving_to = None;
        }
        #[cfg(feature = "server")]
        if timestamp() - self.move_start > 10.0 {
            self.moving_to = None;
        }
        self.step_physics_default(step_len, map);
    }
}
