use keind::prelude::*;

use crate::prelude::*;

entity_struct!(KeindGameLogic, pub struct EmojiEntity {});

impl SEEntity<KeindGameLogic> for EmojiEntity {
    fn step(&self, _engine: &GameEngine<KeindGameLogic>) -> Option<Self> {
        assert!(self.has_system::<AttachSystem>());
        assert!(self.has_system::<DisappearSystem>());
        None
    }
}
