use std::collections::BTreeSet;

use bevy_math::IVec2;
use keind::prelude::*;
use rand::Rng;

use crate::prelude::*;

entity_struct!(
    KeindGameLogic,
    pub struct MobSpawnEntity {
        pub spawn_data: MobSpawnData,
        pub drop_table: Vec<DropTableData>,
        pub last_spawn_step: u64,
        owned_mob_ids: BTreeSet<u128>,
    }
);

impl MobSpawnEntity {
    pub fn new_data(id: u128, spawn_data: MobSpawnData, drop_table: Vec<DropTableData>) -> Self {
        let mut out = Self::new(
            BaseEntityState {
                id,
                position: spawn_data.position,
                size: spawn_data.size,
                ..Default::default()
            },
            vec![],
        );
        out.spawn_data = spawn_data;
        out.drop_table = drop_table;
        out
    }
}

impl SEEntity<KeindGameLogic> for MobSpawnEntity {
    fn prestep(&self, _engine: &GameEngine<KeindGameLogic>) -> bool {
        true
    }

    fn step(&self, engine: &GameEngine<KeindGameLogic>, next_self: &mut Self) {
        let step_index = engine.step_index();
        let current_spawn_count = self.owned_mob_ids.len();
        for id in &self.owned_mob_ids {
            if !engine.entity_by_id_untyped(id, None).is_some() {
                next_self.owned_mob_ids.remove(id);
            }
        }

        if current_spawn_count >= self.spawn_data.max_count {
            return;
        }
        if step_index - self.last_spawn_step < 10 {
            return;
        }
        let mut rng = self.rng(&step_index);
        let max_spawn_count = self.spawn_data.max_count - current_spawn_count;
        let spawn_count = rng.random_range(0..=max_spawn_count);
        for _ in 0..spawn_count {
            // deterministically generate future mob ids
            let id = rng.random();
            next_self.owned_mob_ids.insert(id);
            let mut mob_entity = MobEntity::default();
            mob_entity.drop_table = self.drop_table.clone();
            mob_entity.current_health = 10;
            mob_entity.state.id = id;
            mob_entity.state.position = IVec2::new(
                rng.random_range(self.position().x..self.position().x + self.size().x),
                rng.random_range(self.position().y..self.position().y + self.size().y),
            );
            mob_entity.state.size = IVec2::new(37, 62);
            mob_entity.mob_type = self.spawn_data.mob_type;
            engine.spawn_entity(RefPointer::new(mob_entity.into()));
        }
        next_self.last_spawn_step = *step_index;
    }
}
