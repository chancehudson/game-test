use bevy_math::Vec3;

use keind::prelude::*;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
    pub struct RectEntity {
        pub disappears_at_step_index: Option<u64>,
        pub color: Vec3,
    }
);

impl SEEntity<KeindGameLogic> for RectEntity {
    fn step(&self, engine: &GameEngine<KeindGameLogic>) -> Option<Self> {
        let step_index = engine.step_index();
        if let Some(disappear_step) = self.disappears_at_step_index {
            if step_index >= &disappear_step {
                let entity = engine
                    .entity_by_id_untyped(&self.id(), None)
                    .expect("rect entity did not exist");
                engine.remove_entity(entity.id());
                return None;
            }
        }
        let mut next_self = self.clone();
        next_self.state.position.x = actor::move_x(
            self.rect(),
            self.state.velocity.x / STEPS_PER_SECOND as i32,
            engine,
        );
        Some(next_self)
    }
}
