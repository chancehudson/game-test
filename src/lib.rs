use std::time::SystemTime;

use once_cell::sync::Lazy;

pub mod action;
pub mod actor;
pub mod map;
pub mod mob;

pub use mob::Mob;

pub use actor::Actor;
pub use map::MapData;

static START_TIMESTAMP_MS: Lazy<u128> = Lazy::new(|| {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
});

/// TODO: rework this whole thing
pub fn timestamp() -> f32 {
    let start_timestamp_ms = *START_TIMESTAMP_MS;
    let now_ms: u128 = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let diff = now_ms - start_timestamp_ms;
    // we assume diff is representable in an f64
    // convert to seconds
    (diff as f32) / 1000.0
}
