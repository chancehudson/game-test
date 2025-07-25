/// In the entity we'll handle movement and announcements
/// Dialogue will be handled in non-universal engine events
use bevy_math::IVec2;
use rand::Rng;

use crate::prelude::*;

entity_struct!(
    pub struct NpcEntity {
        pub data: NpcData,
        last_message_step: u64,
        last_announcement: usize,
    }
);

impl NpcEntity {
    pub fn new_data(id: u128, position: IVec2, data: NpcData) -> Self {
        let mut out = Self::new(id, position, data.size);
        out.data = data;
        out
    }
}

impl SEEntity for NpcEntity {
    fn step<T: GameEngine>(&self, engine: &T) -> Self {
        if self.data.announcements.is_empty() {
            return self.clone();
        }
        let mut next_self = self.clone();
        let step_index = engine.step_index();
        let mut rng = self.rng(step_index);
        if &(self.last_message_step + 360) <= step_index && rng.random_bool(0.001) {
            let announcement_index = rng.random_range(0..self.data.announcements.len());
            engine.register_event(
                None,
                EngineEvent::Message {
                    text: self.data.announcements[announcement_index].clone(),
                    entity_id: self.id,
                    entity_type_id: entity_type_ids::Npc,
                    universal: false,
                },
            );
            next_self.last_message_step = *step_index;
        }
        next_self
    }
}
