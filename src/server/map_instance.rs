use std::time::SystemTime;

use super::timestamp;
use super::MapData;
use super::Player;
use super::Actor;

pub struct MapInstance {
    pub map: MapData,
    pub players: Vec<Player>,
    pub actors: Vec<Box<dyn Actor>>,
    pub last_step: f32,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            map,
            players: vec![],
            actors: vec![],
            last_step: timestamp(),
        }
    }

    pub fn step(&mut self) {
        let time = timestamp();
        let step_len = time - self.last_step;
        self.last_step = time;

        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.map);
        }
        for player in &mut self.players {
            player.step_physics(step_len, &self.map);
        }
    }
}
