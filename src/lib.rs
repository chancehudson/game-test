use std::time::Instant;

use once_cell::sync::Lazy;
use rand::rng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

pub mod action;
pub mod actor;
pub mod engine;
pub mod map;
pub mod mob;

pub use mob::MobData;

pub use actor::Actor;
pub use map::MapData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub frame_count: usize,
    pub fps: usize,
    pub sprite_sheet: String,
    pub width: usize,
}

// how many steps each client is behind the server
pub static STEP_DELAY: u64 = 30;

pub static TICK_RATE_MS: f64 = 200.;
pub static TICK_RATE_S_F32: f32 = (TICK_RATE_MS as f32) / 1000.;
pub static TICK_RATE_S: f64 = TICK_RATE_MS / 1000.;
pub static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());

pub fn timestamp() -> f64 {
    Instant::now().duration_since(*START_INSTANT).as_secs_f64()
}

pub fn generate_strong_u128() -> u128 {
    rng().random()
}
