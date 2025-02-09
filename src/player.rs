use macroquad::prelude::*;

use super::Actor;

pub struct Player {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
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

    fn render(&self) {
        draw_circle(self.position.x + self.size.x / 2., self.position.y + self.size.y /2., self.size.x/2., GREEN);
    }
}
