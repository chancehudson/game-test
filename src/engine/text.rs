use bevy_math::IVec2;
use bevy_math::Vec3;

use super::entity::EEntity;
use super::entity::SEEntity;

crate::entity_struct!(
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

impl SEEntity for TextEntity {
    fn step(&self, engine: &mut super::GameEngine) -> Self
    where
        Self: Sized + Clone,
    {
        let step_index = engine.step_index;
        let mut next_self = self.clone();
        if let Some((attached_id, relative_pos)) = self.attached_to {
            if let Some(entity) = engine.entities.get(&attached_id) {
                next_self.position = entity.position() + relative_pos;
            } else {
                println!("WARNING: TextEntity attached to non-existent entity");
            }
        }
        if step_index >= self.disappears_at_step_index {
            engine.remove_entity(self.id, false);
        }
        next_self
    }
}
