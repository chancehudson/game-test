use std::time::Instant;

use game_test::time_since_instant;
use macroquad::prelude::*;

pub use game_test::action::Action;
pub use game_test::action::PlayerAction;
pub use game_test::action::Response;
use game_test::engine::TICK_LEN;
use game_test::time_since;
use game_test::timestamp;
pub use game_test::Actor;
pub use game_test::MapData;

mod asset_buffer;
mod game;
mod item;
mod login;
mod map;
mod network;
mod player;
mod renderable;
mod sprite;
mod tick_sync;

pub use asset_buffer::AssetBuffer;
pub use game::GameState;
pub use item::Item;
use login::LoginScreen;
pub use map::Map;
pub use player::PlayerRenderable;
pub use renderable::Renderable;
pub use sprite::AnimatedEntity;
pub use sprite::Sprite;
use tick_sync::TickSync;

const SERVER_URL: &'static str = "ws://127.0.0.1:1351/socket";

#[macroquad::main("Untitled Game")]
async fn main() -> anyhow::Result<()> {
    AssetBuffer::init().await.unwrap();
    println!("Opening server connection...");
    let mut connection = network::Connection::open(SERVER_URL)?;
    println!("Connected!");
    let mut game = GameState::new(connection).await;
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    let mut last_action = get_player_action();
    let mut last_tick = timestamp();
    let mut tick_delay = 0.0;
    loop {
        if let Err(e) = game.update().await {
            println!("Error updating game: {:?}", e);
        }
        let mut new_action = get_player_action();
        if last_action != new_action {
            last_action = new_action.clone();
            new_action.tick = game.engine.current_tick;
            if let Some(player) = game.player_mut() {
                player.next_action.update(&new_action);
                println!("player {:?}", player.next_action);
                let a = player.next_action.clone();
                println!("{:?}", a);
                new_action.tick = game.sync.current_tick() + 1;
                game.connection.send(&Action::SetPlayerAction(a))?;
            }
        }
        clear_background(RED);
        // game will handle setting the appropriate camera

        let now = timestamp();
        if now - last_tick >= TICK_LEN {
            tick_delay += (now - last_tick) - TICK_LEN;
            if tick_delay >= TICK_LEN {
                println!("double tick");
                tick_delay -= TICK_LEN;
                // game.engine.tick();
            }
            println!("{} tick len", now - last_tick);
            last_tick = now;
            // game.engine.tick();
        }
        game.render();

        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text(&format!("fps: {fps}"), 0., 20., 19., BLACK);
        // if get_time() - last_server_update > 3.0 {
        //     draw_text(&format!("server out of sync!"), 0., 40., 19., RED);
        // }
        if is_key_pressed(KeyCode::R) {
            AssetBuffer::reload_assets().await?;
        }
        draw_text(
            &format!("server latency: {} ms", game.sync.latency * 1000.0),
            0.,
            100.,
            15.,
            RED,
        );
        next_frame().await;
    }
}

fn get_player_action() -> PlayerAction {
    PlayerAction {
        tick: 0,
        enter_portal: is_key_down(KeyCode::Up),
        move_left: is_key_down(KeyCode::Left),
        move_right: is_key_down(KeyCode::Right),
        jump: is_key_pressed(KeyCode::Space) && !is_key_down(KeyCode::Down),
        pickup: is_key_pressed(KeyCode::Z),
        downward_jump: is_key_pressed(KeyCode::Space) && is_key_down(KeyCode::Down),
    }
}
