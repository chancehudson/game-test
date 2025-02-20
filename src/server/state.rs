use std::collections::HashMap;

use bevy::math::Vec2;
use game_test::action::PlayerAction;
use game_test::action::PlayerBody;
use game_test::MapData;
use tokio::sync::RwLock;

use super::MapInstance;
use super::PlayerRecord;

pub struct State {
    // name keyed to instance
    pub map_instances: HashMap<String, RwLock<MapInstance>>,
    pub player_id_map_name: RwLock<HashMap<String, String>>,
}

impl State {
    pub fn new() -> Self {
        let mut map_instances = HashMap::new();
        println!("Loading maps...");
        let maps_dir = std::fs::read_dir("assets/maps").unwrap();
        for entry in maps_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if let Some(extension) = path.extension() {
                if extension != "json5" {
                    continue;
                }
                let name = path.file_stem().unwrap().to_str().unwrap();
                if let Some(_file_name) = entry.file_name().to_str() {
                    let data_str = std::fs::read_to_string(path_str).unwrap();
                    let data = json5::from_str::<MapData>(&data_str).unwrap();
                    map_instances.insert(name.to_string(), RwLock::new(MapInstance::new(data)));
                }
            }
        }
        println!("Done loading maps!");
        Self {
            map_instances,
            player_id_map_name: RwLock::new(HashMap::new()),
        }
    }

    pub async fn player_change_map(&self, player_id: &str, new_map_name: &str) {
        let mut player_id_map_name = self.player_id_map_name.write().await;
        // remove the player from the old map
        if let Some(map_name) = player_id_map_name.get(player_id) {
            if let Some(map_instance) = self.map_instances.get(map_name) {
                map_instance.write().await.remove_player(player_id).await;
            }
        }
        let player_record = PlayerRecord::player_by_id(player_id.to_string()).await;
        if player_record.is_err() {
            println!("Error loading player record: {:?}", player_record.err());
            return;
        }
        let player_record = player_record.unwrap();
        if player_record.is_none() {
            println!("Player record not found: {}", player_id);
            return;
        }
        let player_record = player_record.unwrap();
        player_id_map_name.insert(player_id.to_string(), player_record.current_map.clone());
        if let Some(map_instance) = self.map_instances.get(new_map_name) {
            map_instance.write().await.add_player(player_record).await;
        }
    }

    pub async fn set_player_action(
        &self,
        player_id: &str,
        action: PlayerAction,
        position: Vec2,
        velocity: Vec2,
    ) {
        let player_id_map_name = self.player_id_map_name.read().await;
        let map_name = player_id_map_name.get(player_id);
        if map_name.is_none() {
            println!("player: {}", player_id);
            println!("Player is not on a map!");
            return;
        }
        let map_name = map_name.unwrap();
        let map_instance = self.map_instances.get(map_name);
        if map_instance.is_none() {
            println!("Player is on unknown map: {map_name} !");
            return;
        }
        let mut map_instance = map_instance.unwrap().write().await;
        map_instance
            .set_player_action(player_id, action, position, velocity)
            .await;
    }

    pub async fn logout_player(&self, player_id: &str) {
        let map_name = self.player_id_map_name.write().await.remove(player_id);
        if map_name.is_none() {
            return;
        }
        let map_name = map_name.unwrap();
        let map_instance = self.map_instances.get(&map_name);
        if map_instance.is_none() {
            println!("Player is on unknown map: {map_name} !");
            return;
        }
        let mut map_instance = map_instance.unwrap().write().await;
        map_instance.remove_player(&player_id).await;
    }

    pub async fn login_player(&self, record: PlayerRecord) -> PlayerBody {
        self.player_id_map_name
            .write()
            .await
            .insert(record.id.clone(), record.current_map.clone());
        let map_instance = self.map_instances.get(&record.current_map);
        if map_instance.is_none() {
            println!("Player is on unknown map: {} !", record.current_map);
            // TODO: handle this
            panic!("player map instance non-existent");
        }
        let mut map_instance = map_instance.unwrap().write().await;
        map_instance.add_player(record).await
    }
}
