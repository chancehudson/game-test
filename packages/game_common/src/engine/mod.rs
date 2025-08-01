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
///   game events: game events are processed by the engine, and then by any external observers
///   modification: entities modify themselves and schedule entities for creation/removal
///   creation: entities scheduled for creation are created
///   removal: entities pending removal are removed
///
use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::LazyLock;

use bevy_math::IVec2;
use once_cell::sync::Lazy;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use serde::Serialize;
use web_time::Instant;

pub static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());
pub fn timestamp() -> f64 {
    Instant::now().duration_since(*START_INSTANT).as_secs_f64()
}

pub mod actor;
pub mod damage_calc;
pub mod entity;
pub mod game;
pub mod game_event;
pub mod system;

use entity::EEntity;
use entity::EngineEntity;
use entity::SEEntity;
use game_event::*;

use crate::data::GameData;
use crate::entity::message::MessageEntity;

pub const STEP_LEN_S: f64 = 1. / 60.;
pub const STEP_LEN_S_F32: f32 = 1. / 60.;
pub const STEPS_PER_SECOND: u64 = (1.0 / STEP_LEN_S_F32) as u64;
pub const STEPS_PER_SECOND_I32: i32 = (1.0 / STEP_LEN_S_F32) as i32;
pub const TRAILING_STATE_COUNT: u64 = 360;

// initializable for the map instance
// the npcs are moving platforms are intializable too
pub trait EngineInit {
    fn init(&self, game_data: &GameData, engine: &mut GameEngine) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEngine {
    pub id: u128,
    pub seed: u64,

    pub size: IVec2,

    pub step_index: u64,
    pub start_timestamp: f64,

    // entity id keyed to struct
    // #[serde(serialize_with = "serialize_unpure_entities")]
    entities: BTreeMap<u128, EngineEntity>,
    // step index keyed to entity id to struct
    entities_by_step: BTreeMap<u64, BTreeMap<u128, EngineEntity>>,

    // engine events may be scheduled for the future, game events may not
    engine_events_by_step: BTreeMap<u64, Vec<EngineEvent>>,
    #[serde(skip)]
    game_events_by_step: BTreeMap<u64, Vec<GameEvent>>,

    // for external use
    #[serde(skip)]
    grouped_entities: (u64, HashMap<TypeId, Vec<EngineEntity>>),
    #[serde(skip, default = "default_game_events")]
    pub game_events: (flume::Sender<GameEvent>, flume::Receiver<GameEvent>),

    #[serde(skip, default = "default_rng")]
    rng: (u64, ChaCha8Rng),

    #[serde(skip)]
    pub enable_debug_markers: bool,
}

// default seeded rng for engine
fn default_rng() -> (u64, ChaCha8Rng) {
    (u64::MAX, ChaCha8Rng::seed_from_u64(0))
}

fn default_game_events() -> (flume::Sender<GameEvent>, flume::Receiver<GameEvent>) {
    flume::unbounded()
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new(IVec2 { x: 1000, y: 1000 })
    }
}

impl GameEngine {
    pub fn new(size: IVec2) -> Self {
        Self {
            id: rand::random(),
            start_timestamp: timestamp(),
            size,
            rng: default_rng(),
            game_events: default_game_events(),
            seed: 0,
            step_index: 0,
            entities: BTreeMap::new(),
            entities_by_step: BTreeMap::new(),
            engine_events_by_step: BTreeMap::new(),
            game_events_by_step: BTreeMap::new(),
            grouped_entities: (0, HashMap::new()),
            enable_debug_markers: false,
        }
    }

    pub fn step_hash(&self, step_index: &u64) -> anyhow::Result<blake3::Hash> {
        // hash of all entities
        if let Some(entities) = self.entities_by_step.get(step_index) {
            let serialized = bincode::serialize(&entities.iter().collect::<BTreeMap<_, _>>())?;
            Ok(blake3::hash(&serialized))
        } else {
            anyhow::bail!("error calculating engine.step_hash, {step_index} not known to engine");
        }
    }

    pub fn rng(&mut self) -> &mut ChaCha8Rng {
        if self.rng.0 != self.step_index {
            self.rng.0 = self.step_index;
            self.rng.1 = ChaCha8Rng::seed_from_u64(self.seed ^ self.step_index);
        }
        &mut self.rng.1
    }

    /// generate a seeded, strong random id that isn't in use
    pub fn generate_id(&mut self) -> u128 {
        loop {
            let rng = self.rng();
            let id = rng.random();
            if !self.entities.contains_key(&id) {
                return id;
            }
        }
    }

    pub fn game_events(&self, from_step: u64, to_step: u64) -> Vec<GameEvent> {
        self.game_events_by_step
            .range(from_step..to_step)
            .map(|(_, game_events)| game_events.clone())
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Retrieve a past instance of the engine that will be equal
    /// to self after N steps
    /// universal events occur independently on the engine state. e.g. a player logging on
    pub fn engine_at_step(
        &self,
        target_step_index: &u64,
        rewindable: bool,
    ) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(target_step_index) {
            let mut out = Self::default();
            out.id = self.id;

            // get all future events that are universal
            out.engine_events_by_step = self
                .engine_events_by_step
                .range(target_step_index..)
                .map(|(k, v)| {
                    (
                        *k,
                        v.iter()
                            .filter(|v| v.is_universal())
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>();

            if rewindable {
                out.engine_events_by_step.extend(
                    self.engine_events_by_step
                        .range(..target_step_index)
                        .into_iter()
                        .map(|(k, v)| (*k, v.clone())),
                );
                out.game_events_by_step = self
                    .game_events_by_step
                    .range(..target_step_index)
                    .map(|(k, v)| (*k, v.clone()))
                    .collect::<BTreeMap<_, _>>();
                out.entities_by_step = self
                    .entities_by_step
                    .range(..target_step_index)
                    .map(|(si, data)| (*si, data.clone()))
                    .collect::<BTreeMap<_, _>>();
            } else {
                out.game_events_by_step = BTreeMap::new();
                out.entities_by_step = BTreeMap::new();
            }
            out.start_timestamp = self.start_timestamp;
            out.size = self.size.clone();
            out.entities = entities.clone();
            out.enable_debug_markers = self.enable_debug_markers;

            out.step_index = *target_step_index;
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {target_step_index} is too far in the past")
        }
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
            println!(
                "WARNING: attempting to insert an entity with a common id! you may not be setting this value correctly"
            );
        }
        let target_step_index = step_index.unwrap_or(self.step_index);
        if target_step_index >= self.step_index {
            self.register_event(
                Some(target_step_index),
                EngineEvent::SpawnEntity {
                    entity,
                    universal: is_universal,
                },
            );
        } else {
            self.integrate_event(
                target_step_index,
                EngineEvent::SpawnEntity {
                    entity,
                    universal: is_universal,
                },
            );
        }
    }

    pub fn remove_entity(&mut self, entity_id: u128, universal: bool) {
        self.register_event(
            None,
            EngineEvent::RemoveEntity {
                entity_id,
                universal,
            },
        );
    }

    pub fn integrate_event(&mut self, step_index: u64, event: EngineEvent) {
        let mut btree: BTreeMap<u64, Vec<EngineEvent>> = BTreeMap::new();
        btree.entry(step_index).or_default().push(event);
        self.integrate_events(btree);
    }

    pub fn integrate_events(&mut self, events: BTreeMap<u64, Vec<EngineEvent>>) {
        if events.is_empty() {
            return;
        }
        let from_step_index = *events.first_key_value().unwrap().0 - 1;
        if from_step_index >= self.step_index {
            for (step_index, events) in events {
                for event in events {
                    self.register_event(Some(step_index), event);
                }
            }
        } else {
            println!(
                "at step: {} replaying {} steps",
                self.step_index,
                self.step_index - from_step_index
            );
            // we receive an event from the past, rewind and replay
            if let Ok(mut past_engine) = self.engine_at_step(&from_step_index, true) {
                past_engine.integrate_events(events);
                past_engine.step_to(&self.step_index);

                #[cfg(debug_assertions)]
                for (id, entity) in self
                    .entities_by_step
                    .entry(past_engine.step_index)
                    .or_default()
                {
                    if let Some(other_entity) = past_engine.entities.get(id) {
                        if entity != other_entity {
                            println!("ENTITIES MISMATCH step {}", past_engine.step_index);
                            println!(
                                "{} steps from start of replay",
                                past_engine.step_index - from_step_index
                            );
                            println!("{:?}", entity);
                            println!("{:?}", other_entity);
                        }
                    } else {
                        println!("ENTITY DOES NOT EXIST");
                    }
                }
                self.game_events_by_step = past_engine.game_events_by_step;
                self.entities = past_engine.entities;
                self.entities_by_step = past_engine.entities_by_step;
                self.engine_events_by_step = past_engine.engine_events_by_step;
                self.grouped_entities = (0, HashMap::new());
            } else {
                panic!("failed to generate past engine");
            }
        }
    }

    pub fn register_event(&mut self, step_index: Option<u64>, event: EngineEvent) {
        let step_index = step_index.unwrap_or(self.step_index);
        self.engine_events_by_step
            .entry(step_index)
            .or_default()
            .push(event)
    }

    pub fn entity_by_id_untyped(
        &self,
        id: &u128,
        step_index: Option<u64>,
    ) -> Option<&EngineEntity> {
        let step_index = step_index.unwrap_or(self.step_index);
        self.entities_at_step(step_index).get(id)
    }

    /// Return an entity by id at a certain step index, if possible. Extracts the underlying
    /// entity type from the EngineEntity
    pub fn entity_by_id<T: 'static>(&self, id: &u128, step_index: Option<u64>) -> Option<&T>
    where
        T: EEntity,
    {
        let step_index = step_index.unwrap_or(self.step_index);
        if let Some(engine_entity) = self.entities_at_step(step_index).get(id) {
            if let Some(e) = engine_entity.extract_ref::<T>() {
                return Some(e);
            } else {
                println!("WARNING: attempting to extract entity of mismatched type");
            }
        }
        None
    }

    pub fn entity_by_id_mut<T: 'static>(
        &mut self,
        id: &u128,
        step_index: Option<u64>,
    ) -> Option<&mut T>
    where
        T: EEntity,
    {
        let step_index = step_index.unwrap_or(self.step_index);
        let entities = if step_index == self.step_index {
            Some(&mut self.entities)
        } else {
            self.entities_by_step.get_mut(&step_index)
        };
        if let Some(entities) = entities {
            if let Some(engine_entity) = entities.get_mut(id) {
                if let Some(e) = engine_entity.extract_ref_mut::<T>() {
                    return Some(e);
                } else {
                    println!("WARNING: attempting to extract entity of mismatched type");
                }
            }
        }
        None
    }

    pub fn entities_at_step(&self, step_index: u64) -> &BTreeMap<u128, EngineEntity> {
        if step_index == self.step_index {
            &self.entities
        } else {
            match self.entities_by_step.get(&step_index) {
                Some(entities) => entities,
                None => {
                    #[cfg(debug_assertions)]
                    panic!(
                        "requested entities for an unknown step {step_index}, current step {}",
                        self.step_index
                    );
                    static EMPTY_ENTITIES: LazyLock<BTreeMap<u128, EngineEntity>> =
                        LazyLock::new(|| BTreeMap::new());
                    &EMPTY_ENTITIES
                }
            }
        }
    }

    pub fn entities_by_type<T>(&mut self) -> impl Iterator<Item = &T>
    where
        T: 'static,
    {
        self.grouped_entities();
        let type_id = TypeId::of::<T>();
        self.grouped_entities
            .1
            .entry(type_id)
            .or_default()
            .iter()
            .filter_map(|entity| entity.extract_ref::<T>())
    }

    /// Construct or retrieve entities by type for the current step
    pub fn grouped_entities(&mut self) -> &HashMap<TypeId, Vec<EngineEntity>> {
        if self.grouped_entities.0 == self.step_index {
            return &self.grouped_entities.1;
        }
        let mut groups: HashMap<TypeId, Vec<EngineEntity>> = HashMap::new();

        for entity in self.entities.clone() {
            let type_id = entity.1.type_id();
            groups.entry(type_id).or_default().push(entity.1);
        }
        self.grouped_entities = (self.step_index, groups);
        &self.grouped_entities.1
    }

    pub fn expected_step_index(&self) -> u64 {
        let now = timestamp();
        assert!(now >= self.start_timestamp, "GameEngine time ran backward");
        // rounds toward 0
        self.step_index
            .max(((now - self.start_timestamp) / STEP_LEN_S) as u64)
    }

    /// Automatically step forward in time as much as needed
    pub fn tick(&mut self) -> Vec<GameEvent> {
        let expected = self.expected_step_index();
        if expected <= self.step_index {
            println!("noop tick: your tick rate is too high!");
            return vec![];
        }
        self.step_to(&expected)
    }

    pub fn step_to(&mut self, target_step_index: &u64) -> Vec<GameEvent> {
        if target_step_index < &self.step_index {
            panic!("cannot step forward to a point in the past");
        }
        let mut out = vec![];
        while &self.step_index != target_step_index {
            out.append(&mut self.step());
        }
        out
    }

    /// A step is considered complete at the _end_ of this function
    pub fn step(&mut self) -> Vec<GameEvent> {
        let entities_clone = self.entities.clone();
        self.entities_by_step
            .insert(self.step_index, entities_clone);

        // Execute the modification phase of the step
        let mut stepped_entities = BTreeMap::new();
        for (id, entity) in &self.entities.clone() {
            let stepped = entity.step(self);
            stepped_entities.insert(*id, stepped);
        }
        self.entities = stepped_entities;

        // Execute the creation phase of the step
        for event in self
            .engine_events_by_step
            .entry(self.step_index)
            .or_default()
        {
            match event {
                EngineEvent::SpawnEntity {
                    entity,
                    universal: _,
                } => {
                    if let Some(e) = self.entities.insert(entity.id(), entity.clone()) {
                        println!("WARNING: inserting entity that already existed! {:?}", e);
                        if &e == entity {
                            println!("entities are equal");
                        }
                        println!("new: {:?}", entity);
                    }
                }
                EngineEvent::RemoveEntity {
                    entity_id,
                    universal: _,
                } => {
                    self.entities.remove(entity_id);
                }
                EngineEvent::Message {
                    text,
                    entity_id,
                    universal: _,
                } => {
                    if let Some(entity) = self.entities.get(entity_id) {
                        let is_player = match entity {
                            EngineEntity::Player(_) => true,
                            _ => false,
                        };
                        let msg_entity = MessageEntity::new_text(
                            text.clone(),
                            self.step_index,
                            entity.id(),
                            is_player,
                        );
                        self.entities
                            .insert(msg_entity.id(), EngineEntity::Message(msg_entity));
                    } else {
                        println!("WARNING: sending message from non-existent player entity")
                    }
                }
                _ => {}
            }
        }

        // Do some engine housekeeping
        if self.step_index >= TRAILING_STATE_COUNT {
            let step_to_remove = self.step_index - TRAILING_STATE_COUNT;
            self.entities_by_step.retain(|k, _v| k > &step_to_remove);
            self.engine_events_by_step
                .retain(|k, _v| k > &step_to_remove);
            self.game_events_by_step.retain(|k, _v| k > &step_to_remove);
        }

        let game_events = self.game_events.1.drain().collect::<Vec<_>>();
        self.game_events_by_step
            .insert(self.step_index, game_events.clone());

        for game_event in &game_events {
            game::default_handler(self, game_event);
        }

        // Officially move to the next step
        self.step_index += 1;

        game_events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    use crate::data::map::MobSpawnData;
    use crate::entity::mob_spawn::MobSpawnEntity;
    use crate::entity::platform::PlatformEntity;

    pub fn test_engine() -> GameEngine {
        let mut engine = GameEngine::new(IVec2::new(1000, 1000));
        engine.seed = rand::random();
        {
            let id = engine.generate_id();
            engine.spawn_entity(
                EngineEntity::Platform(PlatformEntity::new(
                    id,
                    IVec2::ZERO,
                    IVec2::new(engine.size.x, 1),
                )),
                None,
                false,
            );
        }
        {
            let id = engine.generate_id();
            engine.spawn_entity(
                EngineEntity::MobSpawner(MobSpawnEntity::new_data(
                    id,
                    MobSpawnData {
                        position: IVec2::new(50, 50),
                        size: IVec2::splat(10),
                        mob_type: 1,
                        max_count: 10,
                    },
                    vec![],
                )),
                None,
                false,
            );
        }
        engine
    }

    #[test]
    fn should_generate_different_ids_for_different_seeds() {
        let id0 = {
            let mut engine = GameEngine::new(IVec2::ZERO);
            engine.seed = rand::random();
            engine.generate_id()
        };
        let id1 = {
            let mut engine = GameEngine::new(IVec2::ZERO);
            engine.seed = rand::random();
            engine.generate_id()
        };
        assert_ne!(id0, id1);
    }

    #[test]
    #[ignore]
    fn should_be_constant_size() -> Result<()> {
        let mut engine = test_engine();
        engine.step_to(&TRAILING_STATE_COUNT);

        let mut total_events = 0;
        for v in engine.engine_events_by_step.values() {
            total_events += v.len();
        }
        for event in engine
            .engine_events_by_step
            .get(&(engine.step_index - 1))
            .cloned()
            .unwrap_or_default()
        {
            println!("{:?}", event);
        }
        println!(
            "{} {} {} {}",
            engine.engine_events_by_step.len(),
            engine.game_events_by_step.len(),
            engine.entities_by_step.len(),
            total_events
        );
        let bytes_start =
            bincode::serialize(&engine.engine_at_step(&(engine.step_index - 1), false)?)?;
        engine.step_to(&100000);

        let mut total_events = 0;
        for v in engine.engine_events_by_step.values() {
            total_events += v.len();
        }
        println!(
            "{} {} {} {}",
            engine.engine_events_by_step.len(),
            engine.game_events_by_step.len(),
            engine.entities_by_step.len(),
            total_events
        );
        for event in engine
            .engine_events_by_step
            .get(&(engine.step_index - 1))
            .cloned()
            .unwrap_or_default()
        {
            println!("{:?}", event);
        }
        let bytes_end =
            bincode::serialize(&engine.engine_at_step(&(engine.step_index - 1), false)?)?;
        assert_eq!(bytes_start.len(), bytes_end.len());
        Ok(())
    }
}
