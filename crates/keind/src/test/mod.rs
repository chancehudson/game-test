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
pub struct TestSystem {}

impl EEntitySystem<TestGameLogic> for TestSystem {
    fn prestep(
        &self,
        _engine: &GameEngine<TestGameLogic>,
        _entity: &<TestGameLogic as GameLogic>::Entity,
    ) -> bool {
        false
    }

    fn step(
        &self,
        _engine: &GameEngine<TestGameLogic>,
        _entity: &<TestGameLogic as GameLogic>::Entity,
        _next_entity: &mut <TestGameLogic as GameLogic>::Entity,
    ) -> Option<Self>
    where
        Self: Sized,
    {
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

    fn handle_game_events(engine: &GameEngine<Self>, game_events: &Vec<RefPointer<Self::Event>>) {}
}

#[test]
fn initialize_engine() {}
