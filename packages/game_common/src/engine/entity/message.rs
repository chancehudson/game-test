use bevy_math::IVec2;

use crate::entity::EEntity;
use crate::entity::SEEntity;
use crate::entity_struct;

entity_struct!(
    pub struct MessageEntity {
        pub text: String,
        disappears_at_step: u64,
    }
);

/// Text centered at a point. Height to be determined by rendering impl
impl MessageEntity {
    pub fn new_text(id: u128, position: IVec2, text: String) -> Self {
        // measure the size of the text?
        //
        let mut out = Self::new(id, position, IVec2::ZERO);
        out.text = text;
        out
    }
}

impl SEEntity for MessageEntity {
    fn step(&self, _engine: &mut crate::GameEngine) -> Self {
        self.clone()
    }
}
