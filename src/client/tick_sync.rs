use std::collections::HashMap;

use game_test::action::Action;
use game_test::engine::TICK_LEN;
use game_test::timestamp;

use crate::network::Connection;

pub struct TickSync {
    pub id_counter: u32,
    pub latency: f64,
    pub req_map: HashMap<u32, f64>,
    pub last_ping_timestamp: f64,
    pub last_known_tick: u64,
    pub last_known_tick_timestamp: f64,
}

impl TickSync {
    pub fn new() -> Self {
        Self {
            last_known_tick: 0,
            last_known_tick_timestamp: 0.0,
            req_map: HashMap::new(),
            id_counter: 1,
            latency: 0.0,
            last_ping_timestamp: 0.0,
        }
    }

    pub fn current_tick(&self) -> u64 {
        self.last_known_tick
            + ((timestamp() - self.last_known_tick_timestamp) / TICK_LEN).floor() as u64
    }

    pub fn step(&mut self, connection: &mut Connection) {
        if timestamp() - self.last_ping_timestamp < 1.0 {
            return;
        }
        self.last_ping_timestamp = timestamp();
        let id = self.id_counter;
        self.id_counter += 1;
        self.req_map.insert(id, timestamp());
        if let Err(e) = connection.send(&Action::TimeSync(id)) {
            println!("Error sending time sync: {:?}", e);
        }
    }

    pub fn tick_response(&mut self, id: u32, tick: u64, since_last_tick: f64) {
        if let Some(req_time) = self.req_map.remove(&id) {
            self.latency = (timestamp() - req_time) / 2.0;
            let expected_diff = ((self.latency + since_last_tick) / TICK_LEN).round() as u64;
            self.last_known_tick = tick + expected_diff;
            self.last_known_tick_timestamp = timestamp() - self.latency;
            println!(
                "raw tick: {tick} latency: {} expected tick: {}",
                self.latency, self.last_known_tick
            );
        }
    }
}
