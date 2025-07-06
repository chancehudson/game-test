use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use super::deserialize_vec2;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NpcData {
    pub asset: String,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub position: IVec2,
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
    // messages the entity will say publicly
    #[serde(default)]
    pub announcements: Vec<String>,
    // messages the entity will say in 1:1 chat with player
    #[serde(default)]
    pub dialogue: Vec<String>,
}
