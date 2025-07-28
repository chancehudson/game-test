use crate::prelude::*;

entity_struct!(
    pub struct EmojiEntity {}
);

impl SEEntity for EmojiEntity {
    fn step(&self, _engine: &GameEngine) -> Option<Self> {
        assert!(self.has_system::<AttachSystem>());
        assert!(self.has_system::<DisappearSystem>());
        None
    }
}
