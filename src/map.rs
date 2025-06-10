use bevy_math::Rect;
use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::mob_spawner::MobSpawnEntity;
use crate::engine::portal::PortalEntity;

// Custom deserializer for Vec2
fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [f32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(Vec2::new(arr[0], arr[1]))
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Npc {
    pub asset: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Platform {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
}

impl Platform {
    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.position.x + self.size.x,
            self.position.y + self.size.y,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MapData {
    pub name: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub spawn_location: Vec2,
    pub background: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
    pub portals: Vec<PortalEntity>,
    pub npc: Vec<Npc>,
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawnEntity>,
}

impl MapData {
    pub fn contains_platform(&self, rect: Rect) -> bool {
        for platform in &self.platforms {
            let intersection = rect.intersect(platform.rect());
            if intersection.width() > 2.0 && intersection.height() >= 1.0 {
                return true;
            }
        }
        false
    }

    pub fn not_contains_platform(&self, rect: Rect) -> bool {
        for platform in &self.platforms {
            if !rect.intersect(platform.rect()).is_empty() {
                return false;
            }
        }
        true
    }
}
