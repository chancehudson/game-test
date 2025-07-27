use crate::entity_struct;

use crate::prelude::*;

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

#[typetag::serde]
impl SEEntity for PortalEntity {}
