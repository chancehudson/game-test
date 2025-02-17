use std::collections::HashMap;

use macroquad::prelude::*;

use game_test::action::Action;
use game_test::action::Response;

use crate::login::LoginScreen;
use crate::network::Connection;

use super::Actor;
use super::Item;
use super::Map;
use super::Player;
use super::Renderable;

pub trait GameStateTrait: Actor + Renderable {}
impl GameStateTrait for Item {}
impl GameStateTrait for Player {}

pub struct GameState {
    pub connection: Connection,
    pub authenticated: bool,
    pub login_screen: Option<LoginScreen>,
    pub player: Option<Player>,
    pub active_map: Option<Map>,
    pub actors: Vec<Box<dyn GameStateTrait>>,
    pub players: HashMap<String, Player>,
    pub last_step: f64,
}

impl GameState {
    pub async fn new(connection: Connection) -> Self {
        GameState {
            connection,
            authenticated: false,
            login_screen: Some(LoginScreen::new()),
            player: None,
            active_map: None,
            actors: vec![],
            players: HashMap::new(),
            last_step: 0.0,
        }
    }

    pub async fn change_map(&mut self, to_map: &str) {
        let map = Map::new(to_map).await;
        self.active_map = Some(map);
        self.players.clear();
        self.actors.clear();
    }

    pub async fn handle_msg(&mut self) -> anyhow::Result<()> {
        // if timestamp() - last_ping_timestamp > 5.0 {
        //     last_ping_timestamp = timestamp();
        //     self.connection.send(&Action::Ping)?;
        // }
        while let Some(msg) = self.connection.try_receive()? {
            // println!("{:?}", msg);
            match msg {
                Response::Pong => {
                    // server_latency = timestamp() - last_ping_timestamp;
                }
                Response::PlayerLoggedIn(state) => {
                    println!("logged in player id {}", state.id);
                    let mut player = Player::new(state.id);
                    player.experience = state.experience;
                    player.username = state.username;
                    self.player = Some(player);
                    self.authenticated = true;
                    self.change_map(&state.current_map).await;
                }
                Response::LoginError(err) => {
                    if let Some(login_screen) = &mut self.login_screen {
                        login_screen.error_message = Some(err);
                    }
                }
                Response::PlayerData(state) => {
                    if let Some(player) = self.players.get_mut(&state.id) {
                        player.username = state.username;
                    } else {
                        let mut player = Player::new(state.id.clone());
                        player.username = state.username;
                        self.players.insert(state.id.clone(), player);
                    }
                }
                Response::PlayerChange(body) => {
                    if let Some(player) = &mut self.player {
                        if body.id == player.id {
                            player.position = body.position;
                            player.velocity = body.velocity;
                            player.size = body.size;
                        } else {
                            if let Some(player) = self.players.get_mut(&body.id) {
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
                                self.players.insert(body.id.clone(), player);
                            }
                        }
                    }
                }
                Response::PlayerRemoved(player_id) => {
                    self.players.remove(&player_id);
                }
                Response::ChangeMap(new_map) => {
                    self.change_map(&new_map).await;
                }
                Response::Log(msg) => {
                    println!("server message: {msg}");
                }
                Response::MapState(entities) => {
                    if let Some(active_map) = &mut self.active_map {
                        active_map.entities = entities;
                    }
                }
                Response::MobChange(id, moving_to) => {
                    if let Some(active_map) = &mut self.active_map {
                        for mob in active_map.entities.iter_mut() {
                            if mob.id == id {
                                mob.moving_to = moving_to;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn render(&mut self) {
        if self.authenticated {
            self.render_camera();
            self.render_game().await;
        } else {
            self.render_login();
        }
    }

    // center on the player, except if we're at the edge of a map
    // then lock the camera viewport edge to the edge of the map
    pub fn render_camera(&mut self) {
        let player = self.player.as_mut().unwrap();
        let active_map = self.active_map.as_mut().unwrap();
        let half_screen = Vec2::new(screen_width() / 2., screen_height() / 2.);
        let camera = Camera2D::from_display_rect(Rect::new(
            (player.position().x - half_screen.x).clamp(0., active_map.size.x - screen_width()),
            (player.position().y + half_screen.y).clamp(0., active_map.size.y + 40.), // 40 is the padding at the bottom
            screen_width(),
            -screen_height(),
        ));
        set_camera(&camera);
    }

    pub async fn render_game(&mut self) {
        let active_map = self.active_map.as_mut().unwrap();
        let player = self.player.as_mut().unwrap();
        let time = get_time();
        let step_len = (time - self.last_step) as f32;
        self.last_step = time;

        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &active_map.data);
        }
        for player in self.players.values_mut() {
            player.step_physics(step_len, &active_map.data);
        }
        player.step_physics(step_len, &active_map.data);

        active_map.step_physics(step_len);
        active_map.render(step_len, player.position());
        for player in self.players.values_mut() {
            player.render(step_len);
        }
        player.render(step_len);
        for actor in &mut self.actors {
            actor.render(step_len);
        }
    }

    pub fn render_login(&mut self) {
        clear_background(RED);
        let login_screen = self.login_screen.as_mut().unwrap();
        let (login, create) = login_screen.draw();
        if login {
            self.connection
                .send(&Action::LoginPlayer(login_screen.username.clone()))
                .unwrap_or_else(|e| {
                    login_screen.error_message = Some(e.to_string());
                });
        } else if create {
            self.connection
                .send(&Action::CreatePlayer(login_screen.username.clone()))
                .unwrap_or_else(|e| {
                    login_screen.error_message = Some(e.to_string());
                });
        }
    }
}
