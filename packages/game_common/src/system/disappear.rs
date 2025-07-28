use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisappearSystem {
    pub at_step: u64,
}

impl EEntitySystem for DisappearSystem {
    fn prestep(&self, engine: &GameEngine, entity: &EngineEntity) -> bool {
        if engine.step_index() == &self.at_step {
            engine.remove_entity(entity.id());
        }
        false
    }

    fn step(&self, _engine: &GameEngine, _entity: &mut EngineEntity) -> Option<Self> {
        None
    }
}
