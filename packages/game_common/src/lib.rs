use serde::Deserialize;
use serde::Serialize;

pub mod data;
pub mod engine;
pub mod network;

pub use engine::*;

pub use data::item::ItemData;
pub use data::map::MapData;
pub use data::mob::MobData;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationData {
    pub frame_count: usize,
    pub fps: usize,
    pub sprite_sheet: String,
    pub width: usize,
    pub height: usize,
}

impl AnimationData {
    pub fn static_data(sprite_sheet: &str, size: bevy_math::UVec2) -> Self {
        Self {
            frame_count: 1,
            fps: 1,
            sprite_sheet: sprite_sheet.to_string(),
            width: size.x as usize,
            height: size.y as usize,
        }
    }

    pub fn is_static(&self) -> bool {
        self.frame_count == 1
    }
}

// how many steps each client is behind the server
pub static STEP_DELAY: u64 = 40;

// Custom deserializer for Vec2
pub fn deserialize_vec2<'de, D>(deserializer: D) -> Result<bevy_math::IVec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [i32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(bevy_math::IVec2::new(arr[0], arr[1]))
}
