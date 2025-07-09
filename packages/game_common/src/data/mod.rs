use std::collections::HashMap;

pub mod item;
pub mod map;
pub mod mob;
pub mod npc;

use item::ItemData;
use map::MapData;
use mob::MobData;

/// Handles loading all game ascii data.
///

#[derive(Clone, Debug, Default)]
pub struct GameData {
    pub maps: HashMap<String, MapData>,
    pub items: HashMap<u64, ItemData>,
    pub mobs: HashMap<u64, MobData>,
}

impl GameData {
    pub fn init() -> Self {
        Self::default()
    }
}
