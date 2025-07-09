use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use super::npc::NpcData;
use crate::GameEngine;
use crate::deserialize_vec2;
use crate::entity::EngineEntity;
use crate::entity::mob_spawn::MobSpawnEntity;
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MapData {
    pub name: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub spawn_location: IVec2,
    pub background: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
    pub portals: Vec<PortalEntity>,
    pub npc: Vec<NpcData>,
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawnEntity>,
}

impl crate::EngineInit for MapData {
    fn init(&self, engine: &mut GameEngine) -> anyhow::Result<()> {
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
            let mut spawn_with_id = spawn.clone();
            spawn_with_id.id = engine.generate_id();
            engine.spawn_entity(EngineEntity::MobSpawner(spawn_with_id), None, true);
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
        Ok(())
    }
}
