/// In the entity we'll handle movement and announcements
/// Dialogue will be handled in non-universal engine events
use bevy_math::IVec2;
use rand::Rng;

use crate::entity::EEntity;
use crate::entity::SEEntity;
use crate::entity::mob::MobEntity;
use crate::entity_struct;

entity_struct!(
    pub struct NpcEntity {
        npc_id: u64,
        announcements: Vec<String>,
        last_message_step: Option<u64>,
    }
);

impl SEEntity for NpcEntity {
    fn step(&self, _engine: &mut crate::GameEngine) -> Self {
        if self.announcements.is_empty() {
            return self.clone();
        }
        self.clone()
    }
}
