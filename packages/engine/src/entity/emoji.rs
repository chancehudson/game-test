use bevy_math::IVec2;

use super::EEntity;
use super::SEEntity;

crate::entity_struct!(
    pub struct EmojiEntity {
        // entity id, relative position
        pub attached_to: Option<(u128, IVec2)>,
        pub disappears_at_step_index: u64,
    }
);

impl SEEntity for EmojiEntity {
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
                println!("WARNING: EmojiEntity attached to non-existent entity");
            }
        }
        if step_index >= self.disappears_at_step_index {
            engine.remove_entity(self.id, false);
        }
        next_self
    }
}
