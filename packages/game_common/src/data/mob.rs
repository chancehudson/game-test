use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::AnimationData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDropData {
    pub item_id: u64,
    pub odds: f32,
    // the minimum and maximum number of items that should be dropped
    // e.g. for dropping gold or other stackable items
    pub range: Option<[u8; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobData {
    pub id: u64,
    pub name: String,
    pub size: IVec2,
    pub walking_animation: AnimationData,
    pub standing_animation: AnimationData,
    pub drop_table: Vec<MobDropData>,
}
