use std::collections::HashMap;
use std::fs::DirEntry;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

use game_test::MapData;

use crate::map_instance::MapInstance;

pub struct GlobalState {
    pub map_instances: HashMap<String, MapInstance>
}

impl GlobalState {
    pub async fn new() -> anyhow::Result<Self> {
        let mut map_instances = HashMap::new();
        println!("Loading maps...");
        let maps_dir = fs::read_dir("maps")?;
        for entry in maps_dir {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if let Some(extension) = path.extension() {
                if extension != "json5" {
                    continue;
                }
                let name = path.file_stem().unwrap().to_str().unwrap();
                if let Some(_file_name) = entry.file_name().to_str() {
                    let data_str = fs::read_to_string(path_str).unwrap();
                    let data = json5::from_str::<MapData>(&data_str).unwrap();
                    map_instances.insert(name.to_string(), MapInstance::new(data));
                }
            }
        }
        println!("Done loading maps!");
        Ok(Self {
            map_instances
        })
    }

    pub fn player_join(&mut self, map_name: &str) {
        if let Some(map_instance) = self.map_instances.get_mut(map_name) {
            map_instance.players.push(super::Player::new());
        } else {
            println!("Map not found: {}", map_name);
        }
    }

    pub fn step(&mut self) {
        // TODO: in parallel
        for map_instance in self.map_instances.values_mut() {
            map_instance.step();
        }
    }

    pub async fn next_tick(&self) {
        sleep(Duration::from_millis(10));
        // TODO: some sort of synchronization
    }
}
