use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::*;

pub mod item;
pub mod map;
pub mod mob;
pub mod npc;

use item::ItemData;
use map::MapData;
use mob::MobData;

/// Handles loading all game ascii data.
///
pub enum DataType {
    PLAYER,
    MOB,
    NPC,
    ITEM,
    MAP,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::PLAYER => "players",
            DataType::MOB => "mobs",
            DataType::NPC => "npc",
            DataType::ITEM => "items",
            DataType::MAP => "maps",
        }
    }
}

/// TODO: don't embed directly in the binary, load it via a bevy asset loader
#[cfg(target_arch = "wasm32")]
const GAME_DATA_STR: &'static str = include_str!("../../assets/game_data.json5");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameData {
    pub maps: HashMap<u64, MapData>,
    pub items: HashMap<u64, ItemData>,
    pub mobs: HashMap<u64, MobData>,
}

/// Write the code to parse and insert into hashmaps
macro_rules! convert_string_keys {
    ($raw_data:expr, $result:expr, $field:expr, $field_id:ident) => {
        if let Some(obj) = $raw_data.get($field).and_then(|v| v.as_array()) {
            for data in obj {
                let id = data.get("id").unwrap().clone().as_u64().unwrap();
                let data = serde_json::from_value(data.clone())?;
                $result.$field_id.insert(id, data);
            }
        } else {
            println!("Did not find data with key {}", $field);
        }
    };
}

impl GameData {
    pub fn from_json(data: Value) -> Result<Self> {
        // json5 keys are strings, we'll rewrite to integers
        let mut out = Self::default();
        convert_string_keys!(data, out, "maps", maps);
        convert_string_keys!(data, out, "items", items);
        convert_string_keys!(data, out, "mobs", mobs);
        Ok(out)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Result<Self> {
        let raw = json5::from_str(GAME_DATA_STR).unwrap();
        Self::from_json(raw)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(assets_dir: &Path) -> Result<Self> {
        let data_str = std::fs::read_to_string(&assets_dir.join("game_data.json5"))?;
        let raw = json5::from_str(&data_str)?;
        Self::from_json(raw)
    }
}
