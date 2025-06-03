/// A game engine instance for a single map.
/// Handles the "physics" of players, mobs, items.
///
/// This engine must allow stepping forward and backward.
///
/// tick: a variable numbers of steps
/// step: the smallest unit of time in the engine
///
use std::collections::BTreeMap;
use std::collections::HashMap;

use bevy::math::Vec2;

pub mod entity;
pub mod mob;
pub mod player;

use entity::EngineEntity;
use entity::Entity;
use entity::EntityInput;
use player::PlayerEntity;

use crate::map::MapData;
use crate::timestamp;

pub const STEP_LEN_S: f64 = 1. / 60.;

pub struct GameEngine {
    pub entity_id_counter: u64,
    pub start_timestamp: f64,
    pub map: MapData,
    // entity id keyed to struct
    pub entities: HashMap<u64, EngineEntity>,
    // tick index keyed to entity id to struct
    pub entities_by_tick: HashMap<u64, HashMap<u64, EngineEntity>>,
    // inputs by entity id, by step index
    pub inputs: HashMap<u64, BTreeMap<u64, EntityInput>>,
    // pub players: HashMap<String, Player>,
    // pub mobs: Vec<Mob>,
    pub step_index: u64,
}

impl GameEngine {
    pub fn new(map: MapData) -> Self {
        Self {
            entity_id_counter: 0,
            start_timestamp: timestamp(),
            map,
            entities: HashMap::new(),
            entities_by_tick: HashMap::new(),
            inputs: HashMap::new(),
            step_index: 0,
        }
    }

    pub fn insert_player(&mut self) -> &EngineEntity {
        self.entity_id_counter += 1;
        let id = self.entity_id_counter;
        let player_entity = PlayerEntity {
            id,
            position: Vec2::new(100., 100.),
            size: Vec2::new(40., 40.),
            player_id: "asfjashf".to_string(),
        };
        self.entities
            .insert(id, EngineEntity::Player(player_entity));
        self.entities.get(&id).unwrap()
    }

    /// Register a new input for an entity. Optionally provide a step index the
    /// input should be applied from.
    ///
    /// TODO: replay history from step_index if it's in the past.
    pub fn register_input(&mut self, step_index: Option<u64>, entity_id: u64, input: EntityInput) {
        let step_index = step_index.unwrap_or(self.step_index);
        let entity_inputs = self.inputs.entry(entity_id).or_insert(BTreeMap::new());
        let (latest_step_index, latest_input) = entity_inputs
            .last_key_value()
            .map(|(k, v)| (k, Some(v)))
            .unwrap_or((&0, None));
        if latest_step_index > &step_index {
            println!("WARNING: attemping to provide input before the latest input");
        }
        if entity_inputs.contains_key(&step_index) {
            println!("WARNING: overwriting existing input for step {step_index}")
        }
        // don't store duplicate input
        if let Some(latest_input) = latest_input {
            if latest_input == &input {
                return;
            }
        }
        entity_inputs.insert(step_index, input);
    }

    pub fn expected_step_index(&self) -> u64 {
        let now = timestamp();
        assert!(now >= self.start_timestamp, "GameEngine time ran backward");
        // rounds toward 0
        ((now - self.start_timestamp) / STEP_LEN_S) as u64
    }

    /// Automatically step forward in time as much as needed
    pub fn tick(&mut self) {
        let expected = self.expected_step_index();
        if expected <= self.step_index {
            println!("noop tick: your tick rate is too high!");
            return;
        }
        let step_count = expected - self.step_index;
        for _ in 0..step_count {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let empty_inputs: BTreeMap<u64, EntityInput> = BTreeMap::new();
        for entity in &mut self.entities.values_mut() {
            let entity_input_map = self.inputs.get(&entity.id()).unwrap_or(&empty_inputs);
            // get the most recent inputs, if they exist
            let input = entity_input_map.last_key_value().map(|(_, val)| val);
            let new_entity = entity.step(input, &self.map);
            *entity = new_entity;
        }
        self.step_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_step() {
        let map_data_str = std::fs::read_to_string("./assets/maps/eastwatch.json5").unwrap();
        let map_data = json5::from_str::<MapData>(&map_data_str).unwrap();
        let mut engine = GameEngine::new(map_data);
        engine.tick();
    }
}
