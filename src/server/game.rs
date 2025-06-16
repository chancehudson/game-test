use std::collections::BTreeMap;
/// The Game module handles everything outside of each individual map.
/// This includes authentication, administration, and meta tasks like moving
/// between maps.
///
use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::RwLock;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::engine::STEPS_PER_SECOND;
use game_test::engine::game_event::EngineEvent;
use game_test::engine::game_event::GameEvent;
use game_test::map::MapData;

use super::MapInstance;
use super::network;

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

#[derive(Clone, Debug)]
pub struct RemoteEngineEvent {
    pub player_id: String,
    pub engine_id: u128,
    pub event: EngineEvent,
    pub step_index: u64,
}

#[derive(Clone)]
pub struct Game {
    pub db: sled::Db,
    pub network_server: Arc<network::Server>,
    // name keyed to instance
    pub map_instances: Arc<HashMap<String, Arc<RwLock<MapInstance>>>>,
    pub instance_for_player_id:
        Arc<DashMap<String, (flume::Sender<RemoteEngineEvent>, Arc<RwLock<MapInstance>>)>>,
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
            instance_for_player_id: Arc::new(DashMap::new()),
        })
    }

    pub async fn handle_game_event(&self, event: GameEvent) -> anyhow::Result<()> {
        match event {
            GameEvent::PlayerEnterPortal {
                player_id,
                entity_id: _,
                from_map,
                to_map,
                requested_spawn_pos,
            } => {
                if from_map == to_map {
                    println!("WARNING: trying to move to same map");
                    return Ok(());
                }

                // this is the slowest, but safest implementation
                // TODO: switch to channels
                if let Some(from_instance) = self.map_instances.get(&from_map) {
                    if let Some(to_instance) = self.map_instances.get(&to_map) {
                        let to_instance_ref = to_instance.clone();
                        // must wait for both
                        let mut from_instance = from_instance.write().await;
                        let mut to_instance = to_instance.write().await;
                        // write change to db
                        let record = PlayerRecord::change_map(
                            self.db.clone(),
                            &player_id,
                            &from_map,
                            &to_map,
                        )
                        .await?;

                        // must wait for all
                        from_instance.remove_player(&player_id).await?;
                        self.instance_for_player_id.insert(
                            player_id.clone(),
                            (to_instance.pending_actions.0.clone(), to_instance_ref),
                        );
                        to_instance
                            .add_player(&PlayerState::from(&record), requested_spawn_pos)
                            .await?;
                        // send an update
                        self.network_server
                            .send_to_player(&player_id, Response::PlayerExitMap(from_map))
                            .await;
                        self.network_server
                            .send_to_player(
                                &player_id,
                                Response::PlayerState(PlayerState::from(&record)),
                            )
                            .await;
                    }
                }
            }
        }
        Ok(())
    }

    /// Associate a socket id with a player id
    /// Key a reference to a map instance to the player id
    /// for all players that are "logged in"
    pub async fn login_player(&self, socket_id: &str, record: &PlayerRecord) -> anyhow::Result<()> {
        let player_state = PlayerState::from(record);
        if let Some(entry) = self.instance_for_player_id.get(&record.id) {
            let (_, current_instance) = entry.value();
            let instance = current_instance.read().await;
            if let Some(player) = instance.player_engines.get(&record.id) {
                if player.last_input_step_index > instance.engine.step_index - 10 * STEPS_PER_SECOND
                {
                    anyhow::bail!("A player with this name is already logged in!");
                }
            }
        }
        let map_instance = self
            .map_instances
            .get(&player_state.current_map)
            .unwrap_or_else(|| {
                println!("WARNING: player on unknown map");
                self.map_instances.get(super::db::DEFAULT_MAP).unwrap()
            });

        let map_instance_ref = map_instance.clone();
        let mut map_instance = map_instance.write().await;
        self.instance_for_player_id.insert(
            player_state.id.clone(),
            (map_instance.pending_actions.0.clone(), map_instance_ref),
        );

        map_instance.remove_player(&player_state.id).await?;
        map_instance.add_player(&player_state, None).await?;

        self.network_server
            .register_player(socket_id.to_string(), record.id.clone())
            .await;
        self.network_server
            .send(&socket_id, Response::PlayerLoggedIn(player_state))
            .await?;
        self.network_server
            .send_to_player(&record.id, Response::PlayerState(PlayerState::from(record)))
            .await;

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
                if let Err(e) = self.login_player(&socket_id, &record).await {
                    self.network_server
                        .send(&socket_id, Response::LoginError(e.to_string()))
                        .await?;
                }
            }
            Action::LoginPlayer(name) => {
                let player = if let Some(player) =
                    PlayerRecord::player_by_name(self.db.clone(), &name).await?
                {
                    player
                } else {
                    PlayerRecord::create(self.db.clone(), name).await?
                };
                if let Err(e) = self.login_player(&socket_id, &player).await {
                    self.network_server
                        .send(&socket_id, Response::LoginError(e.to_string()))
                        .await?;
                }
            }
            Action::RemoteEngineEvent(engine_id, event, step_index) => {
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                if let Some(entry) = self.instance_for_player_id.get(&player_id) {
                    let (event_sender, _map_instance) = entry.value();
                    event_sender.send(RemoteEngineEvent {
                        player_id,
                        engine_id,
                        event,
                        step_index,
                    })?;
                }
            }
            // This is a task that occurs outside of the engine because it may be stalled
            // or otherwise incapable of exchanging EngineEvent structs
            //
            // engines have to stay synchronized to within 60 steps (~1 second) of the server
            //
            // if a client desyncs we re-initialize with a serialized GameEngine, and then
            // exchange EngineEvents to agree on changes to the engine state
            //
            Action::RequestEngineReload(_engine_id, step_index) => {
                // but linus said deep indentation bad
                let player_id = self.network_server.player_by_socket_id(&socket_id).await;
                if player_id.is_none() {
                    println!("No player id for socket {} !", socket_id);
                    return Ok(());
                }
                let player_id = player_id.unwrap();
                if let Some(entry) = self.instance_for_player_id.get(&player_id) {
                    let (_, map_instance) = entry.value();
                    let mut map_instance = map_instance.write().await;
                    map_instance.reload_player_engine(&player_id).await?;
                    println!(
                        "Reload requested from step {step_index}: {:?}",
                        map_instance.engine.entities_by_step.get(&step_index)
                    );
                }
            }
        }
        Ok(())
    }
}
