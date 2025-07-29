use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisappearSystem {
    pub at_step: u64,
}

impl EEntitySystem<KeindGameLogic> for DisappearSystem {
    fn prestep(&self, engine: &GameEngine<KeindGameLogic>, entity: &EngineEntity) -> bool {
        if engine.step_index() == &self.at_step {
            engine.remove_entity(entity.id());
        }
        false
    }

    fn step(
        &self,
        _engine: &GameEngine<KeindGameLogic>,
        _entity: &mut EngineEntity,
    ) -> Option<Self> {
        None
    }
}
