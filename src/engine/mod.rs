/// A game engine instance for a single map.
/// Handles the "physics" of players, mobs, items.
///
/// This engine must allow stepping forward and backward.
///
/// tick: a variable numbers of steps
/// step: the smallest unit of time in the engine
///
/// anatomy of a step
///
/// step:
///   creation: entities scheduled for creation are created
///   modification: entities modify themselves and schedule entities for creation/removal
///   removal: entities pending removal are removed
///
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::mem::discriminant;
use std::mem::Discriminant;

use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

pub mod entity;
pub mod game_event;
pub mod image_tmp;
pub mod mob;
pub mod mob_spawner;
pub mod platform;
pub mod player;
pub mod portal;

use entity::EEntity;
use entity::EngineEntity;
use entity::EntityInput;
use entity::SEEntity;
use player::PlayerEntity;

use crate::engine::game_event::GameEvent;
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
    // the boolean indicates whether this _always_ happens at this step
    // e.g. when we rewind should we preserve this event?
    pub new_entities_by_step: HashMap<u64, Vec<(EngineEntity, bool)>>,
    pub removed_entities_by_step: HashMap<u64, Vec<(u128, bool)>>,
    // inputs by entity id, by step index
    pub inputs: HashMap<u128, BTreeMap<u64, EntityInput>>,
    pub step_index: u64,
    #[serde(skip)]
    grouped_entities: (u64, HashMap<Discriminant<EngineEntity>, Vec<EngineEntity>>),
    #[serde(skip)]
    // map changes, experience gained, anything that needs to be written to db?
    game_events: Vec<GameEvent>,
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
            engine.spawn_entity(
                EngineEntity::Platform(PlatformEntity::new(
                    id,
                    platform.position.clone(),
                    platform.size.clone(),
                )),
                None,
                true,
            );
        }
        for spawn in &map.mob_spawns {
            let mut spawn_with_id = spawn.clone();
            spawn_with_id.id = engine.generate_id();
            engine.spawn_entity(EngineEntity::MobSpawner(spawn_with_id), None, true);
        }
        for portal in &map.portals {
            let id = engine.generate_id();
            let mut portal_clone = portal.clone();
            if portal_clone.size.x == 0. {
                portal_clone.size = Vec2::new(60., 60.);
            }
            portal_clone.id = id;
            engine.spawn_entity(EngineEntity::Portal(portal_clone), None, true);
        }
        engine
    }

    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.game_events)
    }

    pub fn emit_event(&mut self, event: GameEvent) {
        self.game_events.push(event);
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

    pub fn engine_at_step(&self, target_step_index: &u64) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(target_step_index) {
            let mut out = Self::default();
            out.inputs = self.inputs.clone();
            out.start_timestamp = self.start_timestamp;
            out.map = self.map.clone();
            out.entities = entities.clone();

            // some entities spawn regardless, some are caused by game events
            // we need to differentiate
            out.new_entities_by_step = self.new_entities_by_step.clone();
            for step_index in *target_step_index..self.step_index + 1 {
                if let Some(new_entities) = out.new_entities_by_step.get_mut(&step_index) {
                    new_entities.retain(|(_entity, is_universal)| *is_universal);
                }
            }
            out.removed_entities_by_step = self.removed_entities_by_step.clone();
            for step_index in *target_step_index..self.step_index + 1 {
                if let Some(removed_entities) = out.removed_entities_by_step.get_mut(&step_index) {
                    removed_entities.retain(|(_entity, is_universal)| *is_universal);
                }
            }
            out.step_index = *target_step_index;
            out.game_events = self
                .game_events
                .iter()
                .filter(|event| match event {
                    GameEvent::PlayerEnterPortal {
                        step_index,
                        player_id: _,
                        entity_id: _,
                        from_map: _,
                        to_map: _,
                    } => step_index <= target_step_index,
                })
                .cloned()
                .collect::<Vec<_>>();
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {target_step_index} is too far in the past")
        }
    }

    /// TODO: generic implementation
    /// e.g. spawn_entity::<PlayerEntity>(&mut self)
    /// optionally provide a step index the player should spawn at (must be in the past)
    pub fn spawn_player_entity(
        &mut self,
        player_id: String,
        position_maybe: Option<Vec2>,
        step_index: Option<u64>,
    ) -> EngineEntity {
        let id = self.generate_id();
        let player_entity = PlayerEntity::new_with_ids(id, player_id);
        let mut engine_entity = EngineEntity::Player(player_entity);
        if let Some(position) = position_maybe {
            *engine_entity.position_mut() = position;
        } else {
            *engine_entity.position_mut() = self.map.spawn_location;
        }
        self.spawn_entity(engine_entity.clone(), step_index, true);
        engine_entity
    }

    pub fn spawn_entity(
        &mut self,
        entity: EngineEntity,
        step_index: Option<u64>,
        is_universal: bool,
    ) {
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
        let target_step_index = step_index.unwrap_or(self.step_index + 1);
        if target_step_index > self.step_index {
            self.new_entities_by_step
                .entry(target_step_index)
                .or_insert(vec![])
                .push((entity, is_universal));
        } else {
            // backdate the position without stepping
            self.entities.insert(entity.id(), entity.clone());
            for step_index in target_step_index..self.step_index {
                self.entities_by_step
                    .entry(step_index)
                    .or_default()
                    .insert(entity.id(), entity.clone());
            }
            self.new_entities_by_step
                .entry(target_step_index)
                .or_default()
                .push((entity.clone(), is_universal));
        }
    }

    pub fn remove_entity(&mut self, entity_id: &u128) {
        self.removed_entities_by_step
            .entry(self.step_index)
            .or_default()
            .push((*entity_id, true));
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
            println!(
                "WARNING: attemping to provide input before the latest input step {step_index} latest {latest_step_index}"
            );
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
                for (entity, _) in new_entities {
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

    pub fn entities_by_type(&mut self, discr: &Discriminant<EngineEntity>) -> &Vec<EngineEntity> {
        self.grouped_entities();
        self.grouped_entities
            .1
            .entry(discr.clone())
            .or_insert(vec![])
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

    /// A step is considered complete at the _end_ of this function
    pub fn step(&mut self) {
        let step_index = self.step_index;
        // remove entities at the end of the step
        if let Some(entity_ids_to_remove) = self.removed_entities_by_step.get(&step_index) {
            for (id, _) in entity_ids_to_remove {
                self.entities.remove(id);
            }
        }
        let entities_clone = self.entities.clone();
        let mut stepped_entities = HashMap::new();
        for (id, entity) in &entities_clone {
            let stepped = entity.step(self, &step_index);
            stepped_entities.insert(*id, stepped);
        }
        self.entities = stepped_entities;
        self.entities_by_step.insert(step_index, entities_clone);
        if step_index >= TRAILING_STATE_COUNT {
            self.entities_by_step
                .remove(&(step_index - TRAILING_STATE_COUNT));
            self.new_entities_by_step
                .remove(&(step_index - TRAILING_STATE_COUNT));
            self.removed_entities_by_step
                .remove(&(step_index - TRAILING_STATE_COUNT));
        }
        self.step_index += 1;
        // add new entities at the beginning of the step
        let new_entities = self
            .new_entities_by_step
            .get(&(self.step_index))
            .cloned()
            .unwrap_or_default();
        for (entity, _) in new_entities {
            if let Some(_) = self.entities.insert(entity.id(), entity) {
                println!("WARNING: inserting entity that already existed!");
            }
        }
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
