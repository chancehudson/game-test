use std::io::Write;

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
    pub fn new_text(
        position: IVec2,
        text: String,
        step_index: u64,
        player_creator_id: u128,
    ) -> Self {
        // measure the size of the text?
        //
        let mut id_hasher = blake3::Hasher::new();
        id_hasher.write(&player_creator_id.to_le_bytes()).unwrap();
        id_hasher.write(&step_index.to_le_bytes()).unwrap();
        id_hasher.write(text.as_bytes()).unwrap();
        let mut id_bytes: [u8; 16] = [0; 16];
        id_bytes.copy_from_slice(&id_hasher.finalize().as_bytes().as_slice()[..16]);
        let id = u128::from_le_bytes(id_bytes);

        let mut out = Self::new(id, position, IVec2::ZERO);
        out.text = text;
        out.disappears_at_step = step_index + 90;
        out.player_creator_id = Some(player_creator_id);
        out
    }
}

impl SEEntity for MessageEntity {
    fn step(&self, engine: &mut crate::GameEngine) -> Self {
        if engine.step_index >= self.disappears_at_step {
            engine.remove_entity(self.id, false);
        }
        self.clone()
    }
}
