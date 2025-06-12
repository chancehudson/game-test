use crate::engine::entity::SEEntity;
use crate::engine::player::PlayerEntity;
use crate::entity_struct;

use super::entity::EEntity;

entity_struct!(
    pub struct PortalEntity {
        // destination map name
        pub to: String,
    }
);

impl PortalEntity {
    pub fn can_enter(&self, player: &PlayerEntity) -> bool {
        !player.rect().intersect(self.rect()).is_empty()
    }
}

impl SEEntity for PortalEntity {}
