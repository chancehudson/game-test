/// The Game module handles everything outside of each individual map.
/// This includes authentication, administration, and meta tasks like moving
/// between maps.
///
use std::collections::HashMap;
use std::sync::Arc;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::engine::game_event::ServerEvent;
use game_test::map::MapData;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

use crate::map_instance;

use super::network;
use super::MapInstance;

use super::PlayerRecord;

impl From<&PlayerRecord> for PlayerState {
    fn from(value: &PlayerRecord) -> Self {
        PlayerState {
            id: value.id.clone(),
            username: value.username.clone(),
            current_map: value.current_map.clone(),
            experience: value.experience,
            max_health: value.max_health,
            health: value.health,
        }
    }
}

#[derive(Clone)]
pub struct Game {
    pub db: sled::Db,
    pub network_server: Arc<network::Server>,
    // name keyed to instance
    pub map_instances: Arc<HashMap<String, Arc<RwLock<MapInstance>>>>,
    pub instance_for_player_id: Arc<RwLock<HashMap<String, Arc<RwLock<MapInstance>>>>>,
}

impl Game {
    pub async fn new() -> anyhow::Result<Self> {
        let network_server = Arc::new(network::Server::new().await?);
        let mut map_instances = HashMap::new();
        let mut engine_id_to_map_name = HashMap::new();
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
                let name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".map", "");
                if let Some(_file_name) = entry.file_name().to_str() {
                    let data_str = std::fs::read_to_string(path_str).unwrap();
                    let data = json5::from_str::<MapData>(&data_str).unwrap();
                    let map_instance = MapInstance::new(data.clone(), network_server.clone());
                    engine_id_to_map_name.insert(map_instance.engine.id, data.name.clone());
                    map_instances.insert(name.to_string(), Arc::new(RwLock::new(map_instance)));
                }
            }
        }
        println!("Done loading maps!");

        Ok(Game {
            db: sled::open("./game_data")?,
            network_server,
            map_instances: Arc::new(map_instances),
            instance_for_player_id: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_server_event(&self, event: ServerEvent) -> anyhow::Result<()> {
        match event {
            ServerEvent::PlayerEnterPortal {
                player_id,
                entity_id: _,
                from_map,
                to_map,
            } => {
                PlayerRecord::change_map(self.db.clone(), &player_id, &from_map, &to_map).await?;
                let player_id_clone = player_id.clone();
                let from_map_clone = from_map.clone();
                let network_server_clone = self.network_server.clone();
                tokio::spawn(async move {
                    network_server_clone
                        .send_to_player(&player_id_clone, Response::PlayerExitMap(from_map_clone))
                        .await;
                });
                let record = PlayerRecord::player_by_id(self.db.clone(), player_id.clone())
                    .await?
                    .expect("player record does not exist!");

                // remove the player from the old map if they exist
                if let Some(map_instance) = self.map_instances.get(&from_map) {
                    map_instance.write().await.remove_player(&player_id).await;
                }
                self.send_player_state(&player_id).await?;
                // then add them
                if let Some(map_instance) = self.map_instances.get(&to_map) {
                    self.instance_for_player_id
                        .write()
                        .await
                        .insert(player_id.clone(), map_instance.clone());
                    map_instance
                        .write()
                        .await
                        .add_player(&PlayerState::from(&record))
                        .await?;
                } else {
                    anyhow::bail!("unknown map name {to_map}");
                }
            }
        }
        Ok(())
    }

    pub async fn login_player(&self, socket_id: &str, record: &PlayerRecord) -> anyhow::Result<()> {
        let player_state = PlayerState::from(record);
        let map_instance = self.map_instances.get(&player_state.current_map);
        if map_instance.is_none() {
            println!("Player is on unknown map: {} !", record.current_map);
            // TODO: handle this
            anyhow::bail!("player map instance non-existent")
        }
        let map_instance = map_instance.unwrap();
        self.instance_for_player_id
            .write()
            .await
            .insert(player_state.id.clone(), map_instance.clone());
        let mut map_instance = map_instance.write().await;
        map_instance.remove_player(&player_state.id).await;
        map_instance.add_player(&player_state).await?;
        self.network_server
            .register_player(socket_id.to_string(), record.id.clone())
            .await;
        self.network_server
            .send(&socket_id, Response::PlayerLoggedIn(player_state))
            .await?;
        self.send_player_state(&record.id).await?;
        Ok(())
    }

    pub async fn send_player_state(&self, player_id: &str) -> anyhow::Result<()> {
        if let Some(player) =
            PlayerRecord::player_by_id(self.db.clone(), player_id.to_string()).await?
        {
            self.network_server
                .send_to_player(player_id, Response::PlayerState(PlayerState::from(&player)))
                .await;
        } else {
            println!("WARNING: attempting to send player state to non-existent player");
        }

        Ok(())
    }

    pub async fn handle_action(&self, socket_id: String, action: Action) -> anyhow::Result<()> {
        match action {
            Action::LogoutPlayer => {}
            Action::Ping => {
                self.network_server.send(&socket_id, Response::Pong).await?;
            }
            Action::CreatePlayer(name) => {
                let record = PlayerRecord::create(self.db.clone(), name).await?;
                self.login_player(&socket_id, &record).await?;
            }
            Action::LoginPlayer(name) => {
                if let Some(player) = PlayerRecord::player_by_name(self.db.clone(), &name).await? {
                    self.login_player(&socket_id, &player).await?;
                } else {
                    self.network_server
                        .send(
                            &socket_id,
                            Response::LoginError("username does not exist".to_string()),
                        )
                        .await?;
                }
            }
            Action::EngineEvent(engine_id, event, step_index) => {
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                if let Some(map_instance) = self
                    .instance_for_player_id
                    .read()
                    .await
                    .get(&player_id)
                    .cloned()
                {
                    let mut map_instance = map_instance.write().await;
                    map_instance
                        .integrate_client_event(&player_id, &engine_id, event, step_index)
                        .await?;
                }
            }
            // This is a task that occurs outside of the engine because it may be stalled
            // or otherwise incapable of exchanging GameEvent structs
            //
            // engines have to stay synchronized to within 60 steps (~1 second) of the server
            //
            // if a client desyncs we re-initialize with a serialized GameEngine, and then
            // exchange GameEvents to agree on changes to the engine state
            //
            Action::RequestEngineReload(_engine_id) => {
                // but linus said deep indentation bad
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                if let Some(map_instance) = self
                    .instance_for_player_id
                    .read()
                    .await
                    .get(&player_id)
                    .cloned()
                {
                    let mut map_instance = map_instance.write().await;
                    map_instance.reload_player_engine(&player_id).await?;
                }
            }
        }
        Ok(())
    }
}
