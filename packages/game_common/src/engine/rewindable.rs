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
use std::time::Instant;

use bevy_math::IVec2;
use db::AbilityExpRecord;
use once_cell::sync::Lazy;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro64StarStar;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

pub static START_INSTANT: Lazy<Instant> = Lazy::new(|| Instant::now());
pub fn timestamp() -> f64 {
    Instant::now().duration_since(*START_INSTANT).as_secs_f64()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewindableGameEngine {
    pub id: u128,
    pub seed: u64,

    pub size: IVec2,

    pub step_index: u64,

    // entity type, id keyed to struct
    pub entities: BTreeMap<u128, EngineEntity>,
    // step index keyed to entity id to struct
    entities_by_step: BTreeMap<u64, BTreeMap<u128, EngineEntity>>,

    inputs: HashMap<u128, EntityInput>,
    inputs_by_step: BTreeMap<u64, HashMap<u128, EntityInput>>,

    // engine events may be scheduled for the future, game events may not
    engine_events_by_step: BTreeMap<u64, Vec<EngineEvent>>,
    #[serde(skip)]
    game_events_by_step: BTreeMap<u64, Vec<GameEvent>>,

    #[serde(skip, default = "default_game_events")]
    pub game_events: (flume::Sender<GameEvent>, flume::Receiver<GameEvent>),
    #[serde(skip, default = "default_engine_events")]
    pub engine_events: (
        flume::Sender<(u64, EngineEvent)>,
        flume::Receiver<(u64, EngineEvent)>,
    ),

    /// Designed to be distinct for each step. e.g. we don't have to store
    /// rng state (aside from seed) when rolling back.
    #[serde(skip, default = "default_rng")]
    rng_state: (u64, Xoroshiro64StarStar),
    start_timestamp: f64,
}

fn default_rng() -> (u64, Xoroshiro64StarStar) {
    (u64::MAX, Xoroshiro64StarStar::seed_from_u64(0))
}

fn default_game_events() -> (flume::Sender<GameEvent>, flume::Receiver<GameEvent>) {
    flume::unbounded()
}

fn default_engine_events() -> (
    flume::Sender<(u64, EngineEvent)>,
    flume::Receiver<(u64, EngineEvent)>,
) {
    flume::unbounded()
}

impl Default for RewindableGameEngine {
    fn default() -> Self {
        Self::new(IVec2 { x: 1000, y: 1000 }, 1)
    }
}

impl GameEngine for RewindableGameEngine {
    fn id(&self) -> &u128 {
        &self.id
    }

    fn seed(&self) -> &u64 {
        &self.seed
    }

    fn generate_id(&mut self) -> u128 {
        loop {
            let id = self.rng().random::<u128>();
            if !self.entities.contains_key(&id) {
                return id;
            }
        }
    }

    fn rng(&mut self) -> &mut Xoroshiro64StarStar {
        if self.rng_state.0 != self.step_index {
            self.rng_state.1 = Xoroshiro64StarStar::seed_from_u64(self.seed + self.step_index);
            self.rng_state.0 = self.step_index;
        }
        &mut self.rng_state.1
    }

    fn size(&self) -> &IVec2 {
        &self.size
    }

    fn step_index(&self) -> &u64 {
        &self.step_index
    }

    fn spawn_entity(&self, entity: EngineEntity, step_index: Option<u64>, is_universal: bool) {
        self.engine_events
            .0
            .send((
                step_index.unwrap_or(self.step_index),
                EngineEvent::SpawnEntity {
                    entity,
                    universal: is_universal,
                },
            ))
            .unwrap();
    }

    fn remove_entity(&self, entity_id: &u128, step_index: Option<u64>, universal: bool) {
        self.register_event(
            step_index,
            EngineEvent::RemoveEntity {
                entity_id: *entity_id,
                universal,
            },
        );
    }

    fn register_event(&self, step_index: Option<u64>, event: EngineEvent) {
        self.engine_events
            .0
            .send((step_index.unwrap_or(self.step_index), event))
            .unwrap();
    }

    fn register_game_event(&self, event: GameEvent) {
        self.game_events.0.send(event).unwrap();
    }

    fn entity_by_id_untyped(&self, id: &u128, step_index: Option<u64>) -> Option<&EngineEntity> {
        let step_index = step_index.unwrap_or(self.step_index);
        self.entities_at_step(step_index).get(id)
    }

    fn input_for_entity(&self, id: &u128) -> &EntityInput {
        static DEFAULT_INPUT: LazyLock<EntityInput> = LazyLock::new(|| EntityInput::default());
        self.inputs.get(id).unwrap_or(&DEFAULT_INPUT)
    }

    /// Return an entity by id at a certain step index, if possible. Extracts the underlying
    /// entity type from the EngineEntity
    fn entity_by_id<T: 'static + EEntity>(&self, id: &u128, step_index: Option<u64>) -> Option<&T> {
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

    fn entities_by_type<T: 'static + EEntity>(&self) -> impl Iterator<Item = &T> {
        self.entities.iter().filter_map(|(_, val)| {
            if val.runtime_type_id() == TypeId::of::<T>() {
                val.extract_ref::<T>()
            } else {
                None
            }
        })
    }

    /// A step is considered complete at the _end_ of this function
    fn step(&mut self) -> Vec<GameEvent> {
        self.entities_by_step
            .insert(self.step_index, self.entities.clone());
        self.inputs_by_step
            .insert(self.step_index, self.inputs.clone());

        // Execute the modification phase of the step
        let mut next_entities = BTreeMap::new();
        for (id, entity) in &self.entities {
            let stepped = entity.step(self);
            next_entities.insert(*id, stepped);
        }
        self.entities = next_entities;

        // collect engine events in the channel
        for (step_index, event) in self.engine_events.1.drain() {
            if step_index < self.step_index {
                panic!("received engine event in the past!");
            }
            self.engine_events_by_step
                .entry(step_index)
                .or_default()
                .push(event);
        }

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
                EngineEvent::Input {
                    input,
                    entity_id,
                    universal: _,
                } => {
                    self.inputs.insert(*entity_id, input.clone());
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
                        println!("WARNING: sending message from non-existent entity")
                    }
                }
            }
        }

        // Do some engine housekeeping
        if self.step_index >= TRAILING_STATE_COUNT {
            let step_to_remove = self.step_index - TRAILING_STATE_COUNT;
            self.entities_by_step.retain(|k, _v| k > &step_to_remove);
            self.engine_events_by_step
                .retain(|k, _v| k > &step_to_remove);
            self.game_events_by_step.retain(|k, _v| k > &step_to_remove);
            self.inputs_by_step.remove(&step_to_remove);
        }

        let game_events = self.game_events.1.drain().collect::<Vec<_>>();
        self.game_events_by_step
            .insert(self.step_index, game_events.clone());

        for game_event in &game_events {
            self.process_game_event(game_event);
        }

        // Officially move to the next step
        self.step_index += 1;

        game_events
    }

    fn process_game_event(&mut self, event: &GameEvent) {
        // handle game events that occurred during a step
        match event {
            GameEvent::PlayerEnterPortal {
                player_id: _,
                entity_id,
                from_map: _,
                to_map: _,
                requested_spawn_pos: _,
            } => {
                // player will be despawned immediately
                self.entities.remove(entity_id);
            }
            GameEvent::PlayerAbilityExp(player_entity_id, ability, amount) => {
                // we'll just handle synchronizing the player entities stats here
                // database logic lives in map_instance.rs or game.rs
                if let Some(player_entity) = self
                    .entities
                    .get_mut(player_entity_id)
                    .map(|v| v.extract_ref_mut::<PlayerEntity>())
                    .flatten()
                {
                    player_entity.stats.increment(&AbilityExpRecord {
                        player_id: player_entity.player_id.clone(),
                        amount: *amount,
                        ability: ability.clone(),
                    });
                } else {
                    println!("WARNING: player entity does not exist in engine for ability exp!");
                }
            }
            GameEvent::PlayerPickUpRequest(player_entity_id) => {
                if let Some(player_entity) = self.entities.get(player_entity_id).cloned() {
                    let game_events_sender = self.game_events.0.clone();
                    // there are quirks with using entities_by_type in the default handler
                    // see GameEngine::step
                    for item in self
                        .entities_by_type::<ItemEntity>()
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        if !self.entities.contains_key(&item.id) {
                            continue;
                        }
                        if item.rect().intersect(player_entity.rect()).is_empty() {
                            continue;
                        }
                        // otherwise pick up the item
                        self.entities.remove(&item.id);
                        // mark user as having object
                        game_events_sender
                            .send(GameEvent::PlayerPickUp(
                                player_entity
                                    .extract_ref::<PlayerEntity>()
                                    .unwrap()
                                    .player_id
                                    .clone(),
                                item.item_type,
                                item.count,
                            ))
                            .unwrap();
                        return;
                    }
                }
            }
            GameEvent::PlayerPickUp(_, _, _) => {}
            GameEvent::PlayerHealth(_, _) => {}
            GameEvent::Message(_, _) => {}
        }
    }
}

impl RewindableGameEngine {
    pub fn new(size: IVec2, seed: u64) -> Self {
        let mut out = Self {
            id: 0,
            size,
            rng_state: (0, Xoroshiro64StarStar::seed_from_u64(seed)),
            game_events: default_game_events(),
            engine_events: default_engine_events(),
            seed,
            step_index: 0,
            entities: BTreeMap::new(),
            entities_by_step: BTreeMap::new(),
            inputs: HashMap::new(),
            inputs_by_step: BTreeMap::new(),
            engine_events_by_step: BTreeMap::new(),
            game_events_by_step: BTreeMap::new(),
            start_timestamp: timestamp(),
        };
        out.id = out.generate_id();
        out
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

    pub fn entity_by_id_mut<T: 'static + EEntity>(&mut self, id: &u128) -> Option<&mut T> {
        self.entities
            .get_mut(id)
            .map(|v| v.extract_ref_mut::<T>())
            .flatten()
    }

    /// Retrieve a past instance of the engine that will be equal
    /// to self after N steps
    /// universal events occur independently on the engine state. e.g. a player logging on
    pub fn engine_at_step(
        &self,
        target_step_index: &u64,
        rewindable: bool,
    ) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(target_step_index)
            && let Some(inputs) = self.inputs_by_step.get(target_step_index)
        {
            let mut out = Self::default();

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
                out.inputs_by_step = self
                    .inputs_by_step
                    .range(..target_step_index)
                    .map(|(k, v)| (*k, v.clone()))
                    .collect::<BTreeMap<_, _>>();
            }
            out.id = self.id;
            out.size = self.size.clone();
            out.seed = self.seed;
            out.entities = entities.clone();
            out.inputs = inputs.clone();

            out.step_index = *target_step_index;
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {target_step_index} is too far in the past")
        }
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
                self.inputs_by_step = past_engine.inputs_by_step;
                self.inputs = past_engine.inputs;
            } else {
                panic!("failed to generate past engine");
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    pub fn test_engine() -> RewindableGameEngine {
        let mut engine = RewindableGameEngine::new(IVec2::new(1000, 1000), 0);
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
            let mut engine = RewindableGameEngine::new(IVec2::ZERO, rand::random());
            engine.generate_id()
        };
        let id1 = {
            let mut engine = RewindableGameEngine::new(IVec2::ZERO, rand::random());
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
