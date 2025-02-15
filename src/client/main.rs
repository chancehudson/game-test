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
use login::LoginScreen;
pub use map::Map;
pub use player::Player;
pub use renderable::Renderable;
pub use sprite::AnimatedEntity;
pub use sprite::Sprite;

const SERVER_URL: &'static str = "ws://localhost:1351/socket";

#[macroquad::main("Untitled Game")]
async fn main() -> anyhow::Result<()> {
    AssetBuffer::init().await.unwrap();
    println!("Opening server connection...");
    let mut connection = network::Connection::open(SERVER_URL)?;
    println!("Connected!");
    let mut game = None; // GameState::new().await;
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    let mut login_screen = LoginScreen::new();
    let mut last_action = get_player_action();
    let mut last_server_update = 0.0;
    loop {
        if let Some(msg) = connection.try_receive()? {
            println!("{:?}", msg);
            match msg {
                Response::PlayerLoggedIn(player_id) => {
                    println!("logged in player id {player_id}");
                    game = Some(GameState::new(player_id).await);
                }
                Response::LoginError(err) => {
                    login_screen.error_message = Some(err);
                }
                Response::PlayerState(state) => {
                    if let Some(game) = &mut game {
                        game.player.experience = state.experience;
                        game.active_map = Map::new(&state.current_map).await;
                    }
                }
                Response::PlayerChange(body) => {
                    if let Some(game) = &mut game {
                        if body.id == game.player.id {
                            last_server_update = get_time();
                            game.player.position = body.position;
                            game.player.velocity = body.velocity;
                            game.player.size = body.size;
                        } else {
                            if let Some(player) = game.players.get_mut(&body.id) {
                                player.position = body.position;
                                player.velocity = body.velocity;
                                player.size = body.size;
                                player.action = body.action;
                            } else {
                                let mut player = Player::new(body.id.clone());
                                player.position = body.position;
                                player.velocity = body.velocity;
                                player.size = body.size;
                                player.action = body.action;
                                game.players.insert(body.id.clone(), player);
                            }
                        }
                    }
                }
                Response::ChangeMap(new_map) => {
                    if let Some(game) = &mut game {
                        game.active_map = Map::new(&new_map).await;
                    }
                }
                Response::Log(msg) => {
                    println!("server message: {msg}");
                }
                Response::MapState(entities) => {
                    if let Some(game) = &mut game {
                        game.active_map.entities = entities;
                    }
                }
                Response::MobChange(id, moving_to) => {
                    if let Some(game) = &mut game {
                        for mob in game.active_map.entities.iter_mut() {
                            if mob.id == id {
                                mob.moving_to = moving_to;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if game.is_none() {
            clear_background(RED);
            let (login, create) = login_screen.draw();
            if login {
                connection
                    .send(&Action::LoginPlayer(login_screen.username.clone()))
                    .unwrap_or_else(|e| {
                        login_screen.error_message = Some(e.to_string());
                    });
            } else if create {
                connection
                    .send(&Action::CreatePlayer(login_screen.username.clone()))
                    .unwrap_or_else(|e| {
                        login_screen.error_message = Some(e.to_string());
                    });
            }
            next_frame().await;
            continue;
        }
        let game = game.as_mut().unwrap();
        let new_action = get_player_action();
        if last_action != new_action {
            game.player.action = Some(new_action.clone());
            last_action = new_action.clone();
            connection.send(&Action::SetPlayerAction(new_action))?;
        }
        clear_background(RED);
        // game will handle setting the appropriate camera
        game.render(&mut last_action);

        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text(&format!("fps: {fps}"), 0., 20., 19., BLACK);
        if get_time() - last_server_update > 3.0 {
            draw_text(&format!("server out of sync!"), 0., 40., 19., RED);
        }
        if is_key_pressed(KeyCode::R) {
            AssetBuffer::reload_assets().await?;
        }
        next_frame().await
    }
}

fn get_player_action() -> PlayerAction {
    PlayerAction {
        enter_portal: is_key_down(KeyCode::Up),
        move_left: is_key_down(KeyCode::Left),
        move_right: is_key_down(KeyCode::Right),
        jump: is_key_pressed(KeyCode::Space) && !is_key_down(KeyCode::Down),
        pickup: is_key_pressed(KeyCode::Z),
        downward_jump: is_key_pressed(KeyCode::Space) && is_key_down(KeyCode::Down),
    }
}
