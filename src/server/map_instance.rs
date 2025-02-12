use std::collections::HashMap;

use super::Actor;
use super::MapData;
use super::Player;

pub struct MapInstance {
    pub map: MapData,
    pub player_ids: Vec<String>,
    pub actors: Vec<Box<dyn Actor>>,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            map,
            player_ids: vec![],
            actors: vec![],
        }
    }

    pub fn step(&mut self, players: &mut HashMap<String, Player>, step_len: f32) {
        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.map);
        }
        for id in &mut self.player_ids {
            if let Some(player) = players.get_mut(id) {
                player.step_physics(step_len, &self.map);
            }
        }
    }
}
