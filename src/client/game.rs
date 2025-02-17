use std::collections::HashMap;

use game_test::action::Action;
use game_test::action::PlayerAction;
use game_test::action::Response;
use game_test::engine::Engine;
use game_test::player::Player;
use macroquad::prelude::*;

use crate::login::LoginScreen;

use super::network::Connection;
use super::Actor;
use super::Item;
use super::Map;
use super::PlayerRenderable;
use super::Renderable;
use super::TickSync;

pub trait GameStateTrait: Actor + Renderable {}
impl GameStateTrait for Item {}
// impl GameStateTrait for PlayerRenderable {}

pub struct GameState {
    pub sync: TickSync,
    pub connection: Connection,
    pub player_id: Option<String>,
    pub login_screen: LoginScreen,
    pub engine: Engine,
    pub active_map: Map,
    pub renderables: HashMap<String, PlayerRenderable>,
}

impl GameState {
    pub async fn new(connection: Connection) -> Self {
        let active_map = Map::new("welcome").await;
        GameState {
            sync: TickSync::new(),
            connection,
            login_screen: LoginScreen::new(),
            player_id: None,
            engine: Engine::new(active_map.data.clone(), 0, 0),
            active_map,
            renderables: HashMap::new(),
        }
    }

    pub fn authenticated(&self) -> bool {
        self.player_id.is_some()
    }

    pub fn player_mut(&mut self) -> Option<&mut Player> {
        if let Some(player_id) = &self.player_id {
            self.engine.players.get_mut(player_id)
        } else {
            None
        }
    }

    pub fn add_player(&mut self, player_id: String) {
        self.engine
            .players
            .insert(player_id.clone(), Player::new(player_id.clone()));
        self.renderables.insert(player_id, PlayerRenderable::new());
    }

    // center on the player, except if we're at the edge of a map
    // then lock the camera viewport edge to the edge of the map
    pub fn render_camera(&self) {
        let player_id = self.player_id.clone().unwrap();
        let player = self.engine.players.get(&player_id);
        if player.is_none() {
            println!("Player is not present in engine");
            return;
        }
        let player = player.unwrap();
        let half_screen = Vec2::new(screen_width() / 2., screen_height() / 2.);
        let camera = Camera2D::from_display_rect(Rect::new(
            (player.position().x - half_screen.x)
                .clamp(0., self.engine.map.size.x - screen_width()),
            (player.position().y + half_screen.y).clamp(0., self.engine.map.size.y + 40.), // 40 is the padding at the bottom
            screen_width(),
            -screen_height(),
        ));
        set_camera(&camera);
    }

    pub fn render(&mut self) {
        if self.authenticated() {
            let player_id = self.player_id.clone().unwrap();
            let player = self.engine.players.get(&player_id);
            if player.is_none() {
                println!("Player is not present in engine");
                return;
            }
            let player = player.unwrap();
            // begin rendering
            self.render_camera();
            self.active_map.render(player.position());
            for player in self.engine.players.values() {
                let renderable = self.renderables.get_mut(&player.id).unwrap();
                renderable.render(player);
            }
        } else {
            let (login, create) = self.login_screen.draw();
            if login {
                self.connection
                    .send(&Action::LoginPlayer(self.login_screen.username.clone()))
                    .unwrap_or_else(|e| {
                        self.login_screen.error_message = Some(e.to_string());
                    });
            } else if create {
                self.connection
                    .send(&Action::CreatePlayer(self.login_screen.username.clone()))
                    .unwrap_or_else(|e| {
                        self.login_screen.error_message = Some(e.to_string());
                    });
            }
        }
    }

    pub async fn update(&mut self) -> anyhow::Result<()> {
        self.sync.step(&mut self.connection);
        if let Some(msg) = self.connection.try_receive()? {
            // println!("{:?}", msg);
            match msg {
                Response::TimeSync(id, tick, diff) => {
                    self.sync.tick_response(id, tick, diff);
                }
                Response::Pong => {
                    println!("pong");
                }
                Response::PlayerLoggedIn(state) => {
                    // println!("logged in player id {player_id}");
                    self.player_id = Some(state.id.clone());
                    self.active_map = Map::new(&state.current_map).await;
                    println!("starting engine at tick {}", self.sync.current_tick());
                    self.engine =
                        Engine::new(self.active_map.data.clone(), 0, self.sync.current_tick());
                    let mut player = Player::new(state.id.clone());
                    player.username = state.username;
                    self.renderables
                        .insert(state.id.clone(), PlayerRenderable::new());
                    self.engine.players.insert(state.id, player);
                }
                Response::LoginError(err) => {
                    self.login_screen.error_message = Some(err);
                }
                // Response::PlayerState(tick, state) => {
                //     if let Some(player) = &mut self.engine.players.get_mut(&state.id) {
                //         // player.experience = state.experience;
                //         player.username = state.username;
                //     }
                // }
                Response::PlayerChange(tick, body) => {
                    if let Some(player) = self.engine.players.get_mut(&body.id) {
                        // println!("{} {}", tick, self.engine.current_tick);
                        player.position = body.position;
                        player.velocity = body.velocity;
                        player.size = body.size;
                    }
                }
                Response::PlayerRemoved(player_id) => {
                    self.engine.players.remove(&player_id);
                }
                // Response::ChangeMap(new_map) => {
                //     self.active_map = Map::new(&new_map).await;
                //     // println!("starting engine at tick {}", self.sync.current_tick());
                //     self.engine =
                //         Engine::new(self.active_map.data.clone(), 0, self.sync.current_tick());
                // }
                Response::Log(msg) => {
                    println!("server message: {msg}");
                }
                Response::MapState(tick, entities, players) => {
                    self.active_map.entities = entities.clone();
                    for id in players.keys() {
                        if !self.renderables.contains_key(id) {
                            self.renderables.insert(id.clone(), PlayerRenderable::new());
                        }
                    }
                    self.engine.players = players;
                    self.engine.mobs = entities;
                }
                Response::MobChange(ts, id, moving_to) => {}
                _ => {}
            }
        }
        Ok(())
    }
}
