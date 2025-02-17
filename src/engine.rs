use std::collections::HashMap;

use macroquad::prelude::Vec2;

use super::player::Player;
use super::timestamp;
use super::Actor;
use super::MapData;
use super::Mob;

pub const TICK_LEN: f64 = 50.0 * 1.0 / 1000.0;
type Callback = Box<dyn Fn(&mut Engine) + Send + Sync>;

pub struct Engine {
    pub mob_counter: u64,
    pub start_time: f64,
    pub map: MapData,
    pub players: HashMap<String, Player>,
    pub mobs: Vec<Mob>,
    pub next_tick_actions: Vec<Callback>,
    pub current_tick: u64,
}

impl Engine {
    pub fn new(map: MapData, mob_counter: u64, current_tick: u64) -> Self {
        Self {
            next_tick_actions: vec![],
            mob_counter,
            start_time: timestamp(),
            map,
            players: HashMap::new(),
            mobs: vec![],
            current_tick,
        }
    }

    pub fn action(&mut self, cb: Callback) {
        self.next_tick_actions.push(cb);
    }

    // return a vec of mobs that changed
    pub fn tick(&mut self) {
        self.current_tick += 1;
        // spawn mobs as needed
        for spawn in &self.map.mob_spawns {
            if self.current_tick - spawn.last_spawn < 10 {
                continue;
            }
            if self.mobs.len() >= spawn.max_count {
                continue;
            }
            let spawn_count = rand::random_range(0..=spawn.max_count);
            for _ in 0..spawn_count {
                self.mob_counter += 1;
                self.mobs.push(Mob {
                    id: self.mob_counter,
                    mob_type: spawn.mob_type,
                    position: Vec2::new(
                        rand::random_range(spawn.position.x..spawn.position.x + spawn.size.x),
                        rand::random_range(spawn.position.y..spawn.position.y + spawn.size.y),
                    ),
                    velocity: Vec2::ZERO,
                    size: Vec2::new(50., 50.),
                    max_velocity: 200.,

                    moving_to: None,
                    move_start: 0.,
                });
            }
        }
        // step our mobs
        for mob in &mut self.mobs {
            mob.step_physics(&self.map);
        }
        // step the players
        // TODO: in parallel
        for player in self.players.values_mut() {
            player.tick(&self.map);
        }
        for cb in self.next_tick_actions.drain(..).collect::<Vec<Callback>>() {
            cb(self);
        }
    }
}
