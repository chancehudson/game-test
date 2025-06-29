use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use web_time::Instant;

pub mod action;
pub mod actor;
pub mod db;
pub mod engine;
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

pub static TICK_RATE_MS: f64 = 50.;
pub static TICK_RATE_S_F32: f32 = (TICK_RATE_MS as f32) / 1000.;
pub static TICK_RATE_S: f64 = TICK_RATE_MS / 1000.;
pub static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());

pub fn timestamp() -> f64 {
    Instant::now().duration_since(*START_INSTANT).as_secs_f64()
}
