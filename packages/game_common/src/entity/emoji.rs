use keind::prelude::*;

use crate::prelude::*;

entity_struct!(KeindGameLogic, pub struct EmojiEntity {});

impl SEEntity<KeindGameLogic> for EmojiEntity {
    fn prestep(&self, _engine: &GameEngine<KeindGameLogic>) -> bool {
        assert!(self.has_system::<AttachSystem>());
        assert!(self.has_system::<DisappearSystem>());
        false
    }
}
