use std::time::Instant;

use once_cell::sync::Lazy;

pub mod action;
pub mod actor;
pub mod map;
pub mod mob;

pub use mob::Mob;

pub use actor::Actor;
pub use map::MapData;

pub static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());

pub fn timestamp() -> f32 {
    let diff_ms = Instant::now().duration_since(*START_INSTANT).as_millis();
    // cast into an f64 before dividing. An f32 can only store ~60 days of time in milliseconds.
    // It's unlikely a process stays up this long, but we can extend this 1000x by double
    // casting here
    ((diff_ms as f64) / 1000.0) as f32
}
