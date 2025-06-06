use std::collections::HashMap;
use std::collections::HashSet;

use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use super::entity::EngineEntity;
use super::entity::Entity;
use super::entity::EntityInput;
use crate::engine::mob::MobEntity;
use crate::engine::GameEngine;
use crate::timestamp;
use crate::MapData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobSpawnEntity {
    #[serde(skip)]
    pub id: u128,
    /// point at which we stop spawning
    pub max_count: usize,
    /// how quickly dead mobs respawn (mobs/second)
    // pub spawn_rate: f32,
    pub position: Vec2,
    pub size: Vec2,
    pub mob_type: u64,
    #[serde(default)]
    pub last_spawn: f64,
    #[serde(skip)]
    owned_mob_ids: HashSet<u128>,
}

impl MobSpawnEntity {}

impl Entity for MobSpawnEntity {
    fn id(&self) -> u128 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
        let mut next_self = self.clone();
        if !cfg!(feature = "server") {
            return next_self;
        }
        for id in &self.owned_mob_ids {
            if !engine.entities.contains_key(id) {
                next_self.owned_mob_ids.remove(id);
            }
        }

        if next_self.owned_mob_ids.len() >= self.max_count {
            return next_self;
        }
        if timestamp() - self.last_spawn < 10.0 {
            return next_self;
        }
        let spawn_count = rand::random_range(0..=self.max_count);
        for _ in 0..spawn_count {
            let id = engine.generate_id();
            next_self.owned_mob_ids.insert(id);
            let mut mob_entity = MobEntity::default();
            mob_entity.id = id;
            mob_entity.position = Vec2::new(
                rand::random_range(self.position.x..self.position.x + self.size.x),
                rand::random_range(self.position.y..self.position.y + self.size.y),
            );
            mob_entity.size = Vec2::new(37., 37.);
            mob_entity.mob_type = self.mob_type;
            engine.spawn_entity(EngineEntity::Mob(mob_entity));
        }
        next_self.last_spawn = timestamp();
        next_self
    }
}
