use macroquad::prelude::*;

pub use game_test::Actor;
pub use game_test::MapData;

mod asset_buffer;
mod game;
mod input_handler;
mod sprite;
mod network;
mod renderable;
mod item;
mod map;
mod player;

pub use renderable::Renderable;
pub use player::Player;
pub use map::Map;
pub use item::Item;
pub use sprite::Sprite;
pub use sprite::AnimatedEntity;
pub use game::GameState;
pub use input_handler::InputHandler;
pub use asset_buffer::AssetBuffer;

const SERVER_URL: &'static str = "ws://localhost:1351/socket";

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    AssetBuffer::init().await.unwrap();
    println!("Opening server connection...");
    let mut connection = network::Connection::open(SERVER_URL)?;
    println!("Connected!");
    let mut game = GameState::new().await;
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    loop {
        if let Some(msg) = connection.try_receive()? {
            println!("Received message: {:?}", msg);
        }
        clear_background(RED);
        // game will handle setting the appropriate camera
        game.render();

        if is_key_pressed(KeyCode::A) {
            println!("Sending message...");
            connection.send("Hello, world!".to_string())?;
        }

        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text(&format!("fps: {fps}"), 0., 20., 19., BLACK);
        if is_key_pressed(KeyCode::R) {
            AssetBuffer::reload_assets().await?;
        }
        next_frame().await
    }
}
