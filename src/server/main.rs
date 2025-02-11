use std::thread::spawn;
use std::time::Duration;
use std::time::SystemTime;

use once_cell::sync::Lazy;
use global_state::GlobalState;
use tungstenite::accept;

pub use game_test::MapData;
pub use game_test::Actor;

mod item;
mod player;
mod network;
mod global_state;
mod map_instance;

pub use player::Player;

static START_TIMESTAMP_MS: Lazy<u128> = Lazy::new(|| SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

/// TODO: rework this whole thing
pub fn timestamp() -> f32 {
    let now_ms: u128 = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let diff = now_ms - *START_TIMESTAMP_MS;
    // we assume diff is representable in an f64
    // convert to seconds
    (diff as f32) / 1000.0
}

// ticks per second
const TARGET_TICK_RATE: u32 = 30u32;

/// A WebSocket echo server
#[tokio::main]
async fn main () -> anyhow::Result<()> {
    let server = network::Server::new().await?;
    let mut state = GlobalState::new().await?;
    state.player_join("welcome");
    loop {
        state.step();
        state.next_tick().await;
    }
    Ok(())
}
