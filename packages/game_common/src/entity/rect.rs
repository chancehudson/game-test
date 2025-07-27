use bevy_math::Vec3;

use crate::prelude::*;

entity_struct!(
    pub struct RectEntity {
        pub disappears_at_step_index: Option<u64>,
        pub color: Vec3,
    }
);

#[typetag::serde]
impl SEEntity for RectEntity {
    fn step(&self, engine: &GameEngine) -> Option<Box<dyn SEEntity>> {
        let step_index = engine.step_index();
        if let Some(disappear_step) = self.disappears_at_step_index {
            if step_index >= &disappear_step {
                let entity = engine
                    .entity_by_id_untyped(&self.id(), None)
                    .expect("rect entity did not exist");
                engine.remove_entity(entity);
                return None;
            }
        }
        let mut next_self = self.clone();
        next_self.state.position.x = actor::move_x(
            self.rect(),
            self.state.velocity.x / STEPS_PER_SECOND_I32,
            engine,
        );
        Some(Box::new(next_self))
    }
}
