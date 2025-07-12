use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::AnimationData;
use crate::deserialize_vec2;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct NpcData {
    #[serde(deserialize_with = "deserialize_vec2")]
    pub size: IVec2,
    // messages the entity will say publicly
    #[serde(default)]
    pub announcements: Vec<String>,
    // messages the entity will say in 1:1 chat with player
    #[serde(default)]
    pub dialogue: Vec<String>,
    pub standing_animation: AnimationData,
}
