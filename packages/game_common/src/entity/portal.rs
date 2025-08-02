use bevy_math::IVec2;

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
    pub fn new_data(id: u128, map_data: &MapData, portal_data: &PortalData) -> Self {
        Self {
            from: map_data.name.clone(),
            to: portal_data.to.clone(),
            state: BaseEntityState {
                id,
                size: IVec2::new(60, 60),
                position: portal_data.position,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl PortalEntity {
    pub fn can_enter(&self, player: &PlayerEntity) -> bool {
        !player.rect().intersect(self.rect()).is_empty()
    }
}

impl SEEntity<KeindGameLogic> for PortalEntity {
    fn prestep(&self, _engine: &GameEngine<KeindGameLogic>) -> bool {
        false
    }
}
