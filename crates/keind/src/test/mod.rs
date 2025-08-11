use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

engine_entity_enum!(TestGameLogic, pub enum EngineEntity {});

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
