use bevy_math::IVec2;
use bevy_math::Vec3;

use crate::prelude::*;

entity_struct!(
    pub struct TextEntity {
        // entity id, relative position
        pub attached_to: Option<(u128, IVec2)>,
        pub disappears_at_step_index: u64,
        pub text: String,
        pub font_size: f32,
        // srgb
        pub color: Vec3,
    }
);

#[typetag::serde]
impl SEEntity for TextEntity {
    fn step(&self, engine: &GameEngine) -> Option<Box<dyn SEEntity>> {
        let step_index = engine.step_index();
        if step_index >= &self.disappears_at_step_index {
            let entity = engine
                .entity_by_id_untyped(&self.id(), None)
                .expect("text entity did not exist");
            engine.remove_entity(entity);
        }
        let mut next_self = self.clone();
        if let Some((attached_id, relative_pos)) = self.attached_to {
            if let Some(entity) = engine.entity_by_id_untyped(&attached_id, None) {
                next_self.state.position = entity.position() + relative_pos;
            } else {
                println!("WARNING: TextEntity attached to non-existent entity");
            }
        }
        Some(Box::new(next_self))
    }
}
