// use std::cell::RefCell;
// use std::sync::LazyLock;
//
// use bevy_math::IVec2;
// use rand::Rng;
// use rand::SeedableRng;
// use rand_xoshiro::Xoroshiro64StarStar;
//
// use crate::prelude::*;
//
// /// A `GameEngine` implementation optimized for execution
// /// inside a ZKVM. Should execute byte for byte the same as
// /// other GameEngine implementations
// pub struct SimpleGameEngine {
//     id: u128,
//     seed: u64,
//     size: IVec2,
//     step_index: u64,
//     rng_state: (u64, Xoroshiro64StarStar),
//
//     /// step_index, engine event
//     pending_engine_events: RefCell<Vec<(u64, EngineEvent)>>,
//
//     entities: Vec<EngineEntity>,
// }
//
// impl SimpleGameEngine {
//     pub fn new(size: IVec2, seed: u64) -> Self {
//         let mut out = Self {
//             id: 0,
//             seed,
//             size,
//             step_index: 0,
//             rng_state: (0, Xoroshiro64StarStar::seed_from_u64(seed)),
//             pending_engine_events: RefCell::new(Vec::new()),
//             entities: Vec::new(),
//         };
//         out.id = out.generate_id();
//         out
//     }
// }
//
// impl SimpleGameEngine {
//     fn id(&self) -> &u128 {
//         &self.id
//     }
//
//     fn seed(&self) -> &u64 {
//         &self.seed
//     }
//
//     fn size(&self) -> &bevy_math::IVec2 {
//         &self.size
//     }
//
//     fn step_index(&self) -> &u64 {
//         &self.step_index
//     }
//
//     fn rng(&mut self) -> &mut Xoroshiro64StarStar {
//         if self.rng_state.0 != self.step_index {
//             self.rng_state.1 = Xoroshiro64StarStar::seed_from_u64(self.seed + self.step_index);
//             self.rng_state.0 = self.step_index;
//         }
//         &mut self.rng_state.1
//     }
//
//     fn generate_id(&mut self) -> u128 {
//         loop {
//             let id = self.rng().random::<u128>();
//             if self.entity_by_id_untyped(&id, None).is_none() {
//                 return id;
//             }
//         }
//     }
//
//     fn register_event(&self, step_index: Option<u64>, event: EngineEvent) {
//         let step_index = step_index.unwrap_or(self.step_index);
//         self.pending_engine_events
//             .borrow_mut()
//             .push((step_index, event));
//     }
//
//     fn spawn_entity(&self, entity: EngineEntity, step_index: Option<u64>, is_non_determ: bool) {
//         self.register_event(
//             step_index,
//             EngineEvent::SpawnEntity {
//                 entity,
//                 universal: is_non_determ,
//             },
//         );
//     }
//
//     fn remove_entity(&self, id: &u128, step_index: Option<u64>, is_non_determ: bool) {
//         self.register_event(
//             step_index,
//             EngineEvent::RemoveEntity {
//                 entity_id: *id,
//                 universal: is_non_determ,
//             },
//         );
//     }
//
//     fn entity_by_id<T: 'static + EEntity>(&self, id: &u128, step_index: Option<u64>) -> Option<&T> {
//         for entity in &self.entities {
//             if &entity.id() == id {
//                 return entity.extract_ref::<T>();
//             }
//         }
//         None
//     }
//
//     fn entities_by_type<T: 'static + EEntity>(&self) -> impl Iterator<Item = &T> {
//         let mut out = Vec::new();
//         for entity in &self.entities {
//             if entity.type_id() == type_id_of::<T>().unwrap() {
//                 out.push(entity.extract_ref::<T>().unwrap());
//             }
//         }
//         out.into_iter()
//     }
//
//     fn entity_by_id_untyped(&self, id: &u128, step_index: Option<u64>) -> Option<&EngineEntity> {
//         for entity in &self.entities {
//             if &entity.id() == id {
//                 return Some(entity);
//             }
//         }
//         None
//     }
//
//     fn input_for_entity(&self, id: &u128) -> &EntityInput {
//         static DEFAULT_INPUT: LazyLock<EntityInput> = LazyLock::new(|| EntityInput::default());
//         &DEFAULT_INPUT
//     }
//
//     fn register_game_event(&self, event: GameEvent) {}
//
//     fn process_game_event(&mut self, event: &GameEvent) {}
//
//     fn step(&mut self) -> Vec<GameEvent> {
//         let mut next_entities = Vec::new();
//         for entity in &self.entities {
//             let stepped = entity.step(self);
//             next_entities.push(stepped);
//         }
//         self.entities = next_entities;
//
//         for (step_index, event) in self
//             .pending_engine_events
//             .borrow_mut()
//             .drain(..)
//             .collect::<Vec<_>>()
//         {
//             if step_index != self.step_index {
//                 self.pending_engine_events
//                     .borrow_mut()
//                     .push((step_index, event));
//                 continue;
//             }
//             match event {
//                 EngineEvent::SpawnEntity {
//                     entity,
//                     universal: _,
//                 } => {
//                     self.entities.push(entity);
//                 }
//                 EngineEvent::RemoveEntity {
//                     entity_id,
//                     universal: _,
//                 } => {
//                     self.entities.retain(|entity| entity.id() != entity_id);
//                 }
//                 EngineEvent::Message {
//                     text,
//                     entity_id,
//                     universal: _,
//                 } => {
//                     if let Some(entity) = self.entity_by_id_untyped(&entity_id, None) {
//                         let is_player = match entity {
//                             EngineEntity::Player(_) => true,
//                             _ => false,
//                         };
//                         let msg_entity = MessageEntity::new_text(
//                             text.clone(),
//                             self.step_index,
//                             entity.id(),
//                             is_player,
//                         );
//                         self.entities.push(EngineEntity::Message(msg_entity));
//                     } else {
//                         println!("WARNING: sending message from non-existent entity");
//                     }
//                 }
//                 _ => {}
//             }
//         }
//
//         self.step_index += 1;
//         vec![]
//     }
// }
