/// In the entity we'll handle movement and announcements
/// Dialogue will be handled in non-universal engine events
use bevy_math::IVec2;
use rand::Rng;

use keind::prelude::*;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
    pub struct NpcEntity {
        pub data: NpcData,
        last_message_step: u64,
        last_announcement: usize,
    }
);

impl NpcEntity {
    pub fn new_data(id: u128, position: IVec2, data: NpcData) -> Self {
        let mut out = Self::new(
            BaseEntityState {
                id,
                position,
                size: data.size,
                ..Default::default()
            },
            vec![],
        );
        out.data = data;
        out
    }
}

impl SEEntity<KeindGameLogic> for NpcEntity {
    fn step(&self, engine: &GameEngine<KeindGameLogic>) -> Option<Self> {
        if self.data.announcements.is_empty() {
            return None;
        }
        let mut next_self = self.clone();
        let step_index = engine.step_index();
        let mut rng = self.rng(step_index);
        if &(self.last_message_step + 360) <= step_index && rng.random_bool(0.001) {
            let announcement_index = rng.random_range(0..self.data.announcements.len());
            engine.register_game_event(GameEvent::Message(
                self.id(),
                self.data.announcements[announcement_index].clone(),
            ));
            next_self.last_message_step = *step_index;
        }
        Some(next_self)
    }
}
