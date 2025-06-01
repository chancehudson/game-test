/// The Game module handles everything outside of each individual map.
/// This includes authentication, administration, and meta tasks like moving
/// between maps.
///
use std::collections::HashMap;
use std::sync::Arc;

use bevy::math::Vec2;
use game_test::action::Action;
use game_test::action::PlayerAction;
use game_test::action::PlayerBody;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::map::MapData;
use tokio::sync::RwLock;

use super::network;
use super::MapInstance;

use super::PlayerRecord;

/// Actions that may be taken on a map that will change game state
pub enum MapGameAction {
    // from_map, to_map
    EnterPortal(String, String),
}

#[derive(Clone)]
pub struct Game {
    pub db: sled::Db,
    pub network_server: Arc<network::Server>,
    // name keyed to instance
    pub map_instances: Arc<HashMap<String, Arc<RwLock<MapInstance>>>>,
    // cached for fast access
    pub player_id_map_name: Arc<RwLock<HashMap<String, String>>>,
}

impl Game {
    pub async fn new() -> anyhow::Result<Self> {
        let network_server = Arc::new(network::Server::new().await?);
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
                    map_instances.insert(
                        name.to_string(),
                        Arc::new(RwLock::new(MapInstance::new(data, network_server.clone()))),
                    );
                }
            }
        }
        println!("Done loading maps!");

        Ok(Game {
            db: sled::open("./game_data")?,
            network_server,
            map_instances: Arc::new(map_instances),
            player_id_map_name: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn player_change_map(&self, player_id: &str, new_map_name: &str) {
        let mut player_id_map_name = self.player_id_map_name.write().await;
        // remove the player from the old map
        if let Some(map_name) = player_id_map_name.get(player_id) {
            if let Some(map_instance) = self.map_instances.get(map_name) {
                map_instance.write().await.remove_player(player_id).await;
            }
        }
        let player_record =
            PlayerRecord::player_by_id(self.db.clone(), player_id.to_string()).await;
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
    ) -> Option<MapGameAction> {
        let player_id_map_name = self.player_id_map_name.read().await;
        let map_name = player_id_map_name.get(player_id);
        if map_name.is_none() {
            println!("player: {}", player_id);
            println!("Player is not on a map!");
            return None;
        }
        let map_name = map_name.unwrap();
        let map_instance = self.map_instances.get(map_name);
        if map_instance.is_none() {
            println!("Player is on unknown map: {map_name} !");
            return None;
        }
        let mut map_instance = map_instance.unwrap().write().await;
        map_instance
            .set_player_action(player_id, action, position, velocity)
            .await
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

    pub async fn handle_action(&self, socket_id: String, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Ping => {
                self.network_server.send(&socket_id, Response::Pong).await?;
            }
            Action::LogoutPlayer => {
                // if the socket is associated we'll deassociate it
                if let Some(player_id) = self.network_server.logout_socket(&socket_id).await {
                    // map instances will auto detect the disconnection on next tick
                    // self.logout_player(&player_id).await;
                }
            }
            Action::LoginPlayer(name) => {
                if let Some(player) = PlayerRecord::player_by_name(self.db.clone(), &name).await? {
                    self.network_server
                        .register_player(socket_id.clone(), player.id.clone())
                        .await;
                    let body = self.login_player(player.clone()).await;
                    self.network_server
                        .send(
                            &socket_id,
                            Response::PlayerLoggedIn(
                                PlayerState {
                                    id: player.id.clone(),
                                    username: player.username.clone(),
                                    current_map: player.current_map,
                                    experience: player.experience,
                                    max_health: player.max_health,
                                    health: player.health,
                                },
                                body,
                            ),
                        )
                        .await?;
                } else {
                    self.network_server
                        .send(
                            &socket_id,
                            Response::LoginError("username does not exist".to_string()),
                        )
                        .await?;
                }
            }
            Action::CreatePlayer(name) => {
                let record = PlayerRecord::create(self.db.clone(), name).await;
                if record.is_err() {
                    self.network_server
                        .send(
                            &socket_id,
                            Response::LoginError(record.err().unwrap().to_string()),
                        )
                        .await?;
                    return Ok(());
                }
                let record = record.unwrap();
                self.network_server
                    .register_player(socket_id.clone(), record.id.clone())
                    .await;
                let body = self.login_player(record.clone()).await;
                self.network_server
                    .send(
                        &socket_id,
                        Response::PlayerLoggedIn(
                            PlayerState {
                                id: record.id.clone(),
                                username: record.username.clone(),
                                current_map: record.current_map,
                                experience: record.experience,
                                max_health: record.max_health,
                                health: record.health,
                            },
                            body,
                        ),
                    )
                    .await
                    .unwrap();
            }
            Action::SetPlayerAction(player_action, position, velocity) => {
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                let game_action = self
                    .set_player_action(&player_id, player_action, position, velocity)
                    .await;
                if game_action.is_none() {
                    return Ok(());
                }
                let game_action = game_action.unwrap();
                match game_action {
                    MapGameAction::EnterPortal(from_map, to_map) => {
                        let game = self.clone();
                        tokio::spawn(async move {
                            if let Err(e) = PlayerRecord::change_map(
                                game.db.clone(),
                                player_id.to_string(),
                                &from_map,
                                &to_map,
                            )
                            .await
                            {
                                println!("Error changing map: {:?}", e);
                            } else {
                                game.player_change_map(&player_id, &to_map).await;
                                game.network_server
                                    .send_to_player(&player_id, Response::ChangeMap(to_map))
                                    .await;
                            }
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
