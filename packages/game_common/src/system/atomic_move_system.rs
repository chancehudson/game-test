use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

/// Move an entity over a single step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtomicMoveSystem {}

impl EEntitySystem<KeindGameLogic> for AtomicMoveSystem {
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
        let lower_speed_limit = IVec2::new(-250, -350);
        let upper_speed_limit = IVec2::new(250, 700);

        // clamp next velocity
        next_entity.state_mut().velocity = next_entity
            .velocity()
            .clamp(lower_speed_limit, upper_speed_limit);

        let velocity = entity.velocity();

        let body = entity.rect();
        // approximate displacement using integer math
        let disp = IVec2::new(
            velocity.x / STEPS_PER_SECOND as i32,
            velocity.y / STEPS_PER_SECOND as i32,
        );
        next_entity.state_mut().position.x = actor::move_x(body, disp.x, engine);
        next_entity.state_mut().position.y = actor::move_y(
            body,
            disp.y,
            &engine.entities_by_type::<PlatformEntity>(),
            engine.size,
        );
        Some(self.clone())
    }
}
