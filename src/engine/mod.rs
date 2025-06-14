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
///   modification: entities modify themselves and schedule entities for creation/removal
///   creation: entities scheduled for creation are created
///   removal: entities pending removal are removed
///
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::mem;
use std::mem::discriminant;
use std::mem::Discriminant;

use bevy_math::IVec2;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

pub mod entity;
pub mod game_event;
// #[cfg(test)]
// pub mod tests;

use entity::player::PlayerEntity;
use entity::EEntity;
use entity::EngineEntity;
use entity::EntityInput;
use entity::SEEntity;
use game_event::ServerEvent;

use crate::engine::entity::platform::PlatformEntity;
use crate::engine::entity::text::TextEntity;
use crate::engine::game_event::GameEvent;
use crate::engine::game_event::GameEventType;
use crate::engine::game_event::HasUniversal;
use crate::map::MapData;
use crate::timestamp;

pub const STEP_LEN_S: f64 = 1. / 60.;
pub const STEP_LEN_S_F32: f32 = 1. / 60.;
pub const STEPS_PER_SECOND: u64 = (1.0 / STEP_LEN_S_F32) as u64;
pub const STEPS_PER_SECOND_I32: i32 = (1.0 / STEP_LEN_S_F32) as i32;
pub const TRAILING_STATE_COUNT: u64 = 360;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEngine {
    pub id: u128,
    pub step_index: u64,
    pub start_timestamp: f64,
    pub map: MapData,

    // entity id keyed to struct
    #[serde(serialize_with = "serialize_unpure_entities")]
    pub entities: BTreeMap<u128, EngineEntity>,
    // step index keyed to entity id to struct
    // #[serde(skip)]
    pub entities_by_step: HashMap<u64, BTreeMap<u128, EngineEntity>>,

    // #[serde(skip)]
    // pub events_by_step: BTreeMap<A
    pub events_by_type: HashMap<GameEventType, BTreeMap<u64, BTreeMap<u128, GameEvent>>>,
    #[serde(skip)]
    pub server_events: Vec<ServerEvent>,

    #[serde(skip)]
    grouped_entities: (u64, HashMap<Discriminant<EngineEntity>, Vec<EngineEntity>>),
    #[serde(skip)]
    pub enable_debug_markers: bool,
    pub seed: u64,
    #[serde(skip, default = "default_rng")]
    rng: (u64, ChaCha8Rng),
}

// default seeded rng for engine
fn default_rng() -> (u64, ChaCha8Rng) {
    (0, ChaCha8Rng::seed_from_u64(0))
}

impl Default for GameEngine {
    fn default() -> Self {
        Self {
            id: 0,
            step_index: 0,
            start_timestamp: 0.0,
            map: MapData::default(),
            entities: BTreeMap::new(),
            entities_by_step: HashMap::new(),
            events_by_type: HashMap::new(),
            server_events: Vec::new(),
            grouped_entities: (0, HashMap::new()),
            enable_debug_markers: false,
            seed: 0,
            rng: default_rng(),
        }
    }
}

fn serialize_unpure_entities<S>(
    entities: &BTreeMap<u128, EngineEntity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    entities
        .iter()
        .filter(|(_id, entity)| !entity.pure())
        .collect::<Vec<_>>()
        .serialize(serializer)
}

impl GameEngine {
    pub fn new(map: MapData) -> Self {
        let mut engine = Self {
            id: rand::random(),
            start_timestamp: timestamp(),
            map: map.clone(),
            step_index: 0,
            rng: default_rng(),
            ..Default::default()
        };
        // TODO: move this into map parsing/loading logicso the
        // system can be extended without involving the engineI
        //
        //
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
        // mob spawns
        for spawn in &map.mob_spawns {
            let mut spawn_with_id = spawn.clone();
            spawn_with_id.id = engine.generate_id();
            engine.spawn_entity(EngineEntity::MobSpawner(spawn_with_id), None, true);
        }
        // portal spawns
        for portal in &map.portals {
            let id = engine.generate_id();
            let mut portal_clone = portal.clone();
            if portal_clone.size.x == 0 {
                portal_clone.size = IVec2::new(60, 60);
            }
            portal_clone.id = id;
            engine.spawn_entity(EngineEntity::Portal(portal_clone), None, true);
        }
        engine
    }

    pub fn server_events(&mut self) -> Vec<ServerEvent> {
        mem::take(&mut self.server_events)
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

    pub fn say_to(&mut self, entity_id: &u128, msg: String) {
        if let Some(entity) = self.entities.get(entity_id).cloned() {
            let mut text = TextEntity::new(self.generate_id(), IVec2::MAX, IVec2::splat(100));
            text.text = msg;
            text.pure = false;
            text.attached_to = Some((
                *entity_id,
                entity.center() + IVec2::new(0, entity.size().y + 50),
            ));
            text.disappears_at_step_index = self.step_index + 120;
            self.spawn_entity(EngineEntity::Text(text), None, true);
        } else {
            println!("WARNING: attempting to say {msg} to entity {entity_id} which does not exist in engine");
        }
    }

    /// Retrieve a past instance of the engine that will be equal
    /// to self after N steps
    /// universal events occur independently on the engine state. e.g. a player logging on
    ///
    pub fn engine_at_step(&self, target_step_index: &u64) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(target_step_index) {
            let mut out = Self::default();
            out.id = self.id;
            // get all events in the past
            // and universal events in the future?
            out.events_by_type = self.events_by_type.clone();
            // out.events_by_step = self.events_by_step.clone();
            for (_, events_by_step) in out.events_by_type.iter_mut() {
                for (step_index, events) in events_by_step.iter_mut() {
                    // keep all events in past
                    if step_index < target_step_index {
                        continue;
                    }
                    // keep only universal events and inputs in the future
                    events.retain(|_event_id, event| event.universal());
                }
            }

            out.start_timestamp = self.start_timestamp;
            out.map = self.map.clone();
            out.entities = entities.clone();
            out.enable_debug_markers = self.enable_debug_markers;

            out.step_index = *target_step_index;
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {target_step_index} is too far in the past")
        }
    }

    /// Return the game events necessary to replay a past engine
    /// to current engine state
    ///
    /// it's indendented super deep because it's inside a lot
    /// of nested data structures for efficient access
    pub fn universal_events_since_step(
        &self,
        from_step_index: &u64,
        to_step_index: Option<u64>,
    ) -> BTreeMap<u64, HashMap<u128, GameEvent>> {
        let to_step_index = to_step_index.unwrap_or(self.step_index);
        let mut out = BTreeMap::new();
        for (_, events_by_step) in &self.events_by_type {
            for (step_index, events) in events_by_step {
                if step_index < &from_step_index
                    || step_index >= &to_step_index
                    || events.is_empty()
                {
                    continue;
                }
                let universal_events = events
                    .iter()
                    .filter(|(_event_id, event)| event.universal())
                    .collect::<BTreeMap<_, _>>();
                if universal_events.is_empty() {
                    continue;
                }
                let out_step: &mut HashMap<u128, GameEvent> = out.entry(*step_index).or_default();
                for (event_id, event) in universal_events {
                    out_step.insert(*event_id, event.clone());
                }
            }
        }
        out
    }

    /// TODO: generic implementation
    /// e.g. spawn_entity::<PlayerEntity>(&mut self)
    /// optionally provide a step index the player should spawn at (must be in the past)
    pub fn spawn_player_entity(
        &mut self,
        player_id: String,
        position_maybe: Option<IVec2>,
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
        let target_step_index = step_index.unwrap_or(self.step_index);
        let id: u128 = self.rng().random();
        if target_step_index >= self.step_index {
            self.register_event(
                Some(target_step_index),
                GameEvent::SpawnEntity {
                    id,
                    entity,
                    universal: is_universal,
                },
            );
        } else {
            self.integrate_event(
                target_step_index,
                GameEvent::SpawnEntity {
                    id,
                    entity,
                    universal: is_universal,
                },
            );
        }
    }

    pub fn remove_entity(&mut self, entity_id: u128, universal: bool) {
        let id = self.rng().random();
        self.register_event(
            None,
            GameEvent::RemoveEntity {
                id,
                entity_id,
                universal,
            },
        );
    }

    pub fn integrate_event(&mut self, step_index: u64, event: GameEvent) {
        let mut btree: BTreeMap<u64, HashMap<u128, GameEvent>> = BTreeMap::new();
        btree
            .entry(step_index)
            .or_default()
            .insert(self.generate_id(), event);
        self.integrate_events(btree);
    }

    pub fn integrate_events(&mut self, events: BTreeMap<u64, HashMap<u128, GameEvent>>) {
        if events.is_empty() {
            return;
        }
        let from_step_index = *events.first_key_value().unwrap().0 - 1;
        if from_step_index >= self.step_index {
            for (step_index, events) in events {
                for (_event_id, event) in events {
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
            if let Ok(mut past_engine) = self.engine_at_step(&from_step_index) {
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
                self.entities = past_engine.entities;
                self.entities_by_step = past_engine.entities_by_step;
                self.events_by_type = past_engine.events_by_type;
                self.grouped_entities = (0, HashMap::new());
            } else {
                panic!("failed to generate past engine");
            }
        }
    }

    pub fn register_event(&mut self, step_index: Option<u64>, event: GameEvent) {
        let step_index = step_index.unwrap_or(self.step_index);
        // if the step is in the past we insert and replay
        // engine will replay all over events that ocurred as well
        let event_id = self.generate_id();
        if let Some(event) = self
            .events_by_type
            .entry(GameEventType::from(&event))
            .or_default()
            .entry(step_index)
            .or_default()
            .insert(event_id, event)
        {
            println!("overwriting event: {:?}", event);
        }
    }

    pub fn latest_input(&mut self, id: &u128) -> (u64, EntityInput) {
        self.events_by_type
            .entry(GameEventType::Input)
            .or_default()
            .range(..=self.step_index)
            .rev()
            .find_map(|(step_index, events)| {
                events.values().find_map(|event| match event {
                    GameEvent::Input {
                        entity_id, input, ..
                    } if entity_id == id => Some((*step_index, input.clone())),
                    _ => None,
                })
            })
            .unwrap_or((0, EntityInput::default()))
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
        self.step_to(&expected);
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

    pub fn step_to(&mut self, target_step_index: &u64) {
        if target_step_index < &self.step_index {
            panic!("cannot step forward to a point in the past");
        }
        while &self.step_index != target_step_index {
            self.step();
        }
    }

    /// A step is considered complete at the _end_ of this function
    pub fn step(&mut self) {
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
        for (_event_id, event) in self
            .events_by_type
            .entry(GameEventType::SpawnEntity)
            .or_default()
            .entry(self.step_index)
            .or_default()
        {
            match event {
                GameEvent::SpawnEntity {
                    id: _,
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
                _ => {}
            }
        }

        // Execute the removal phase of the step
        for (_event_id, event) in self
            .events_by_type
            .entry(GameEventType::RemoveEntity)
            .or_default()
            .entry(self.step_index)
            .or_default()
        {
            match event {
                GameEvent::RemoveEntity {
                    id: _,
                    entity_id,
                    universal: _,
                } => {
                    self.entities.remove(entity_id);
                }
                _ => {}
            }
        }

        // Do some engine housekeeping
        if self.step_index >= TRAILING_STATE_COUNT {
            let step_to_remove = self.step_index - TRAILING_STATE_COUNT;
            self.entities_by_step.remove(&step_to_remove);
            for (_, events_by_step) in self.events_by_type.iter_mut() {
                events_by_step.remove(&step_to_remove);
            }
        }

        // Officially move to the next step
        self.step_index += 1;
    }
}
