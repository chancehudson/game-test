use macroquad::prelude::*;

pub use game_test::action::Action;
pub use game_test::action::PlayerAction;
pub use game_test::action::Response;
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

pub use asset_buffer::AssetBuffer;
pub use game::GameState;
pub use item::Item;
pub use map::Map;
pub use player::Player;
pub use renderable::Renderable;
pub use sprite::AnimatedEntity;
pub use sprite::Sprite;

const SERVER_URL: &'static str = "ws://127.0.0.1:1351/socket";

fn window_conf() -> Conf {
    Conf {
        window_title: "Untitled Game".to_owned(),
        sample_count: 0,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
    AssetBuffer::init().await.unwrap();
    println!("Opening server connection...");
    let connection = network::Connection::open(SERVER_URL)?;
    println!("Connected!");
    let mut game = GameState::new(connection).await;
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    let mut last_action = get_player_action();
    loop {
        if let Err(e) = game.handle_msg().await {
            println!("Error handling message: {:?}", e);
        }
        game.render().await;

        let mut new_action = get_player_action();
        if last_action != new_action {
            if let Some(player) = &mut game.player {
                new_action.position = Some(player.position());
                new_action.velocity = Some(player.velocity);
                player.action = Some(new_action.clone());
                last_action = new_action.clone();
                game.connection.send(&Action::SetPlayerAction(new_action))?;
            }
        }
        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text_ex(
            &format!("fps: {fps}"),
            0.,
            20.,
            TextParams {
                font: AssetBuffer::font("helvetica_light"),
                font_size: 15,
                color: BLACK,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!("latency: {} ms", 0.0 * 1000.0),
            0.,
            40.,
            TextParams {
                font: AssetBuffer::font("helvetica_light"),
                font_size: 15,
                color: BLACK,
                ..Default::default()
            },
        );
        show_mouse(false);
        draw_texture_ex(
            &AssetBuffer::texture("assets/pointer.png"),
            mouse_position().0,
            mouse_position().1,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(30., 30.)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}

fn get_player_action() -> PlayerAction {
    PlayerAction {
        velocity: None,
        position: None,
        attack: is_key_pressed(KeyCode::A),
        enter_portal: is_key_down(KeyCode::Up),
        move_left: is_key_down(KeyCode::Left),
        move_right: is_key_down(KeyCode::Right),
        jump: is_key_pressed(KeyCode::Space) && !is_key_down(KeyCode::Down),
        pickup: is_key_pressed(KeyCode::Z),
        downward_jump: is_key_pressed(KeyCode::Space) && is_key_down(KeyCode::Down),
    }
}
