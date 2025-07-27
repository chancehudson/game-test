use crate::prelude::*;

entity_struct!(
    pub struct EmojiEntity {}
);

#[typetag::serde]
impl SEEntity for EmojiEntity {
    fn step(&self, _engine: &GameEngine) -> Option<Box<dyn SEEntity>> {
        assert!(self.has_system::<AttachSystem>());
        assert!(self.has_system::<DisappearSystem>());
        None
    }
}
