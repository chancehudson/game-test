use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;

use game_test::action::PlayerAction;
use game_test::action::PlayerBody;
use game_test::Actor;
use game_test::MapData;

use super::PlayerRecord;

const MAX_VELOCITY: f32 = 500.0;

pub struct Player {
    pub id: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub action: PlayerAction,
    pub record: PlayerRecord,
}

impl Player {
    pub fn new(record: PlayerRecord) -> Self {
        Self {
            id: record.id.clone(),
            position: Vec2::new(0., 0.),
            velocity: Vec2::new(0., 0.),
            size: Vec2::new(52., 52.),
            action: PlayerAction::default(),
            record,
        }
    }

    pub fn body(&self) -> PlayerBody {
        PlayerBody {
            id: self.id.clone(),
            position: self.position,
            velocity: self.velocity,
            size: self.size,
            action: None,
        }
    }
}

impl Actor for Player {
    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        self.step_physics_default(step_len, map);
        self.velocity = self.velocity.clamp(
            Vec2::new(-MAX_VELOCITY, -MAX_VELOCITY),
            Vec2::new(MAX_VELOCITY, MAX_VELOCITY),
        );
    }
}
