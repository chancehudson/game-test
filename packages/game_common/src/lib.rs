use serde::Deserialize;
use serde::Serialize;

pub mod action;
pub mod map;
pub mod mob;

pub use map::MapData;
pub use mob::MobData;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationData {
    pub frame_count: usize,
    pub fps: usize,
    pub sprite_sheet: String,
    pub width: usize,
}

// how many steps each client is behind the server
pub static STEP_DELAY: u64 = 40;
