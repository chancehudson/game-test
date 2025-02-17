use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;
use serde::Deserialize;
use serde::Serialize;

// Custom deserializer for Vec2
fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [f32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(Vec2::new(arr[0], arr[1]))
}

#[derive(Clone, Debug, Deserialize)]
pub struct Portal {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    pub to: String,
}

impl Portal {
    pub fn center(&self) -> Vec2 {
        self.position - Vec2::new(50., 50.)
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y - 150., 150., 150.)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Platform {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: Vec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobSpawn {
    /// point at which we stop spawning
    pub max_count: usize,
    /// how quickly dead mobs respawn (mobs/second)
    // pub spawn_rate: f32,
    pub position: Vec2,
    pub size: Vec2,
    pub mob_type: u64,
    #[serde(default)]
    pub last_spawn: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MapData {
    pub name: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub spawn_location: Vec2,
    pub background: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
    pub portals: Vec<Portal>,
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub mob_spawns: Vec<MobSpawn>,
}
