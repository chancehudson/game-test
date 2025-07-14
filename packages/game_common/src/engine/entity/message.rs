use std::io::Write;

use bevy_math::IVec2;

use crate::entity::EEntity;
use crate::entity::SEEntity;
use crate::entity_struct;

const MESSAGE_WIDTH: i32 = 100;
const MESSAGE_TOP_PADDING: i32 = 10;

entity_struct!(
    pub struct MessageEntity {
        pub text: String,
        disappears_at_step: u64,
        pub creator_id: u128,
    }
);

/// Text centered at a point. Height to be determined by rendering impl
impl MessageEntity {
    pub fn new_text(
        text: String,
        step_index: u64,
        creator_id: u128,
        is_sender_player: bool,
    ) -> Self {
        // measure the size of the text?
        //
        let mut id_hasher = blake3::Hasher::new();
        id_hasher.write(&creator_id.to_le_bytes()).unwrap();
        id_hasher.write(&step_index.to_le_bytes()).unwrap();
        id_hasher.write(text.as_bytes()).unwrap();
        let mut id_bytes: [u8; 16] = [0; 16];
        id_bytes.copy_from_slice(&id_hasher.finalize().as_bytes().as_slice()[..16]);
        let id = u128::from_le_bytes(id_bytes);

        let mut out = Self::new(id, IVec2::MAX, IVec2::new(MESSAGE_WIDTH, 0));
        out.creator_id = creator_id;
        out.text = text;
        out.disappears_at_step = step_index + 180;
        if is_sender_player {
            out.player_creator_id = Some(creator_id);
        }
        out
    }
}

impl SEEntity for MessageEntity {
    fn step(&self, engine: &mut crate::GameEngine) -> Self {
        let mut next_self = self.clone();
        if engine.step_index >= self.disappears_at_step {
            engine.remove_entity(self.id, false);
            return next_self;
        }
        if let Some(entity) = engine.entities.get(&self.creator_id) {
            next_self.position = (entity.center()
                + IVec2::new(0, entity.size().y / 2 + MESSAGE_TOP_PADDING)
                - IVec2::new(MESSAGE_WIDTH / 2, 0))
            .clamp(IVec2::ZERO, engine.size - self.size());
        }
        for id in engine
            .entities_by_type::<MessageEntity>()
            .filter_map(|e| {
                if e.creator_id == self.creator_id && e.disappears_at_step < self.disappears_at_step
                {
                    Some(e.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
        {
            engine.remove_entity(id, false);
        }
        next_self
    }
}
