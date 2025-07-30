// A game engine instance for a single map.
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
///   engine events: engine events are process, entity/system addition/removal
///   modification: entities modify themselves and schedule entities for creation/removal
///   game events: game events are processed by the game logic
///   snapshot: the engine state is persisted in memory for rollback
///
use std::collections::BTreeMap;
use std::collections::HashMap;

use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[cfg(not(feature = "zk"))]
pub static START_INSTANT: once_cell::sync::Lazy<std::time::Instant> =
    once_cell::sync::Lazy::new(|| std::time::Instant::now());
#[cfg(not(feature = "zk"))]
pub fn timestamp() -> f64 {
    std::time::Instant::now()
        .duration_since(*START_INSTANT)
        .as_secs_f64()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "G: for<'dee> serde::Deserialize<'dee>"))]
pub struct GameEngine<G: GameLogic> {
    pub id: u128,

    pub size: IVec2,

    pub step_index: u64,

    // entity type, id keyed to struct
    entities: BTreeMap<u128, RefPointer<G::Entity>>,
    empty_entities: BTreeMap<u128, RefPointer<G::Entity>>,
    // step index keyed to entity id to struct
    entities_by_step: BTreeMap<u64, BTreeMap<u128, RefPointer<G::Entity>>>,

    default_input: G::Input,
    inputs: HashMap<u128, G::Input>,
    inputs_by_step: BTreeMap<u64, HashMap<u128, G::Input>>,

    // engine events may be scheduled for the future, game events may not
    engine_events_by_step: BTreeMap<u64, Vec<EngineEvent<G>>>,
    #[serde(skip)]
    game_events_by_step: BTreeMap<u64, Vec<RefPointer<G::Event>>>,

    #[serde(skip, default = "default_game_events::<G>")]
    game_events: (flume::Sender<G::Event>, flume::Receiver<G::Event>),
    #[serde(skip, default = "default_engine_events::<G>")]
    engine_events: (
        flume::Sender<(u64, EngineEvent<G>)>,
        flume::Receiver<(u64, EngineEvent<G>)>,
    ),

    /// Designed to be distinct for each step. e.g. we don't have to store
    #[cfg(not(feature = "zk"))]
    start_timestamp: f64,
    #[cfg(not(feature = "zk"))]
    step_len: f64,
    #[serde(default = "default_trailing_state_len")]
    pub trailing_state_len: u64,
}

fn default_engine_events<G: GameLogic>() -> (
    flume::Sender<(u64, EngineEvent<G>)>,
    flume::Receiver<(u64, EngineEvent<G>)>,
) {
    flume::unbounded()
}

fn default_game_events<G: GameLogic>() -> (flume::Sender<G::Event>, flume::Receiver<G::Event>) {
    flume::unbounded()
}

fn default_trailing_state_len() -> u64 {
    360
}

impl<G: GameLogic> Default for GameEngine<G> {
    fn default() -> Self {
        let mut inputs_by_step = BTreeMap::default();
        inputs_by_step.insert(0, HashMap::default());
        let mut entities_by_step = BTreeMap::default();
        entities_by_step.insert(0, BTreeMap::default());
        Self {
            id: rand::random(),
            size: IVec2::new(1000, 1000),
            step_index: 0,
            entities: BTreeMap::default(),
            empty_entities: BTreeMap::default(),
            entities_by_step,
            default_input: G::Input::default(),
            inputs: HashMap::new(),
            inputs_by_step,
            game_events_by_step: BTreeMap::default(),
            engine_events_by_step: BTreeMap::default(),
            game_events: flume::unbounded(),
            engine_events: flume::unbounded(),
            #[cfg(not(feature = "zk"))]
            start_timestamp: timestamp(),
            #[cfg(not(feature = "zk"))]
            step_len: 1.0 / 60.0,
            trailing_state_len: default_trailing_state_len(),
        }
    }
}

impl<G: GameLogic> GameEngine<G> {
    pub fn id(&self) -> &u128 {
        &self.id
    }

    pub fn size(&self) -> &IVec2 {
        &self.size
    }

    pub fn step_index(&self) -> &u64 {
        &self.step_index
    }

    pub fn spawn_entity(&self, entity: RefPointer<G::Entity>) {
        self.register_event(
            Some(self.step_index),
            EngineEvent::SpawnEntity {
                entity,
                is_non_determinism: false,
            },
        );
    }

    pub fn remove_entity(&self, entity_id: u128) {
        self.register_event(
            Some(self.step_index),
            EngineEvent::RemoveEntity {
                entity_id,
                is_non_determinism: false,
            },
        );
    }

    pub fn spawn_system(&self, entity_id: u128, system_ptr: RefPointer<G::System>) {
        self.register_event(
            Some(self.step_index),
            EngineEvent::SpawnSystem {
                entity_id,
                system_ptr,
                is_non_determinism: false,
            },
        );
    }

    pub fn remove_system(&self, entity_id: u128, system_ptr: RefPointer<G::System>) {
        self.register_event(
            Some(self.step_index),
            EngineEvent::RemoveSystem {
                entity_id,
                system_ptr,
                is_non_determinism: false,
            },
        );
    }

    pub fn register_event(&self, step_index: Option<u64>, event: EngineEvent<G>) {
        let step_index = step_index.unwrap_or(self.step_index);
        self.engine_events.0.send((step_index, event)).unwrap();
    }

    pub fn register_game_event(&self, event: G::Event) {
        self.game_events.0.send(event).unwrap();
    }

    pub fn entity_by_id_untyped(
        &self,
        id: &u128,
        step_index: Option<u64>,
    ) -> Option<&RefPointer<G::Entity>> {
        let step_index = step_index.unwrap_or(self.step_index);
        self.entities_at_step(step_index).get(id)
    }

    pub fn entity_by_id<T: SEEntity<G> + 'static>(
        &self,
        id: &u128,
        step_index: Option<u64>,
    ) -> Option<&T> {
        self.entity_by_id_untyped(id, step_index)
            .map(|entity| entity.extract_ref::<T>())
            .flatten()
    }

    pub fn entities_by_type<T: SEEntity<G> + Send + Sync + 'static>(&self) -> Vec<&T> {
        self.entities
            .iter()
            .filter_map(|(_id, entity)| entity.extract_ref::<T>())
            .collect::<Vec<_>>()
    }

    pub fn input_for_entity(&self, id: &u128) -> &G::Input {
        self.inputs.get(id).unwrap_or(&self.default_input)
    }

    /// A step is considered complete at the _end_ of this function
    pub fn step(&mut self) -> Vec<RefPointer<G::Event>> {
        // Execute the modification phase of the step
        // When an entity is stepped we get a mutable next version
        // as a clone of the current version, then apply all
        // systems. Once this is complete it is put in a RefPointer
        // and stored.
        let mut next_entities = BTreeMap::default();
        for (id, entity) in &self.entities {
            let mut next_self_maybe = None;
            if entity.prestep(self) {
                let mut next_self = (**entity).clone();
                entity.step(self, &mut next_self);
                next_self_maybe = Some(next_self);
            }
            entity.step_systems(self, &mut next_self_maybe);
            // insert the next_self, if it exists
            // otherwise copy the existingRefPointer
            let next_self_ptr = if let Some(next_self) = next_self_maybe {
                RefPointer::from(next_self)
            } else {
                entity.clone()
            };
            if next_entities.insert(*id, next_self_ptr).is_some() {
                println!("WARNING: stepped an entity that was not previously present!");
            }
        }

        self.entities = next_entities;

        // our entities are stepped, now we have discrete
        // engine events to apply to self.entities

        for (step_index, event) in self.engine_events.1.drain() {
            assert!(
                step_index >= self.step_index,
                "received engine event in the past!"
            );
            self.engine_events_by_step
                .entry(step_index)
                .or_default()
                .push(event);
        }

        // iterate over all events for the current step
        for event in self
            .engine_events_by_step
            .entry(self.step_index)
            .or_default()
        {
            match event {
                EngineEvent::SpawnEntity { entity, .. } => {
                    if let Some(e) = self.entities.insert(entity.id(), entity.clone()) {
                        println!("WARNING: inserting entity that already existed! {:?}", e);
                        // if &e == entity {
                        //     println!("entities are equal");
                        // }
                        println!("new: {:?}", entity);
                    }
                }
                EngineEvent::RemoveEntity {
                    entity_id,
                    is_non_determinism: _,
                } => {
                    if let None = self.entities.remove(&entity_id) {
                        println!("WARNING: attempting to remove non-existent entity");
                    }
                }
                EngineEvent::Input {
                    input,
                    entity_id,
                    is_non_determinism: _,
                } => {
                    self.inputs.insert(*entity_id, input.clone());
                }
                EngineEvent::SpawnSystem {
                    entity_id,
                    system_ptr,
                    ..
                } => {
                    if let Some(entity_ptr) = self.entities.get(entity_id) {
                        let mut entity = (**entity_ptr).clone();
                        entity.systems_mut().push(system_ptr.clone());
                        assert!(
                            self.entities
                                .insert(entity.id(), RefPointer::new(entity))
                                .is_some()
                        );
                    } else {
                        println!("WARNING: attempting to spawn system for non-existent entity");
                    }
                }
                EngineEvent::RemoveSystem {
                    entity_id,
                    system_ptr,
                    ..
                } => {
                    if let Some(entity_ptr) = self.entities.get(entity_id) {
                        let mut entity = (**entity_ptr).clone();
                        entity
                            .systems_mut()
                            .retain(|ptr| !RefPointer::ptr_eq(ptr, &system_ptr));
                        assert!(
                            self.entities
                                .insert(entity.id(), RefPointer::new(entity))
                                .is_some()
                        );
                    } else {
                        println!("WARNING: attempting to remove system for non-existent entity");
                    }
                }
            }
        }

        // record step change
        // this changes the behavior of e.g. `GameEngine<G>::entity_by_id`
        self.step_index += 1;

        // record state for rewind
        if self.trailing_state_len != 0 {
            self.entities_by_step
                .insert(self.step_index, self.entities.clone());
            self.inputs_by_step
                .insert(self.step_index, self.inputs.clone());
        }

        let game_events = self
            .game_events
            .1
            .drain()
            .map(|event| RefPointer::from(event))
            .collect::<Vec<_>>();

        // step 0 cannot have game events, because no entities or
        // systems can be spawned in time to dispatch game events
        self.game_events_by_step
            .insert(self.step_index, game_events.clone());

        // Game logic is the first invocation in a step. It's synchronous
        // with the step change (~15 lines above here)
        G::handle_game_events(self, &game_events.clone());

        // Do some engine housekeeping
        if self.trailing_state_len != 0 && self.step_index >= self.trailing_state_len {
            let step_to_remove = self.step_index - self.trailing_state_len;
            self.entities_by_step.retain(|k, _v| k > &step_to_remove);
            self.engine_events_by_step
                .retain(|k, _v| k > &step_to_remove);
            self.game_events_by_step.retain(|k, _v| k > &step_to_remove);
            self.inputs_by_step.remove(&step_to_remove);
        }

        // for exfil
        game_events
    }

    pub fn new(size: IVec2, id: u128) -> Self {
        Self {
            id,
            size,
            #[cfg(not(feature = "zk"))]
            start_timestamp: timestamp(),
            ..Default::default()
        }
    }

    pub fn new_simple(size: IVec2, id: u128) -> Self {
        Self {
            id,
            size,
            #[cfg(not(feature = "zk"))]
            start_timestamp: timestamp(),
            trailing_state_len: 0,
            ..Default::default()
        }
    }

    pub fn step_hash(&self, step_index: &u64) -> anyhow::Result<blake3::Hash> {
        let step_index = if step_index == &0 {
            println!(
                "WARNING: Calculating a hash for step 0 is nonsensical, there cannot be any entities"
            );
            &1
        } else {
            step_index
        };
        // hash of all entities
        if let Some(entities) = self.entities_by_step.get(step_index) {
            let serialized = bincode::serialize(entities)?;
            Ok(blake3::hash(&serialized))
        } else {
            anyhow::bail!("error calculating engine.step_hash, {step_index} not known to engine");
        }
    }

    pub fn game_events(&self, from_step: u64, to_step: u64) -> Vec<RefPointer<G::Event>> {
        self.game_events_by_step
            .range(from_step..to_step)
            .map(|(_, game_events)| game_events.clone())
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Retrieve an engine at the _end_ of `target_step_index`.
    /// is_non_determinism events occur independently on the engine state. e.g. a player logging on
    pub fn engine_at_step(
        &self,
        target_step_index: &u64,
        rewindable: bool,
    ) -> anyhow::Result<Self> {
        if let Some(entities) = self.entities_by_step.get(target_step_index)
            && let Some(inputs) = self.inputs_by_step.get(target_step_index)
        {
            let mut out = Self::default();

            // get all future events that areis_non_determinism
            out.engine_events_by_step = self
                .engine_events_by_step
                .range(target_step_index..)
                .map(|(k, v)| {
                    (
                        *k,
                        v.iter()
                            .filter(|v| v.is_non_determinism())
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
                    .range(..=target_step_index)
                    .map(|(k, v)| (*k, v.clone()))
                    .collect::<BTreeMap<_, _>>();
                out.entities_by_step = self
                    .entities_by_step
                    .range(..=target_step_index)
                    .map(|(si, data)| (*si, data.clone()))
                    .collect::<BTreeMap<_, _>>();
                out.inputs_by_step = self
                    .inputs_by_step
                    .range(..=target_step_index)
                    .map(|(k, v)| (*k, v.clone()))
                    .collect::<BTreeMap<_, _>>();
            }
            out.id = self.id;
            out.size = self.size.clone();
            out.entities = entities.clone();
            out.inputs = inputs.clone();

            out.step_index = *target_step_index;
            Ok(out)
        } else {
            anyhow::bail!("WARNING: step index {target_step_index} is too far in the past")
        }
    }

    pub fn integrate_event(&mut self, step_index: u64, event: EngineEvent<G>) {
        let mut btree: BTreeMap<u64, Vec<EngineEvent<G>>> = BTreeMap::new();
        btree.entry(step_index).or_default().push(event);
        self.integrate_events(btree);
    }

    pub fn integrate_events(&mut self, events: BTreeMap<u64, Vec<EngineEvent<G>>>) {
        if events.is_empty() {
            return;
        }
        // go to the step before the first event
        let from_step_index = *events.first_key_value().unwrap().0;
        if from_step_index >= self.step_index {
            for (step_index, events) in events {
                for event in events {
                    println!("integrating event at step {step_index}");
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

    pub fn entities_at_step(&self, step_index: u64) -> &BTreeMap<u128, RefPointer<G::Entity>> {
        if step_index == 0 {
            return &self.empty_entities;
        }
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
                    &self.empty_entities
                }
            }
        }
    }

    #[cfg(not(feature = "zk"))]
    pub fn expected_step_index(&self) -> u64 {
        let now = timestamp();
        assert!(now >= self.start_timestamp, "GameEngine time ran backward");
        // rounds toward 0
        self.step_index
            .max(((now - self.start_timestamp) / self.step_len) as u64)
    }

    /// Automatically step forward in time as much as needed
    #[cfg(not(feature = "zk"))]
    pub fn tick(&mut self) {
        let expected = self.expected_step_index();
        if expected <= self.step_index {
            println!("noop tick: your tick rate is too high!");
            return;
        }
        self.step_to(&expected);
    }

    pub fn step_to(&mut self, to_step: &u64) -> Vec<RefPointer<G::Event>> {
        assert!(to_step > self.step_index());
        let mut all_events = Vec::new();
        for _ in 0..(to_step - self.step_index()) {
            let mut events = self.step();
            all_events.append(&mut events);
        }
        all_events
    }
}

#[cfg(test)]
mod tests {}
