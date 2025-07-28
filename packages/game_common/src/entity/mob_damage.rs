/// An entity that causes damage to a mob
/// on behalf of a player
use bevy_math::IVec2;

use db::Ability;

use crate::prelude::*;

crate::entity_struct!(
    pub struct MobDamageEntity {
        pub attached_to: u128,
        pub contacted_mob_id: Option<u128>,
        pub ability: Ability,
        pub has_despawned: bool,
    }
);

impl MobDamageEntity {
    pub fn new_with_entity(id: u128, entity: &EngineEntity, ability: Ability) -> Self {
        let mut out = Self::new(
            BaseEntityState {
                id,
                position: entity.position(),
                size: entity.size(),
                player_creator_id: entity.player_creator_id(),
                ..Default::default()
            },
            vec![Rc::new(EngineEntitySystem::from(AttachSystem {
                attached_to: entity.id(),
                offset: IVec2::ZERO,
            }))],
        );
        out.attached_to = entity.id();
        out.ability = ability;
        out
    }
}

impl SEEntity for MobDamageEntity {
    fn step(&self, engine: &GameEngine) -> Option<Self> {
        assert!(self.has_system::<AttachSystem>());
        if self.has_despawned || self.contacted_mob_id.is_some() {
            // despawn the mob damage entity
            let entity = engine
                .entity_by_id_untyped(&self.id(), None)
                .expect("mob_damage entity did not exist");
            engine.remove_entity(entity.id());
            return None;
        }
        let mut next_self = self.clone();
        if let Some(attached_entity) = engine.entity_by_id_untyped(&self.attached_to, None) {
            next_self.state.player_creator_id = attached_entity.player_creator_id();
            next_self.state.size = attached_entity.size();
            next_self.state.position = attached_entity.position();
            // handle contact with a mob
            for entity in engine.entities_by_type::<MobEntity>() {
                if !entity.rect().intersect(self.rect()).is_empty() {
                    if self.player_creator_id().is_none() {
                        println!("WARNING: mob damage entity has not player creator!");
                        continue;
                    }
                    next_self.contacted_mob_id = Some(entity.id());

                    // despawn whatever it's attached to
                    engine.remove_entity(entity.id());
                    break;
                }
            }
        } else {
            next_self.has_despawned = true;
        }
        Some(next_self)
    }
}
