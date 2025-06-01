use std::time::Duration;

use game_test::action::Action;
use game_test::timestamp;
use game_test::Actor;
use game_test::TICK_RATE_MS;

mod db;
mod game;
mod item;
mod map_instance;
mod mob;
mod network;
mod player;

pub use db::PlayerRecord;
use map_instance::MapInstance;
pub use player::Player;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::broadcast;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // shutdown channel
    let (tx, rx) = broadcast::channel::<()>(1);
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = sigterm.recv() => println!("Received SIGTERM"),
            _ = sigint.recv() => println!("Received SIGINT"),
            _ = tokio::signal::ctrl_c() => println!("Received Ctrl+C"),
        }
        tx.send(()).unwrap();
    });

    let game = game::Game::new().await?;

    // WebSocket core loop
    // start the websocket server loop in it's own thread
    let game_clone = game.clone();
    println!("Starting websocket server");
    tokio::spawn(async move {
        while let Ok((stream, _)) = game_clone.network_server.listener.accept().await {
            let server_cloned = game_clone.network_server.clone();
            tokio::spawn(async move {
                server_cloned.accept_connection(stream).await;
            });
        }
    });

    // handle player events as they are received
    let game_clone = game.clone();
    println!("Listening for websocket actions");
    tokio::spawn(async move {
        loop {
            // handle inputs from the clients
            while let Some((socket_id, action)) = game_clone
                .network_server
                .action_queue
                .write()
                .await
                .pop_front()
            {
                if let Err(e) = game_clone.handle_action(socket_id, action.clone()).await {
                    println!("failed to handle action: {:?} {:?}", action, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });

    // game core loop
    println!(
        "Starting game loop ({} maps loaded)",
        game.map_instances.len()
    );
    loop {
        if !rx.is_empty() {
            println!("Halting game loop");
            break;
        }
        let tick_start = timestamp();
        // step the game state in parallel
        let mut join_set = JoinSet::new();
        for map_instance in game.map_instances.values().cloned().collect::<Vec<_>>() {
            join_set.spawn(async move {
                map_instance.write().await.tick().await;
            });
        }
        // wait for all map ticks to complete
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                eprintln!("Map tick failed: {e}");
            }
        }
        let tick_time = timestamp() - tick_start;
        if tick_time >= TICK_RATE_MS / 1000. {
            println!("WARNING: server tick took more than TICK_RATE_MS !");
        } else {
            let remaining = TICK_RATE_MS / 1000. - tick_time;
            tokio::time::sleep(Duration::from_secs_f32(remaining)).await;
        }
    }

    println!("Goodbye!");
    Ok(())
}
