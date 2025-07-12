use bevy_math::IVec2;
use serde::Deserialize;
use serde::Serialize;

use crate::AnimationData;
use crate::data::map::DropTableData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobData {
    pub id: u64,
    pub name: String,
    pub size: IVec2,
    pub walking_animation: AnimationData,
    pub standing_animation: AnimationData,
    pub drop_table: Vec<DropTableData>,
}
