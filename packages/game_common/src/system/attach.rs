use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachSystem {
    pub attached_to: u128,
    pub offset: IVec2,
}

impl EEntitySystem for AttachSystem {
    fn prestep(&self, engine: &GameEngine, entity: &EngineEntity) -> bool {
        // only allow one of these per entity
        assert_eq!(
            entity
                .systems()
                .iter()
                .filter_map(|system| (system.clone() as Rc<dyn Any>).downcast::<Self>().ok())
                .collect::<Vec<_>>()
                .len(),
            1,
            "multiple attach systems"
        );

        // TODO: assert no duplicate system on entity
        // check if entity positions are equal
        // if yes don't step
        if let Some(entity_0) = engine.entity_by_id_untyped(&self.attached_to, None)
            && let Some(entity_1) = engine.entity_by_id_untyped(&entity.id(), None)
        {
            entity_0.state().position != entity_1.state().position
        } else {
            false
        }
    }

    fn step(&self, engine: &GameEngine, entity: &mut EngineEntity) -> Option<Self> {
        if let Some(attached_entity) = engine.entity_by_id_untyped(&self.attached_to, None) {
            entity.state_mut().position = attached_entity.state().position + self.offset
        } else {
            unreachable!("entities changed existence during step");
        }
        None
    }
}
