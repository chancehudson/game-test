use std::collections::HashMap;

use macroquad::prelude::*;

use super::Actor;
use super::Item;
use super::Map;
use super::Player;
use super::Renderable;

pub trait GameStateTrait: Actor + Renderable {}
impl GameStateTrait for Item {}
impl GameStateTrait for Player {}

pub struct GameState {
    pub player: Player,
    pub active_map: Map,
    pub actors: Vec<Box<dyn GameStateTrait>>,
    pub players: HashMap<String, Player>,
    pub last_step: f64,
}

impl GameState {
    pub async fn new(id: String) -> Self {
        let player = Player::new(id);
        let active_map = Map::new("welcome").await;
        GameState {
            player,
            active_map,
            actors: vec![],
            players: HashMap::new(),
            last_step: 0.0,
        }
    }

    // center on the player, except if we're at the edge of a map
    // then lock the camera viewport edge to the edge of the map
    pub fn render_camera(&mut self) {
        let half_screen = Vec2::new(screen_width() / 2., screen_height() / 2.);
        let camera = Camera2D::from_display_rect(Rect::new(
            (self.player.position().x - half_screen.x)
                .clamp(0., self.active_map.size.x - screen_width()),
            (self.player.position().y + half_screen.y).clamp(0., self.active_map.size.y + 40.), // 40 is the padding at the bottom
            screen_width(),
            -screen_height(),
        ));
        set_camera(&camera);
    }

    pub fn render(&mut self) {
        let time = get_time();
        let step_len = (time - self.last_step) as f32;
        self.last_step = time;

        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.active_map.data);
        }
        for player in self.players.values_mut() {
            player.step_physics(step_len, &self.active_map.data);
        }
        self.player.step_physics(step_len, &self.active_map.data);

        // begin rendering
        self.render_camera();
        self.active_map.step_physics(step_len);
        self.active_map.render(step_len, self.player.position());
        self.player.render(step_len);
        for player in self.players.values_mut() {
            player.render(step_len);
        }
        for actor in &mut self.actors {
            actor.render(step_len);
        }
    }
}
