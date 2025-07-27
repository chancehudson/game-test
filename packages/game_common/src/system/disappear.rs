use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct DisappearSystem {
    pub at_step: u64,
}

#[typetag::serde]
impl EEntitySystem for DisappearSystem {
    fn prestep(&self, engine: &GameEngine, entity: &Rc<dyn SEEntity>) -> bool {
        if engine.step_index() == &self.at_step {
            engine.remove_entity(entity.clone());
        }
        false
    }

    fn step(
        &self,
        _engine: &GameEngine,
        _entity: &mut dyn SEEntity,
    ) -> Option<Box<dyn EEntitySystem>> {
        None
    }

    fn clone_box(&self) -> Box<dyn EEntitySystem> {
        Box::new(self.clone())
    }
}
