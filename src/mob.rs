use std::collections::HashMap;

use bevy_math::Vec2;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use walkdir::WalkDir;

use super::AnimationData;

/// Key the mob type to the data
pub static MOB_DATA: Lazy<HashMap<u64, MobAnimationData>> = Lazy::new(|| {
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
                let data = json5::from_str::<MobAnimationData>(&data_str).unwrap();
                mob_data.insert(data.id, data);
            }
        }
    }
    mob_data
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobAnimationData {
    pub id: u64,
    pub name: String,
    pub size: Vec2,
    pub max_velocity: f32,
    pub standing: AnimationData,
    pub walking: AnimationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobData {
    pub id: u64,
    pub mob_type: u64,
    pub position: Vec2,
    // the position we're moving to at the end of the next tick
    // if position == next_position the mob isn't moving in this tick
    pub next_position: Vec2,
    pub moving_dir: Option<f32>,
    pub max_health: u64,
    pub health: u64,
    pub level: u64,
}
