use bevy_math::IVec2;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use crate::GameEngine;
use crate::data::GameData;
use crate::deserialize_vec2;
use crate::entity::EngineEntity;
use crate::entity::mob_spawn::MobSpawnEntity;
use crate::entity::npc::NpcEntity;
use crate::entity::platform::PlatformEntity;
use crate::entity::portal::PortalEntity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Platform {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: IVec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
}

impl Platform {}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct MobSpawnData {
    pub position: IVec2,
    pub size: IVec2,
    pub mob_type: u64,
    pub max_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DropTableData {
    pub item_id: u64,
    pub odds: f32,
    pub count_range: (u32, u32),
}

impl DropTableData {
    pub fn drop<R: Rng>(&self, rng: &mut R) -> Option<(u64, u32)> {
        if rng.random_bool(self.odds as f64) {
            let count = rng.random_range(self.count_range.0..=self.count_range.1);
            Some((self.item_id, count))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapNpcData {
    pub npc_id: u64,
    pub position: IVec2,
    #[serde(default)]
    pub announcements: Vec<String>, // overrides at the map level
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MapData {
    pub id: u64,
    pub name: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub spawn_location: IVec2,
    pub background: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
    pub portals: Vec<PortalEntity>,
    pub npc: Vec<MapNpcData>,
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawnData>,
}

impl crate::EngineInit for MapData {
    fn init(&self, game_data: &GameData, engine: &mut GameEngine) -> anyhow::Result<()> {
        // spawn the map components as needed
        for platform in &self.platforms {
            let id = engine.generate_id();
            engine.spawn_entity(
                EngineEntity::Platform(PlatformEntity::new(
                    id,
                    platform.position.clone(),
                    platform.size.clone(),
                )),
                None,
                true,
            );
        }
        // mob spawns
        for spawn in &self.mob_spawns {
            let drop_table = game_data.mob_drop_table(spawn.mob_type)?;
            let id = engine.generate_id();
            engine.spawn_entity(
                EngineEntity::MobSpawner(MobSpawnEntity::new_data(id, spawn.clone(), drop_table)),
                None,
                true,
            );
        }
        // portal spawns
        for portal in &self.portals {
            let id = engine.generate_id();
            let mut portal_clone = portal.clone();
            if portal_clone.size.x == 0 {
                portal_clone.size = IVec2::new(60, 60);
            }
            portal_clone.id = id;
            portal_clone.from = self.name.clone();
            engine.spawn_entity(EngineEntity::Portal(portal_clone), None, true);
        }

        for map_npc_data in &self.npc {
            let mut npc = game_data
                .npc
                .get(&map_npc_data.npc_id)
                .expect("Invalid npc_id in MapNpcData")
                .clone();
            npc.announcements
                .append(&mut map_npc_data.announcements.clone());
            let id = engine.generate_id();
            engine.spawn_entity(
                EngineEntity::Npc(NpcEntity::new_data(id, map_npc_data.position, npc)),
                None,
                true,
            );
        }
        Ok(())
    }
}
