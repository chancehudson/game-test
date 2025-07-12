use serde::Deserialize;
use serde::Serialize;

use crate::AnimationData;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemData {
    pub id: u64,
    pub name: String,
    pub icon_animation: AnimationData,
}
