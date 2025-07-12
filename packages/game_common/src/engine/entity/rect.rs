use bevy_math::Vec3;

use crate::STEPS_PER_SECOND_I32;
use crate::actor::move_x;

use super::EEntity;
use super::SEEntity;

crate::entity_struct!(
    pub struct RectEntity {
        pub disappears_at_step_index: Option<u64>,
        pub color: Vec3,
    }
);

impl SEEntity for RectEntity {
    fn step(&self, engine: &mut super::GameEngine) -> Self
    where
        Self: Sized + Clone,
    {
        let step_index = engine.step_index;
        if let Some(disappear_step) = self.disappears_at_step_index {
            if step_index >= disappear_step {
                engine.remove_entity(self.id, false);
            }
        }
        let mut next_self = self.clone();
        next_self.position.x = move_x(self.rect(), self.velocity.x / STEPS_PER_SECOND_I32, engine);
        next_self
    }
}
