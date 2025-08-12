use serde::Deserialize;
use serde::Serialize;

use keind::prelude::*;

use crate::prelude::*;

/// Move an entity over a single step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtomicMoveSystem {
    /// lower and upper speed limit
    speed_limit: Option<(IVec2, IVec2)>,
}

impl AtomicMoveSystem {
    pub fn new_with_speed_limit(
        lower_x_maybe: Option<i32>,
        lower_y_maybe: Option<i32>,
        upper_x_maybe: Option<i32>,
        upper_y_maybe: Option<i32>,
    ) -> Self {
        let (default_lower, default_upper) = Self::default_speed_limit();
        Self {
            speed_limit: Some((
                IVec2::new(
                    lower_x_maybe.unwrap_or(default_lower.x),
                    lower_y_maybe.unwrap_or(default_lower.y),
                ),
                IVec2::new(
                    upper_x_maybe.unwrap_or(default_upper.x),
                    upper_y_maybe.unwrap_or(default_upper.y),
                ),
            )),
        }
    }

    pub fn default_speed_limit() -> (IVec2, IVec2) {
        (IVec2::new(-250, -350), IVec2::new(250, 700))
    }
}

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
        let (lower_speed_limit, upper_speed_limit) =
            self.speed_limit.unwrap_or(Self::default_speed_limit());

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
            engine.size(),
        );
        Some(self.clone())
    }
}
