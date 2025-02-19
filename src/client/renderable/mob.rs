use macroquad::prelude::Rect;
use macroquad::prelude::Vec2;

use game_test::mob::MOB_DATA;
use game_test::Actor;
use game_test::MapData;
use game_test::Mob;

use crate::AnimatedEntity;

use super::Renderable;

pub struct MobRenderable {
    pub mob: Mob,
    pub standing_animation: AnimatedEntity,
    pub walking_animation: AnimatedEntity,
    pub flip_x: bool,
}

impl MobRenderable {
    pub fn new(mob: Mob) -> Self {
        let data = MOB_DATA.get(&mob.mob_type).unwrap();
        let walking_animation = AnimatedEntity::new(
            &data.walking.sprite_sheet,
            data.walking.frame_count,
            data.walking.fps,
        );
        let standing_animation = AnimatedEntity::new(
            &data.standing.sprite_sheet,
            data.standing.frame_count,
            data.standing.fps,
        );
        Self {
            mob,
            flip_x: false,
            walking_animation,
            standing_animation,
        }
    }
}

impl Renderable for MobRenderable {
    fn render(&mut self, _step_len: f32) {
        let velocity = self.velocity_mut().clone();
        if velocity.x > 0. {
            self.flip_x = true;
        } else if velocity.x < 0. {
            self.flip_x = false;
        }
        if velocity.x != 0. {
            self.walking_animation.flip_x = self.flip_x;
            self.walking_animation.position = *self.position_mut();
            self.walking_animation.update();
            self.walking_animation.draw();
        } else {
            self.standing_animation.flip_x = self.flip_x;
            self.standing_animation.position = *self.position_mut();
            self.standing_animation.update();
            self.standing_animation.draw();
        }
    }
}

impl Actor for MobRenderable {
    fn rect(&self) -> Rect {
        self.mob.rect()
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        self.mob.position_mut()
    }

    fn velocity_mut(&mut self) -> &mut Vec2 {
        self.mob.velocity_mut()
    }

    fn step_physics(&mut self, step_len: f32, map: &MapData) {
        self.mob.step_physics(step_len, map);
    }
}
