use crate::entity_struct;

use super::player::PlayerEntity;
use super::EEntity;
use super::SEEntity;

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
