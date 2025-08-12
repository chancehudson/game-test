use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

/// Only mutate velocity, not position.
/// Preserve upward momentum.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GravitySystem;

impl EEntitySystem<KeindGameLogic> for GravitySystem {
    fn prestep(
        &self,
        _engine: &GameEngine<KeindGameLogic>,
        _entity: &<KeindGameLogic as GameLogic>::Entity,
    ) -> bool {
        true
    }

    fn step(
        &self,
        engine: &GameEngine<KeindGameLogic>,
        entity: &<KeindGameLogic as GameLogic>::Entity,
        next_entity: &mut <KeindGameLogic as GameLogic>::Entity,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if actor::on_platform(entity.rect(), engine) || entity.has_system::<WeightlessSystem>() {
            // apply no gravity acceleration
            next_entity.state_mut().velocity.y = next_entity.state().velocity.y.max(0);
        } else {
            // not on a platform, not weightless, we accelerate down
            next_entity.state_mut().velocity.y -= 20;
        }
        Some(self.clone())
    }
}
