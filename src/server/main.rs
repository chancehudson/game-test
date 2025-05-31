use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use game_test::action::Action;
use game_test::action::PlayerState;
use game_test::action::Response;
use game_test::timestamp;
use game_test::Actor;
use game_test::TICK_RATE_MS;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

mod db;
mod item;
mod map_instance;
mod mob;
mod network;
mod player;
mod player_connection;
mod state;

pub use db::PlayerRecord;
use map_instance::MapInstance;
pub use player::Player;
pub use player_connection::PlayerConnection;

pub static DB: Lazy<sled::Db> = Lazy::new(|| sled::open("./game_data").unwrap());

pub static SERVER: OnceLock<Arc<network::Server>> = OnceLock::new();
pub static PLAYER_CONNS: Lazy<RwLock<PlayerConnection>> =
    Lazy::new(|| RwLock::new(PlayerConnection::new()));
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
        let diff = time - last_step;

        // handle inputs from the clients
        while let Some((socket_id, action)) = server.action_queue.write().await.pop_front() {
            if let Err(e) = handle_action(socket_id, action.clone()).await {
                println!("failed to handle action: {:?} {:?}", action, e);
            }
        }

        // TODO: have each map tick independently with slightly different offsets
        if diff * 1000.0 < TICK_RATE_MS {
            continue;
        }
        last_step = time;

        // step the game state
        for map_instance in STATE.map_instances.values() {
            map_instance.write().await.tick().await;
        }

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
        Action::LogoutPlayer => {
            // if the socket is associated we'll deassociate it
            if let Some(player_id) = PLAYER_CONNS.write().await.logout_socket(&socket_id).await {
                STATE.logout_player(&player_id).await;
            }
        }
        Action::LoginPlayer(name) => {
            if let Some(player) = PlayerRecord::player_by_name(&name).await? {
                PLAYER_CONNS
                    .write()
                    .await
                    .register_player(socket_id.clone(), player.id.clone())
                    .await;
                let body = STATE.login_player(player.clone()).await;
                SERVER
                    .get()
                    .unwrap()
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
            let record = PlayerRecord::create(name).await;
            if record.is_err() {
                SERVER
                    .get()
                    .unwrap()
                    .send(
                        &socket_id,
                        Response::LoginError(record.err().unwrap().to_string()),
                    )
                    .await?;
                return Ok(());
            }
            let record = record.unwrap();
            PLAYER_CONNS
                .write()
                .await
                .register_player(socket_id.clone(), record.id.clone())
                .await;
            let body = STATE.login_player(record.clone()).await;
            SERVER
                .get()
                .unwrap()
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
            if let Some(player_id) = PLAYER_CONNS
                .read()
                .await
                .player_by_socket_id(&socket_id)
                .await
            {
                STATE
                    .set_player_action(&player_id, player_action, position, velocity)
                    .await;
            }
        }
    }
    Ok(())
}
