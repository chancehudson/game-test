use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::OnceLock;
use std::time::Duration;

use nanoid::nanoid;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::timestamp;
use game_test::Actor;
use once_cell::sync::Lazy;

mod db;
mod item;
mod map_instance;
mod network;
mod player;
mod player_connection;
mod state;

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
// pub static PLAYERS: LazyLock<RwLock<HashMap<String, Player>>> =
//     LazyLock::new(|| RwLock::new(HashMap::new()));
pub static ACTIONS_BY_PLAYER_ID: LazyLock<RwLock<HashMap<String, VecDeque<Action>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
pub static STATE: Lazy<state::State> = Lazy::new(|| state::State::new());

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
        for map_instance in STATE.map_instances.values() {
            map_instance.write().await.step(step_len).await;
        }

        DB_HANDLER.write().await.commit().await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub async fn send_to_player_err(player_id: &str, res: Response) -> anyhow::Result<()> {
    if let Some(socket_id) = PLAYER_CONNS
        .read()
        .await
        .socket_by_player_id(player_id)
        .await
    {
        SERVER.get().unwrap().send(&socket_id, res.clone()).await?;
    }
    Ok(())
}

pub async fn send_to_player(player_id: &str, res: Response) {
    if let Err(e) = send_to_player_err(player_id, res.clone()).await {
        println!("Error sending to player {player_id}: {:?}", e);
        println!("message: {:?}", res);
        if e.to_string() == "channel closed" {
            let player_id = player_id.to_string();
            // do this async or we deadlock
            tokio::spawn(async move {
                STATE.logout_player(&player_id).await;
            });
        }
    }
}

async fn handle_action(socket_id: String, action: Action) -> anyhow::Result<()> {
    match action {
        Action::Ping => {
            SERVER
                .get()
                .unwrap()
                .send(&socket_id, Response::Pong)
                .await?;
        }
        Action::LoginPlayer(name) => {
            if let Some(player) =
                PlayerRecord::player_by_name(&mut DB_HANDLER.read().await.db.begin_read()?, &name)?
            {
                STATE.login_player(player.clone()).await;
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
                    STATE.login_player(player.clone()).await;
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
                STATE.set_player_action(&player_id, player_action).await;
            }
        }
    }
    Ok(())
}
