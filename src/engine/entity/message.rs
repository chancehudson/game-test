use bevy_math::IVec2;

use super::EEntity;
use super::SEEntity;

crate::entity_struct!(
    pub struct MessageEntity {
        // entity id, relative position
        pub attached_to: Option<(u128, IVec2)>,
        pub disappears_at: u64,
        pub text: String,
    }
);

impl MessageEntity {
    pub fn text(id: u128, text: String, speaker_entity: u128, disappears_at: u64) -> Self {
        Self {
            attached_to: Some((speaker_entity, IVec2::ZERO)),
            text,
            disappears_at,
            ..Self::new(id, IVec2::MAX, IVec2::new(100, 100))
        }
    }
}

impl SEEntity for MessageEntity {
    fn step(&self, engine: &mut super::GameEngine) -> Self
    where
        Self: Sized + Clone,
    {
        let step_index = engine.step_index;
        let mut next_self = self.clone();
        if let Some((attached_id, relative_pos)) = self.attached_to {
            if let Some(entity) = engine.entities.get(&attached_id) {
                next_self.position = entity.position()
                    + IVec2::new(entity.size().x / 2, entity.size().y)
                    + relative_pos;
            } else {
                println!("WARNING: MessageEntity attached to non-existent entity");
            }
        }
        if step_index >= self.disappears_at {
            engine.remove_entity(self.id, false);
        }
        next_self
    }
}
