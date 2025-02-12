use std::sync::Arc;
use std::time::SystemTime;

use global_state::GlobalState;
use nanoid::nanoid;
use once_cell::sync::Lazy;
use redb::Database;

pub use game_test::action::Action;
pub use game_test::action::PlayerState;
pub use game_test::action::Response;
pub use game_test::Actor;
pub use game_test::MapData;

mod db;
mod global_state;
mod item;
mod map_instance;
mod network;
mod player;

pub use db::PlayerRecord;
pub use db::PLAYER_TABLE;
pub use player::Player;

static START_TIMESTAMP_MS: Lazy<u128> = Lazy::new(|| {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
});

/// TODO: rework this whole thing
pub fn timestamp() -> f32 {
    let now_ms: u128 = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let diff = now_ms - *START_TIMESTAMP_MS;
    // we assume diff is representable in an f64
    // convert to seconds
    (diff as f32) / 1000.0
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // start the websocket server loop in it's own thread
    let server = Arc::new(network::Server::new().await?);
    let server_cloned = server.clone();
    let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        while let Ok((stream, _)) = server_cloned.listener.accept().await {
            let server_cloned = server_cloned.clone();
            let _: tokio::task::JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
                server_cloned.accept_connection(stream).await?;
                Ok(())
            });
        }
        Ok(())
    });
    // spawn our db
    let db = Database::create("./game.redb")?;
    {
        let write = db.begin_write()?;
        write.open_table(PLAYER_TABLE)?;
        write.commit()?;
    }
    let mut state = GlobalState::new(db, &server).await?;
    let mut last_step = timestamp();
    let mut last_broadcast = timestamp();

    loop {
        let time = timestamp();
        let step_len = time - last_step;
        last_step = time;

        let mut write_tx = state.db.begin_write()?;
        while let Some((socket_id, action)) = server.action_queue.write().unwrap().pop_front() {
            handle_action(&mut state, &mut write_tx, socket_id, action).await?;
        }
        state.step(step_len);
        write_tx.commit()?;
        // send positions as needed
        if timestamp() - last_broadcast > 1.0 {
            last_broadcast = timestamp();
            for (id, player) in &state.players {
                let r = Response::PlayerBody(game_test::action::PlayerBody {
                    position: (player.position.x, player.position.y),
                    velocity: (player.velocity.x, player.velocity.y),
                });
                state.send_to_player(&id, r).await?;
            }
        }
        state.next_tick().await;
    }
}

async fn handle_action<'a>(
    state: &mut GlobalState<'a>,
    write_tx: &mut redb::WriteTransaction,
    socket_id: String,
    action: Action,
) -> anyhow::Result<()> {
    match action {
        Action::LoginPlayer(name) => {
            if let Some(player) = PlayerRecord::player_by_name(&mut state.db.begin_read()?, &name)?
            {
                state.bind_socket(&socket_id, &player.id);
                state.player_logged_in(player.clone());
                state
                    .server
                    .send(&socket_id, Response::PlayerLoggedIn(player.id.clone()))
                    .await?;
                state
                    .server
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
                state
                    .server
                    .send(
                        &socket_id,
                        Response::LoginError("username does not exist".to_string()),
                    )
                    .await?;
            }
        }
        Action::CreatePlayer(name) => {
            let player_id = nanoid!();
            let mut player_table = write_tx.open_table(PLAYER_TABLE)?;
            let player = PlayerRecord {
                id: player_id.clone(),
                username: name.clone(),
                current_map: "welcome".to_string(),
                experience: 0,
            };
            state.player_logged_in(player.clone());
            state.bind_socket(&socket_id, &player_id);
            player_table.insert(player_id.clone(), bincode::serialize(&player)?.as_slice())?;
            state
                .server
                .send(&socket_id, Response::PlayerLoggedIn(player_id))
                .await?;
            state
                .server
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
        }
        Action::SetPlayerAction(player_action) => {
            if let Some(player_id) = state.socket_player_map.get(&socket_id) {
                if let Some(existing_action) = state.player_actions.get_mut(player_id) {
                    existing_action.update(player_action);
                } else {
                    state
                        .player_actions
                        .insert(player_id.clone(), player_action);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
