use keind::prelude::*;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
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

impl SEEntity<KeindGameLogic> for PortalEntity {}
