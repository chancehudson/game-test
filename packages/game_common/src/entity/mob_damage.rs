/// An entity that causes damage to a mob
/// on behalf of a player
use bevy_math::Vec3;

use db::Ability;

use crate::prelude::*;

crate::entity_struct!(
    pub struct MobDamageEntity {
        pub attached_to: u128,
        pub color: Vec3,
        pub contacted_mob_id: Option<u128>,
        pub ability: Ability,
        pub has_despawned: bool,
    }
);

impl MobDamageEntity {
    pub fn new_with_entity(id: u128, entity: &EngineEntity, ability: Ability) -> Self {
        let mut out = Self::new(id, entity.position(), entity.size());
        out.attached_to = entity.id();
        out.player_creator_id = entity.player_creator_id();
        out.ability = ability;
        out
    }
}

impl SEEntity for MobDamageEntity {
    fn step<T: super::GameEngine>(&self, engine: &T) -> Self
    where
        Self: Sized + Clone,
    {
        if self.has_despawned || self.contacted_mob_id.is_some() {
            // despawn the mob damage entity
            engine.register_event(
                None,
                EngineEvent::RemoveEntity {
                    entity_id: self.id,
                    universal: false,
                },
            );
            return self.clone();
        }
        let mut next_self =
            if let Some(attached_entity) = engine.entity_by_id_untyped(&self.attached_to, None) {
                let mut next_self = self.clone();
                next_self.player_creator_id = attached_entity.player_creator_id();
                next_self.size = attached_entity.size();
                next_self.position = attached_entity.position();
                next_self
            } else {
                let mut next_self = self.clone();
                next_self.has_despawned = true;
                next_self
            };

        // handle contact with a mob
        for entity in engine.entities_by_type::<MobEntity>().collect::<Vec<_>>() {
            if !entity.rect().intersect(self.rect()).is_empty() {
                if self.player_creator_id().is_none() {
                    println!("WARNING: mob damage entity has not player creator!");
                    continue;
                }
                next_self.contacted_mob_id = Some(entity.id);

                // despawn whatever it's attached to
                engine.register_event(
                    None,
                    EngineEvent::RemoveEntity {
                        entity_id: self.attached_to,
                        universal: false,
                    },
                );
                break;
            }
        }
        next_self
    }
}
