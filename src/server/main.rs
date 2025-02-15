use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::OnceLock;

use macroquad::prelude::Vec2;
use nanoid::nanoid;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::timestamp;
use game_test::Actor;
use game_test::MapData;

mod db;
mod item;
mod map_instance;
mod network;
mod player;
mod player_connection;

use db::DBHandler;
pub use db::PlayerRecord;
use db::WriteRequest;
pub use db::PLAYER_TABLE;
use map_instance::MapInstance;
pub use player::Player;
pub use player_connection::PlayerConnection;
use tokio::sync::RwLock;

pub static SERVER: OnceLock<Arc<network::Server>> = OnceLock::new();
pub static DB_HANDLER: LazyLock<RwLock<DBHandler>> =
    LazyLock::new(|| RwLock::new(DBHandler::new("./game.redb").unwrap()));
pub static PLAYER_CONNS: LazyLock<RwLock<PlayerConnection>> =
    LazyLock::new(|| RwLock::new(PlayerConnection::new()));
pub static PLAYERS: LazyLock<RwLock<HashMap<String, Player>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
pub static MAP_INSTANCES: LazyLock<RwLock<HashMap<String, MapInstance>>> = LazyLock::new(|| {
    let mut map_instances = HashMap::new();
    println!("Loading maps...");
    let maps_dir = std::fs::read_dir("maps").unwrap();
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
                map_instances.insert(name.to_string(), MapInstance::new(data));
            }
        }
    }
    println!("Done loading maps!");
    RwLock::new(map_instances)
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //#########################
    // Websocket core loop
    // start the websocket server loop in it's own thread
    let server = Arc::new(network::Server::new().await?);
    SERVER
        .set(server.clone())
        .map_err(|_| anyhow::anyhow!("Server was already initialized"))?;
    let server_cloned = server.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = server_cloned.listener.accept().await {
            let server_cloned = server_cloned.clone();
            tokio::spawn(async move {
                server_cloned.accept_connection(stream).await;
            });
        }
    });

    //#########################
    // Game core loop
    let mut last_step = timestamp();
    let mut last_broadcast = timestamp();
    loop {
        let time = timestamp();
        let step_len = time - last_step;
        last_step = time;

        // handle inputs from the clients
        while let Some((socket_id, action)) = server.action_queue.write().await.pop_front() {
            if let Err(e) = handle_action(socket_id, action.clone()).await {
                println!("failed to handle action: {:?} {:?}", action, e);
            }
        }

        // step the game state
        step(step_len).await?;

        DB_HANDLER.write().await.commit().await?;

        // send position updates as needed
        // TODO: in background thread
        if timestamp() - last_broadcast > 1.0 {
            println!(
                "action queue len: {}",
                server.action_queue.read().await.len()
            );
            println!(
                "socket sender count: {}",
                server.socket_sender.read().await.keys().len()
            );

            last_broadcast = timestamp();
            for (id, player) in PLAYERS.read().await.iter() {
                send_to_player(&id, Response::PlayerChange(player.body())).await;
                {
                    if let Some(map) = MAP_INSTANCES.read().await.get(&player.record.current_map) {
                        let r = Response::MapState(map.mobs.clone());
                        send_to_player(&id, r).await;
                    }
                }
            }
        }
    }
}

/// TODO: calculate correct position on map
async fn login_player(record: PlayerRecord) {
    PLAYERS
        .write()
        .await
        .insert(record.id.clone(), Player::new(record));
}

async fn logout_player(player_id: &str) {
    PLAYERS.write().await.remove(player_id);
}

pub async fn send_to_player(player_id: &str, res: Response) {
    if let Some(socket_id) = PLAYER_CONNS
        .read()
        .await
        .socket_by_player_id(player_id)
        .await
    {
        if let Err(e) = SERVER.get().unwrap().send(&socket_id, res.clone()).await {
            println!("Error sending to player {player_id}: {:?}", e);
            println!("message: {:?}", res);
            if e.to_string() == "channel closed" {
                let player_id = player_id.to_string();
                // do this async or we deadlock
                tokio::spawn(async move {
                    logout_player(&player_id).await;
                });
            }
        }
    }
}

pub async fn step(step_len: f32) -> anyhow::Result<()> {
    // TODO: in parallel
    for (_, player) in PLAYERS.write().await.iter_mut() {
        let new_action = player.action.clone().step_action(player, step_len);
        player.action = new_action;
        if player.action.enter_portal {
            player.action.enter_portal = false;
            // determine if the player is overlapping a portal
            let map_instances = MAP_INSTANCES.read().await;
            let map = map_instances.get(&player.record.current_map);
            if map.is_none() {
                println!("Player {} is on unknown map!", player.record.username);
                continue;
            }
            let map = map.unwrap();
            for portal in &map.map.portals {
                if portal
                    .rect()
                    .contains(player.position + Vec2::new(15., 15.))
                {
                    player.record.current_map = portal.to.clone();
                    // user is moving
                    let player_id = player.id.clone();
                    let to_map = portal.to.clone();
                    DB_HANDLER.write().await.write(WriteRequest {
                        table: "players".to_string(),
                        key: player.id.clone(),
                        value: bincode::serialize(&player.record)?,
                        callback: Some(Box::pin(async move {
                            send_to_player(&player_id, Response::ChangeMap(to_map)).await;
                        })),
                    });
                    break;
                }
            }
        }
    }
    for player in PLAYERS.write().await.values_mut() {
        let map_instances = MAP_INSTANCES.read().await;
        let map = map_instances.get(&player.record.current_map);
        if map.is_none() {
            println!("Player {} is on unknown map!", player.record.username);
            continue;
        }
        let map = map.unwrap();
        player.step_physics(step_len, &map.map);
    }
    for map_instance in MAP_INSTANCES.write().await.values_mut() {
        let mut players = PLAYERS.write().await;
        map_instance.step(&mut *players, step_len);
    }
    Ok(())
}

async fn handle_action(socket_id: String, action: Action) -> anyhow::Result<()> {
    match action {
        Action::LoginPlayer(name) => {
            if let Some(player) =
                PlayerRecord::player_by_name(&mut DB_HANDLER.read().await.db.begin_read()?, &name)?
            {
                login_player(player.clone()).await;
                PLAYER_CONNS
                    .write()
                    .await
                    .register_player(socket_id.clone(), player.id.clone())
                    .await;
                SERVER
                    .get()
                    .unwrap()
                    .send(&socket_id, Response::PlayerLoggedIn(player.id.clone()))
                    .await?;
                SERVER
                    .get()
                    .unwrap()
                    .send(
                        &socket_id,
                        Response::PlayerState(PlayerState {
                            id: player.id.clone(),
                            username: player.username.clone(),
                            current_map: player.current_map,
                            experience: player.experience,
                        }),
                    )
                    .await?;
            } else {
                SERVER
                    .get()
                    .unwrap()
                    .send(
                        &socket_id,
                        Response::LoginError("username does not exist".to_string()),
                    )
                    .await?;
            }
        }
        Action::CreatePlayer(name) => {
            let player_id = nanoid!();
            let player = PlayerRecord {
                id: player_id.clone(),
                username: name.clone(),
                current_map: "welcome".to_string(),
                experience: 0,
            };
            PLAYER_CONNS
                .write()
                .await
                .register_player(socket_id.clone(), player_id.clone())
                .await;
            DB_HANDLER.write().await.write(WriteRequest {
                table: "players".to_string(),
                key: player_id.clone(),
                value: bincode::serialize(&player)?,
                callback: Some(Box::pin(async move {
                    login_player(player.clone()).await;
                    SERVER
                        .get()
                        .unwrap()
                        .send(&socket_id, Response::PlayerLoggedIn(player_id))
                        .await
                        .unwrap();
                    SERVER
                        .get()
                        .unwrap()
                        .send(
                            &socket_id,
                            Response::PlayerState(PlayerState {
                                id: player.id.clone(),
                                username: player.username.clone(),
                                current_map: player.current_map,
                                experience: player.experience,
                            }),
                        )
                        .await
                        .unwrap();
                })),
            });
        }
        Action::SetPlayerAction(player_action) => {
            if let Some(player_id) = PLAYER_CONNS
                .read()
                .await
                .player_by_socket_id(&socket_id)
                .await
            {
                if let Some(player) = PLAYERS.write().await.get_mut(&player_id) {
                    // if the player has begun moving or stopped moving broadcast
                    // to the rest of the map
                    if player.action.move_left != player_action.move_left
                        || player.action.move_right != player_action.move_right
                    {
                        let active_map = player.record.current_map.clone();
                        let player_id = player.id.clone();
                        let mut body = player.body();
                        body.action = Some(player_action.clone());
                        tokio::spawn(async move {
                            for (id, player_int) in PLAYERS.read().await.iter() {
                                if player_int.record.current_map == active_map && id != &player_id {
                                    send_to_player(
                                        &player_int.id,
                                        Response::PlayerChange(body.clone()),
                                    )
                                    .await;
                                }
                            }
                        });
                    }
                    player.action.update(player_action);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
