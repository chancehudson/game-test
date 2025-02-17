use once_cell::sync::Lazy;
use std::time::Instant;

pub mod action;
pub mod actor;
pub mod engine;
pub mod map;
pub mod mob;
pub mod player;

pub use mob::Mob;

pub use actor::Actor;
pub use map::MapData;

static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());

pub fn timestamp() -> f64 {
    time_since_instant(*START_INSTANT)
}

pub fn time_since_instant(i: Instant) -> f64 {
    let diff_duration = Instant::now().duration_since(i);
    let diff_millis = diff_duration.as_millis();
    (diff_millis as f64) / 1000.0
}

pub fn time_since(t: f64) -> f32 {
    (timestamp() - t) as f32
}
