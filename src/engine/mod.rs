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
use std::mem::discriminant;
use std::mem::Discriminant;

use serde::Deserialize;
use serde::Serialize;

pub mod entity;
pub mod mob;
pub mod mob_spawner;
pub mod platform;
pub mod player;

use entity::EngineEntity;
use entity::Entity;
use entity::EntityInput;
use player::PlayerEntity;

use crate::engine::platform::PlatformEntity;
use crate::generate_strong_u128;
use crate::map::MapData;
use crate::timestamp;

pub const STEP_LEN_S: f64 = 1. / 60.;
pub const STEP_LEN_S_F32: f32 = 1. / 60.;
pub const TRAILING_STATE_COUNT: u64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameEngine {
    pub start_timestamp: f64,
    map: MapData,
    // entity id keyed to struct
    pub entities: HashMap<u128, EngineEntity>,
    // step index keyed to entity id to struct
    #[serde(skip)]
    pub entities_by_step: HashMap<u64, HashMap<u128, EngineEntity>>,
    #[serde(skip)]
    pub new_entities_by_step: HashMap<u64, Vec<EngineEntity>>,
    // inputs by entity id, by step index
    pub inputs: HashMap<u128, BTreeMap<u64, EntityInput>>,
    pub step_index: u64,
    #[serde(skip)]
    grouped_entities: (u64, HashMap<Discriminant<EngineEntity>, Vec<EngineEntity>>),
}

impl GameEngine {
    pub fn new(map: MapData) -> Self {
        // spawn the platforms when we initilize the map
        let mut engine = Self {
            start_timestamp: timestamp(),
            map: map.clone(),
            step_index: 0,
            ..Default::default()
        };
        // spawn the map components as needed
        for platform in &map.platforms {
            let id = engine.generate_id();
            engine.spawn_entity(EngineEntity::Platform(PlatformEntity::new(
                id,
                platform.position.clone(),
                platform.size.clone(),
            )));
        }
        for spawn in &map.mob_spawns {
            let mut spawn_with_id = spawn.clone();
            spawn_with_id.id = engine.generate_id();
            engine.spawn_entity(EngineEntity::MobSpawner(spawn_with_id));
        }
        engine
    }

    /// generate a strong random id that isn't in use
    pub fn generate_id(&self) -> u128 {
        loop {
            let id = generate_strong_u128();
            if !self.entities.contains_key(&id) {
                return id;
            }
        }
    }

    pub fn engine_at_step(&self, step_index: &u64) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(step_index) {
            let mut out = self.clone();
            out.entities = entities.clone();
            out.entities_by_step = HashMap::new();
            out.new_entities_by_step = HashMap::new();
            out.step_index = *step_index;
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {step_index} is too far in the past")
        }
    }

    /// TODO: generic implementation
    /// e.g. spawn_entity::<PlayerEntity>(&mut self)
    pub fn spawn_player_entity(&mut self) -> EngineEntity {
        let id = self.generate_id();
        let player_entity = PlayerEntity::new(id);
        let engine_entity = EngineEntity::Player(player_entity);
        self.spawn_entity(engine_entity.clone());
        engine_entity
    }

    pub fn spawn_entity(&mut self, entity: EngineEntity) {
        if self.entities.contains_key(&entity.id()) {
            // TODO: decide how to handle this better?
            println!(
                "WARNING: attempting to insert entity with duplicate id, this is an error case"
            );
            panic!("encountered unrecoverable error");
        }
        if entity.id() == 0u128 {
            println!("WARNING: attempting to insert an entity with a common id! you may not be setting this value correctly");
        }
        let target_step_index = self.step_index + 1;
        if let Some(new_entities) = self.new_entities_by_step.get_mut(&target_step_index) {
            new_entities.push(entity);
        } else {
            self.new_entities_by_step
                .insert(target_step_index, vec![entity]);
        }
    }

    pub fn remove_entity(&mut self, entity_id: &u128) {
        self.entities.remove(entity_id);
    }

    /// Register a new input for an entity. Optionally provide a step index the
    /// input should be applied from.
    ///
    /// TODO: replay history from step_index if it's in the past.
    pub fn register_input(&mut self, step_index: Option<u64>, entity_id: u128, input: EntityInput) {
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

    /// Change an entities position in the world at a point in time
    /// This is an authoritative change, history will be rewritten from step_index
    /// TODO: disallow repositioning before latest input
    /// TODO: include an optional RespositionMode that is Replay | Overwrite ?
    pub fn reposition_entity(
        &mut self,
        new_entity: EngineEntity,
        step_index: &u64,
    ) -> anyhow::Result<()> {
        if step_index > &self.step_index {
            anyhow::bail!("WARNING: position change is in the future")
        } else if self.step_index - step_index >= TRAILING_STATE_COUNT {
            anyhow::bail!("WARNING: position change is too far in the past");
        }
        // take the entities at the previous step and reposition one
        if let Some(old_entities) = self.entities_by_step.get(step_index) {
            if !old_entities.contains_key(&new_entity.id()) {
                anyhow::bail!(
                    "entity {} does not exist at step {step_index}",
                    new_entity.id()
                )
            }
            // in each step we replay we need to check for new entities
            let replay_steps = self.step_index - step_index;
            self.entities = old_entities.clone();
            self.entities.insert(new_entity.id(), new_entity);
            self.step_index = *step_index;
            println!("WARNING: engine replaying {replay_steps} steps");
            for i in 0..replay_steps {
                let step_index = step_index + i;
                let new_entities = self
                    .new_entities_by_step
                    .get(&step_index)
                    .cloned()
                    .unwrap_or_else(|| vec![]);
                for entity in new_entities {
                    self.entities.insert(entity.id(), entity);
                }
                self.step();
            }
        } else {
            anyhow::bail!("entities do not exist for step {step_index}");
        }
        Ok(())
    }

    pub fn expected_step_index(&self) -> u64 {
        let now = timestamp();
        assert!(now >= self.start_timestamp, "GameEngine time ran backward");
        // rounds toward 0
        ((now - self.start_timestamp) / STEP_LEN_S) as u64
    }

    pub fn latest_input(&self, entity_id: &u128) -> Option<EntityInput> {
        let empty_inputs: BTreeMap<u64, EntityInput> = BTreeMap::new();
        let entity_input_map = self.inputs.get(entity_id).unwrap_or(&empty_inputs);
        entity_input_map
            .range(..=self.step_index)
            .next_back()
            .map(|(_step_index, input)| input)
            .cloned()
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

    /// Construct or retrieve entities by type for the current step
    pub fn grouped_entities(&mut self) -> &HashMap<Discriminant<EngineEntity>, Vec<EngineEntity>> {
        if self.grouped_entities.0 == self.step_index {
            return &self.grouped_entities.1;
        }
        let mut groups: HashMap<Discriminant<EngineEntity>, Vec<EngineEntity>> = HashMap::new();

        for entity in self.entities.clone() {
            let discriminant = discriminant(&entity.1);
            groups
                .entry(discriminant)
                .or_insert_with(Vec::new)
                .push(entity.1);
        }
        self.grouped_entities = (self.step_index, groups);
        &self.grouped_entities.1
    }

    pub fn step(&mut self) {
        let step_index = self.step_index;
        let spawned_entities = self
            .new_entities_by_step
            .get(&step_index)
            .cloned()
            .unwrap_or_default();
        for entity in spawned_entities {
            self.entities.insert(entity.id(), entity);
        }
        let entities_clone = self.entities.clone();
        let mut new_entities = HashMap::new();
        for (id, entity) in &entities_clone {
            let new_entity = entity.step(self, &step_index);
            new_entities.insert(*id, new_entity);
        }
        self.entities = new_entities;
        self.entities_by_step.insert(step_index, entities_clone);
        if step_index >= TRAILING_STATE_COUNT {
            self.entities_by_step
                .remove(&(step_index - TRAILING_STATE_COUNT));
        }
        self.step_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_step() {
        let map_data_str = std::fs::read_to_string("./assets/maps/eastwatch.map.json5").unwrap();
        let map_data = json5::from_str::<MapData>(&map_data_str).unwrap();
        let mut engine = GameEngine::new(map_data);
        engine.tick();
    }
}
