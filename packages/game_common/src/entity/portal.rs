use crate::entity_struct;

use super::EEntity;
use super::SEEntity;
use super::player::PlayerEntity;

entity_struct!(
    pub struct PortalEntity {
        // destination map name
        #[serde(skip)]
        pub from: String,
        pub to: String,
    }
);

impl PortalEntity {
    pub fn can_enter(&self, player: &PlayerEntity) -> bool {
        !player.rect().intersect(self.rect()).is_empty()
    }
}

impl SEEntity for PortalEntity {}
