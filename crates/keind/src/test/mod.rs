use std::collections::HashMap;

use anyhow::Result;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EntityInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {}

entity_struct!(TestGameLogic, pub struct TestEntity {});

impl SEEntity<TestGameLogic> for TestEntity {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestSystem;

impl EEntitySystem<TestGameLogic> for TestSystem {
    fn prestep(
        &self,
        engine: &GameEngine<TestGameLogic>,
        entity: &<TestGameLogic as GameLogic>::Entity,
    ) -> bool {
        let mut rng = entity.rng(engine.step_index());
        rng.random_bool(0.5)
    }

    fn step(
        &self,
        engine: &GameEngine<TestGameLogic>,
        entity: &<TestGameLogic as GameLogic>::Entity,
        next_entity: &mut <TestGameLogic as GameLogic>::Entity,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        let mut rng = entity.rng(engine.step_index());
        println!("stepping system {} id {}", engine.step_index(), entity.id());
        // x,y delta
        let delta = IVec2::new(rng.random_range(-10..10), rng.random_range(-10..10));
        next_entity.state_mut().position =
            (next_entity.state().position + delta).clamp(IVec2::ZERO, *engine.size());
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, EngineEntity)]
pub enum EngineEntity {
    Test(TestEntity),
}

#[derive(Clone, Debug, Serialize, Deserialize, EntitySystem)]
pub enum EngineEntitySystem {
    Test(TestSystem),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestGameLogic {}
impl GameLogic for TestGameLogic {
    type Entity = EngineEntity;
    type System = EngineEntitySystem;
    type Event = GameEvent;
    type Input = EntityInput;

    fn handle_game_events(
        engine: &mut GameEngine<Self>,
        game_events: &Vec<RefPointer<Self::Event>>,
    ) {
    }
}

#[test]
fn should_reset_id_counter() -> Result<()> {
    let mut engine = GameEngine::<TestGameLogic>::default();
    let mut ids_by_step = HashMap::new();
    // step a bunch of times
    for _ in 0..100 {
        let mut ids = vec![];
        // before each step spawn a bunch of entities that move randomly
        for _ in 0..10 {
            let id = engine.generate_id();
            ids.push(id);
            let entity = TestEntity::new(
                BaseEntityState {
                    id,
                    position: engine.size() / IVec2::splat(2),
                    ..Default::default()
                },
                vec![RefPointer::new(TestSystem.into())],
            );
            engine.spawn_entity(entity.into());
        }
        ids_by_step.insert(*engine.step_index(), ids);
        engine.step();
    }

    let mut engine = engine.engine_at_step(&50, false)?;

    for _ in 0..50 {
        let ids = ids_by_step.get(engine.step_index()).unwrap();
        for j in 0..10 {
            let id = ids.get(j).unwrap();
            assert_eq!(
                id,
                &engine.generate_id(),
                "id mismatch at step {} id {j}",
                engine.step_index()
            );
        }
        engine.step();
    }

    Ok(())
}

#[test]
fn should_rewind_replay() -> Result<()> {
    let mut engine = GameEngine::<TestGameLogic>::default();
    // step a bunch of times
    for _ in 0..100 {
        // before each step spawn a bunch of entities that move randomly
        println!(
            "step {} id {} entity count {}",
            engine.step_index(),
            engine.clone().generate_id(),
            engine.entities_at_step(engine.step_index()).len()
        );
        for _ in 0..10 {
            let entity = TestEntity::new(
                BaseEntityState {
                    id: engine.generate_id(),
                    position: engine.size() / IVec2::splat(2),
                    ..Default::default()
                },
                vec![RefPointer::new(TestSystem.into())],
            );
            engine.spawn_entity(entity.into());
        }
        engine.step();
    }

    // rewind and step again
    let mut old_engine = engine.engine_at_step(&50, true)?;
    for _ in 0..50 {
        println!(
            "step {} id {} entity count {}",
            old_engine.step_index(),
            old_engine.clone().generate_id(),
            old_engine.entities_at_step(old_engine.step_index()).len()
        );
        // before each step spawn a bunch of entities that move randomly
        for _ in 0..10 {
            let entity = TestEntity::new(
                BaseEntityState {
                    id: old_engine.generate_id(),
                    position: old_engine.size() / IVec2::splat(2),
                    ..Default::default()
                },
                vec![RefPointer::new(TestSystem.into())],
            );
            old_engine.spawn_entity(entity.into());
        }
        old_engine.step();
    }

    for i in 1..100 {
        if engine.step_hash(&i)? != old_engine.step_hash(&i)? {
            println!("{:?}", engine.entities_at_step(&i));
            println!("{:?}", old_engine.entities_at_step(&i));
            panic!("mismatch at step {i}");
        }
    }
    Ok(())
}
