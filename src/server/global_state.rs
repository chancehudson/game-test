use std::collections::HashMap;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

use game_test::action::PlayerAction;
use game_test::MapData;
use redb::Database;

use super::Player;
use super::PlayerRecord;
use super::Response;
use crate::map_instance::MapInstance;
use crate::network;

pub struct GlobalState<'a> {
    pub map_instances: HashMap<String, MapInstance>,
    pub db: Database,
    // socket id keyed to player
    pub socket_player_map: HashMap<String, String>,
    // player id keyed to socket
    pub player_socket_map: HashMap<String, String>,
    pub server: &'a network::Server,
    pub players: HashMap<String, Player>,
    pub player_actions: HashMap<String, PlayerAction>,
}

impl<'a> GlobalState<'a> {
    pub async fn new(db: Database, server: &'a network::Server) -> anyhow::Result<Self> {
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
            player_actions: HashMap::new(),
            server,
            db,
            players: HashMap::new(),
            socket_player_map: HashMap::new(),
            player_socket_map: HashMap::new(),
            map_instances,
        })
    }

    pub fn player_logged_in(&mut self, player_record: PlayerRecord) {
        self.players.insert(
            player_record.id.clone(),
            Player::new(player_record.id.clone()),
        );
        self.map_instances
            .get_mut(&player_record.current_map)
            .unwrap()
            .player_ids
            .push(player_record.id.clone());
    }

    pub async fn send_to_player(&self, user_id: &String, res: Response) -> anyhow::Result<()> {
        if let Some(socket_id) = self.player_socket_map.get(user_id) {
            self.server.send(&socket_id, res).await?;
        }
        Ok(())
    }

    pub fn bind_socket(&mut self, socket_id: &String, user_id: &String) {
        self.socket_player_map
            .insert(socket_id.clone(), user_id.clone());
        self.player_socket_map
            .insert(user_id.clone(), socket_id.clone());
    }

    pub fn step(&mut self, step_len: f32) {
        // TODO: in parallel
        for (key, val) in self.player_actions.iter_mut() {
            if let Some(player) = self.players.get_mut(key) {
                val.step_action(player, step_len);
            }
        }
        for map_instance in self.map_instances.values_mut() {
            map_instance.step(&mut self.players, step_len);
        }
    }

    pub async fn next_tick(&self) {
        sleep(Duration::from_millis(10));
        // TODO: some sort of synchronization
    }
}
