/// The Game module handles everything outside of each individual map.
/// This includes authentication, administration, and meta tasks like moving
/// between maps.
///
use std::collections::HashMap;
use std::sync::Arc;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::engine::game_event::GameEvent;
use game_test::map::MapData;
use tokio::sync::RwLock;

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
    // cached for fast access
    pub player_state_map: Arc<RwLock<HashMap<String, PlayerState>>>,
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
                let name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".map", "");
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
            player_state_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_game_event(&self, event: GameEvent) -> anyhow::Result<()> {
        match event {
            GameEvent::PlayerEnterPortal {
                player_id,
                entity_id,
                from_map,
                to_map,
            } => {
                let player_record =
                    PlayerRecord::player_by_id(self.db.clone(), player_id.to_string())
                        .await?
                        .expect(&format!("player does not exist with id {player_id}"));
                if player_record.current_map != from_map {
                    println!("WARNING: deplicate game event, ignoring");
                    return Ok(());
                }
                PlayerRecord::change_map(self.db.clone(), &player_id, &from_map, &to_map).await?;
                let player_id_clone = player_id.clone();
                let from_map_clone = from_map.clone();
                let network_server_clone = self.network_server.clone();
                tokio::spawn(async move {
                    network_server_clone
                        .send_to_player(&player_id_clone, Response::PlayerExitMap(from_map_clone))
                        .await;
                });

                if let Some(map_instance) = self.map_instances.get(&from_map) {
                    map_instance.write().await.remove_player(&player_id).await;
                }
                let mut player_state = self.player_state_map.write().await;
                // remove the player from the old map
                // and add to the new one
                let state = player_state
                    .get_mut(&player_id)
                    .expect("expected player state to exist");
                state.current_map = to_map.to_string();
                // remove the player from the new map if they exist
                if let Some(map_instance) = self.map_instances.get(&state.current_map) {
                    map_instance.write().await.remove_player(&player_id).await;
                }
                // then add them
                if let Some(map_instance) = self.map_instances.get(&to_map) {
                    map_instance.write().await.add_player(state).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn login_player(&self, socket_id: &str, record: &PlayerRecord) -> anyhow::Result<()> {
        let mut player_state_map = self.player_state_map.write().await;

        let player_state = PlayerState::from(record);
        player_state_map.insert(record.id.clone(), player_state.clone());
        let map_instance = self.map_instances.get(&player_state.current_map);
        if map_instance.is_none() {
            println!("Player is on unknown map: {} !", record.current_map);
            // TODO: handle this
            anyhow::bail!("player map instance non-existent")
        }
        let mut map_instance = map_instance.unwrap().write().await;
        map_instance.remove_player(&player_state.id).await;
        map_instance.add_player(&player_state).await?;
        self.network_server
            .register_player(socket_id.to_string(), record.id.clone())
            .await;
        self.network_server
            .send(&socket_id, Response::PlayerLoggedIn(player_state))
            .await?;
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
                    self.network_server
                        .register_player(socket_id.clone(), player.id.clone())
                        .await;
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
            Action::PlayerInput(step_index, entity, input) => {
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                let player_state_map = self.player_state_map.read().await;
                if let Some(player_state) = player_state_map.get(&player_id) {
                    if let Some(map_instance) = self.map_instances.get(&player_state.current_map) {
                        let mut map_instance = map_instance.write().await;
                        map_instance
                            .update_player_input(&player_id, step_index, entity, input)
                            .await?;
                    } else {
                        println!(
                            "ERROR: Player {} is on unknown map: {} !",
                            player_state.username, player_state.current_map
                        );
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}
