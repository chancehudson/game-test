use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::mob_spawn::MobSpawnEntity;
use crate::engine::portal::PortalEntity;

// Custom deserializer for Vec2
fn deserialize_vec2<'de, D>(deserializer: D) -> Result<IVec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [i32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(IVec2::new(arr[0], arr[1]))
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Npc {
    pub asset: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: IVec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
}

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
    pub npc: Vec<Npc>,
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawnEntity>,
}
