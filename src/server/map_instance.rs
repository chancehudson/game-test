use std::collections::HashMap;

use super::Actor;
use super::MapData;
use super::Player;

pub struct MapInstance {
    pub map: MapData,
    // pub player_ids: HashMap<String, ()>,
    pub actors: Vec<Box<dyn Actor + Sync + Send>>,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            map,
            // player_ids: HashMap::new(),
            actors: vec![],
        }
    }

    pub fn step(&mut self, _players: &mut HashMap<String, Player>, step_len: f32) {
        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.map);
        }
    }
}
