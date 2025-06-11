use std::collections::HashMap;

#[cfg(feature = "client")]
use bevy::image::TextureAtlasLayout;
use bevy_math::UVec2;
use bevy_math::Vec2;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use super::AnimationData;

const MOB_MANIFEST_STR: &'static str = include_str!("../assets/mob_manifest.json5");

// sprite id keyed to data path
pub static SPRITE_MANIFEST: Lazy<HashMap<u64, String>> = Lazy::new(|| {
    let mut out = HashMap::new();
    for (id, d) in json5::from_str::<HashMap<String, String>>(MOB_MANIFEST_STR)
        .unwrap()
        .into_iter()
    {
        out.insert(u64::from_str_radix(&id, 10).unwrap(), d);
    }
    out
});

#[cfg(not(target_arch = "wasm32"))]
pub fn load_json5<T>(filepath: &str) -> T
where
    T: for<'de> Deserialize<'de>,
{
    let data_str = std::fs::read_to_string(filepath).unwrap();
    json5::from_str::<T>(&data_str).unwrap()
}

/// Key the mob type to the data
#[cfg(not(target_arch = "wasm32"))]
pub static SPRITE_DATA: Lazy<HashMap<u64, SpriteAnimationData>> = Lazy::new(|| {
    let mut mob_data = HashMap::new();
    for (mob_type, filepath) in SPRITE_MANIFEST.iter() {
        mob_data.insert(*mob_type, load_json5::<SpriteAnimationData>(&filepath));
    }
    mob_data
});

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpriteAnimationData {
    pub sprite_type: u64,
    pub name: String,
    pub size: Vec2,
    pub standing: AnimationData,
    pub walking: AnimationData,
}

#[cfg(feature = "client")]
impl SpriteAnimationData {
    pub fn sprite_sheets(&self) -> Vec<(String, TextureAtlasLayout)> {
        vec![
            (
                self.standing.sprite_sheet.clone(),
                TextureAtlasLayout::from_grid(
                    UVec2::new(self.standing.width as u32, self.size.y as u32),
                    self.standing.frame_count as u32,
                    1,
                    None,
                    None,
                ),
            ),
            (
                self.walking.sprite_sheet.clone(),
                TextureAtlasLayout::from_grid(
                    UVec2::new(self.walking.width as u32, self.size.y as u32),
                    self.walking.frame_count as u32,
                    1,
                    None,
                    None,
                ),
            ),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobData {
    pub id: u64,
    pub sprite_type: u64,
    pub position: Vec2,
    // the position we're moving to at the end of the next tick
    // if position == next_position the mob isn't moving in this tick
    pub next_position: Vec2,
    pub moving_dir: Option<f32>,
    pub max_health: u64,
    pub health: u64,
    pub level: u64,
}
