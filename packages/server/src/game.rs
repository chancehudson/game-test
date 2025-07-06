/// The Game module handles everything outside of each individual map.
/// This includes authentication, administration, and meta tasks like moving
/// between maps.
///
use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use db::AbilityExpRecord;
use db::DEFAULT_MAP;
use db::PlayerStats;
use tokio::sync::RwLock;

use game_common::STEPS_PER_SECOND;
use game_common::game_event::EngineEvent;
use game_common::game_event::GameEvent;
use game_common::map::MapData;
use game_common::network::Action;
use game_common::network::Response;

use super::MapInstance;
use super::network;

use super::PlayerRecord;

#[derive(Clone, Debug)]
pub struct RemoteEngineEvent {
    pub player_id: String,
    pub engine_id: u128,
    pub event: EngineEvent,
    pub step_index: u64,
}

#[derive(Clone)]
pub struct Game {
    pub db: Arc<redb::Database>,
    pub network_server: Arc<network::Server>,
    // name keyed to instance
    pub map_instances: HashMap<String, Arc<RwLock<MapInstance>>>,
    pub instance_for_player_id:
        Arc<DashMap<String, (flume::Sender<RemoteEngineEvent>, Arc<RwLock<MapInstance>>)>>,
    // game events that bubble up
    game_events: (flume::Sender<GameEvent>, flume::Receiver<GameEvent>),
}

impl Game {
    pub async fn new() -> anyhow::Result<Self> {
        let db = Arc::new(redb::Database::create("./game_data.redb")?);
        // init the database collections
        AbilityExpRecord::init(&db)?;
        PlayerRecord::init(&db)?;

        let network_server = Arc::new(network::Server::new().await?);
        let game_events = flume::unbounded();

        let mut map_instances = HashMap::new();
        let mut engine_id_to_map_name = HashMap::new();
        println!("Loading maps...");
        let maps_dir = std::fs::read_dir("./assets/maps").unwrap();
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
                if let Some(file_name) = entry.file_name().to_str() {
                    // fucking apple
                    if file_name.starts_with("._") {
                        continue;
                    }
                    let data_str = std::fs::read_to_string(path_str).unwrap();
                    let data = json5::from_str::<MapData>(&data_str).unwrap();
                    let map_instance = MapInstance::new(
                        data.clone(),
                        network_server.clone(),
                        db.clone(),
                        game_events.0.clone(),
                    )?;
                    engine_id_to_map_name.insert(map_instance.engine.id, data.name.clone());
                    map_instances.insert(name.to_string(), Arc::new(RwLock::new(map_instance)));
                }
            }
        }
        println!("Done loading maps!");

        Ok(Game {
            db: db.clone(),
            network_server: network_server.clone(),
            map_instances,
            instance_for_player_id: Arc::new(DashMap::new()),
            game_events,
        })
    }

    pub async fn handle_events(&self) -> anyhow::Result<()> {
        for game_event in self.game_events.1.drain() {
            match game_event {
                GameEvent::Message(_, _) => {}
                GameEvent::PlayerHealth(_, _) => {
                    // handled in map_instance
                    unreachable!()
                }
                GameEvent::PlayerAbilityExp(_, _, _) => {
                    // handled in map_instance
                    unreachable!()
                }
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
                            // transaction ???
                            let record =
                                PlayerRecord::change_map(&self.db, &player_id, &from_map, &to_map)?;
                            let stats = PlayerStats::by_id(&self.db, &record.id)?;
                            let socket_id =
                                self.network_server.socket_by_player_id(&player_id).await;
                            if socket_id.is_none() {
                                println!("WARNING: player disconnected during map change");
                                return Ok(());
                            }
                            let socket_id = socket_id.unwrap();

                            // must wait for all
                            from_instance.remove_player(&player_id).await?;
                            self.instance_for_player_id.insert(
                                player_id.clone(),
                                (to_instance.pending_actions.0.clone(), to_instance_ref),
                            );
                            to_instance
                                .add_player(socket_id, &record, &stats, requested_spawn_pos)
                                .await?;
                            // send an update
                            self.network_server
                                .send_to_player(&player_id, Response::PlayerExitMap(from_map))
                                .await;
                            self.network_server
                                .send_to_player(&player_id, Response::PlayerState(record))
                                .await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Associate a socket id with a player id
    /// Key a reference to a map instance to the player id
    /// for all players that are "logged in"
    pub async fn login_player(
        &self,
        socket_id: &str,
        record: &PlayerRecord,
        stats: &PlayerStats,
    ) -> anyhow::Result<()> {
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
            .get(&record.current_map)
            .unwrap_or_else(|| {
                println!("WARNING: player on unknown map");
                self.map_instances.get(DEFAULT_MAP).unwrap()
            });

        let map_instance_ref = map_instance.clone();
        let mut map_instance = map_instance.write().await;
        self.instance_for_player_id.insert(
            record.id.clone(),
            (map_instance.pending_actions.0.clone(), map_instance_ref),
        );

        map_instance.remove_player(&record.id).await?;
        map_instance
            .add_player(socket_id.to_string(), &record, &stats, None)
            .await?;

        self.network_server
            .register_player(socket_id.to_string(), record.id.clone())
            .await;
        self.network_server
            .send(&socket_id, Response::PlayerLoggedIn(record.clone()))
            .await?;
        self.network_server
            .send_to_player(&record.id, Response::PlayerState(record.clone()))
            .await;

        Ok(())
    }

    pub async fn handle_action(&self, socket_id: String, action: Action) -> anyhow::Result<()> {
        match action {
            Action::LogoutPlayer => {}
            Action::Ping => {
                self.network_server.send(&socket_id, Response::Pong).await?;
            }
            Action::CreatePlayer(_name) => {
                panic!("not in use");
            }
            Action::LoginPlayer(name) => {
                let player = if let Some(player) = PlayerRecord::player_by_name(&self.db, &name)? {
                    player
                } else {
                    PlayerRecord::create(&self.db, name)?
                };
                let stats = PlayerStats::by_id(&self.db, &player.id)?;
                if let Err(e) = self.login_player(&socket_id, &player, &stats).await {
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
