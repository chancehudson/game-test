use std::collections::BTreeSet;

use bevy_math::IVec2;
use rand::Rng;

use super::EEntity;
use super::EngineEntity;
use super::SEEntity;
use super::mob::MobEntity;
use crate::TICK_RATE_S;
use crate::engine::GameEngine;
use crate::entity_struct;

entity_struct!(
    pub struct MobSpawnEntity {
        /// point at which we stop spawning
        pub max_count: usize,
        /// how quickly dead mobs respawn (mobs/second)
        // pub spawn_rate: f32,
        pub mob_type: u64,
        #[serde(default)]
        pub last_spawn_step: u64,
        #[serde(default)]
        owned_mob_ids: BTreeSet<u128>,
    }
);

impl SEEntity for MobSpawnEntity {
    fn step(&self, engine: &mut GameEngine) -> Self {
        let step_index = engine.step_index;
        let mut next_self = self.clone();
        let current_spawn_count = self.owned_mob_ids.len();
        for id in &self.owned_mob_ids {
            if !engine.entities.contains_key(id) {
                next_self.owned_mob_ids.remove(id);
            }
        }

        if current_spawn_count >= self.max_count {
            return next_self;
        }
        if step_index - self.last_spawn_step < 10 * ((TICK_RATE_S).ceil() as u64) {
            return next_self;
        }
        let mut rng = self.rng(&step_index);
        let max_spawn_count = self.max_count - current_spawn_count;
        let spawn_count = rng.random_range(0..=max_spawn_count);
        for _ in 0..spawn_count {
            // deterministically generate future mob ids
            // absolutely disgusting
            let id = rng.random();
            next_self.owned_mob_ids.insert(id);
            let mut mob_entity = MobEntity::default();
            mob_entity.current_health = 10;
            mob_entity.id = id;
            mob_entity.position = IVec2::new(
                rng.random_range(self.position.x..self.position.x + self.size.x),
                rng.random_range(self.position.y..self.position.y + self.size.y),
            );
            mob_entity.size = IVec2::new(37, 62);
            mob_entity.mob_type = self.mob_type;
            engine.spawn_entity(EngineEntity::Mob(mob_entity), None, false);
        }
        next_self.last_spawn_step = step_index;
        next_self
    }
}
