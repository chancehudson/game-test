use game_test::timestamp;
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

const SERVER_URL: &'static str = "ws://127.0.0.1:1351/socket";

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
    let mut last_ping_timestamp = 0.0;
    let mut server_latency = 0.0;
    loop {
        if timestamp() - last_ping_timestamp > 5.0 {
            last_ping_timestamp = timestamp();
            connection.send(&Action::Ping)?;
        }
        if let Some(msg) = connection.try_receive()? {
            println!("{:?}", msg);
            match msg {
                Response::Pong => {
                    server_latency = timestamp() - last_ping_timestamp;
                }
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
                        game.player.username = state.username;
                    }
                }
                Response::PlayerData(state) => {
                    if let Some(game) = &mut game {
                        if let Some(player) = game.players.get_mut(&state.id) {
                            player.username = state.username;
                        } else {
                            let mut player = Player::new(state.id.clone());
                            player.username = state.username;
                            game.players.insert(state.id.clone(), player);
                        }
                    }
                }
                Response::PlayerChange(body) => {
                    if let Some(game) = &mut game {
                        if body.id == game.player.id {
                            last_server_update = get_time();
                            if game.player.position.is_none() {
                                game.player.position = Some(body.position);
                            } else if game.player.position.unwrap() != body.position {
                                let diff = game.player.position.unwrap() - body.position;
                                game.player.position = Some(body.position);
                                game.player.position_err = diff;
                            }
                            game.player.velocity = body.velocity;
                            game.player.size = body.size;
                        } else {
                            if let Some(player) = game.players.get_mut(&body.id) {
                                player.position = Some(body.position);
                                player.velocity = body.velocity;
                                player.size = body.size;
                                player.action = body.action;
                            } else {
                                let mut player = Player::new(body.id.clone());
                                player.position = Some(body.position);
                                player.velocity = body.velocity;
                                player.size = body.size;
                                player.action = body.action;
                                game.players.insert(body.id.clone(), player);
                            }
                        }
                    }
                }
                Response::PlayerRemoved(player_id) => {
                    if let Some(game) = &mut game {
                        game.players.remove(&player_id);
                    }
                }
                Response::ChangeMap(new_map) => {
                    if let Some(game) = &mut game {
                        game.active_map = Map::new(&new_map).await;
                        game.players.clear();
                        game.actors.clear();
                        game.player.position = None;
                        game.player.position_err = Vec2::ZERO;
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
        draw_text(
            &format!("server latency: {} ms", server_latency * 1000.0),
            0.,
            100.,
            15.,
            RED,
        );
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
