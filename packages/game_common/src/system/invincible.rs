use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

/// Move an entity over a single step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvincibleSystem {
    pub until_step: Option<u64>,
}

impl EEntitySystem<KeindGameLogic> for InvincibleSystem {
    fn prestep(
        &self,
        engine: &GameEngine<KeindGameLogic>,
        _entity: &<KeindGameLogic as GameLogic>::Entity,
    ) -> bool {
        engine.step_index() >= &self.until_step.unwrap_or(u64::MAX)
    }
}
