/// In the entity we'll handle movement and announcements
/// Dialogue will be handled in non-universal engine events
use bevy_math::IVec2;

use crate::data::npc::NpcData;
use crate::entity::EEntity;
use crate::entity::SEEntity;
use crate::entity_struct;

entity_struct!(
    pub struct NpcEntity {
        pub data: NpcData,
        last_message_step: Option<u64>,
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
    fn step(&self, _engine: &mut crate::GameEngine) -> Self {
        if self.data.announcements.is_empty() {
            return self.clone();
        }
        self.clone()
    }
}
