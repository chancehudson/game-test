use std::time::Duration;

use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal;
use tokio::sync::broadcast;
use tokio::task::JoinSet;

use db::PlayerRecord;
use game_common::network::Action;
use game_common::timestamp;

mod game;
mod map_instance;
mod network;

use map_instance::MapInstance;

pub static TICK_RATE_MS: f64 = 50.;
pub static TICK_RATE_S_F32: f32 = (TICK_RATE_MS as f32) / 1000.;
pub static TICK_RATE_S: f64 = TICK_RATE_MS / 1000.;

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
            for (socket_id, action) in game_clone.network_server.pending_actions.1.drain() {
                if let Err(e) = game_clone.handle_action(socket_id, action.clone()).await {
                    println!("failed to handle action: {:?} {:?}", action, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let game_clone = game.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let network_action_queue_len = game_clone.network_server.pending_actions.1.len();
            let connected_count = game_clone.network_server.socket_sender.len();
            if network_action_queue_len > 0 || connected_count > 0 {
                for (name, instance) in game_clone.map_instances.iter() {
                    let entity_count = instance.read().await.engine.entities.len();
                    if entity_count > 50 {
                        println!("{name} has {entity_count} entities present");
                    }
                }

                println!("server action_queue len: {}", network_action_queue_len);
                println!(
                    "server socket_sender len: {}",
                    game_clone.network_server.socket_sender.len()
                );
            }
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
        for map_instance in game.map_instances.values().cloned() {
            join_set.spawn(async move {
                let mut map_instance = map_instance.write().await;
                if let Err(e) = map_instance.tick().await {
                    println!("WARNING: error stepping map_instance!");
                    println!("{}", e);
                }
            });
        }
        join_set.join_all().await;
        if let Err(e) = game.handle_events().await {
            println!("WARNING: error handling game events {:?}", e);
        }
        let tick_time = timestamp() - tick_start;
        if tick_time >= TICK_RATE_S {
            println!(
                "WARNING: server tick took more than TICK_RATE_MS ! target: {} ms, actual: {} ms",
                TICK_RATE_MS.round(),
                (1000.0 * tick_time).round(),
            );
        } else {
            let remaining = TICK_RATE_S - tick_time;
            // println!("wait time: {}", remaining);
            tokio::time::sleep(Duration::from_secs_f64(remaining)).await;
        }
    }

    println!("Goodbye!");
    Ok(())
}
