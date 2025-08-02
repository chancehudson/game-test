use bevy_math::Vec3;

use keind::prelude::*;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
    pub struct RectEntity {
        pub color: Vec3,
    }
);

impl SEEntity<KeindGameLogic> for RectEntity {}
