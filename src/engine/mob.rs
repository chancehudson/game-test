use std::mem::discriminant;

use bevy_math::Vec2;
use rand::Rng;

use crate::actor::can_move_left_right;
use crate::actor::can_move_left_right_without_falling;
use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::entity::EEntity;
use crate::engine::entity::EngineEntity;
use crate::engine::entity::SEEntity;
use crate::engine::GameEngine;
use crate::engine::STEPS_PER_SECOND;
use crate::engine::STEP_LEN_S_F32;
use crate::entity_struct;

use super::entity::EntityInput;

entity_struct!(
    pub struct MobEntity {
        pub mob_type: u64,
        weightless_until: Option<u64>,
        moving_sign: f32,
        moving_until: Option<u64>,
        aggro_to: Option<u128>,
    }
);

impl MobEntity {
    fn prestep(&mut self, engine: &mut GameEngine, step_index: &u64) {
        let mut rng = self.rng(step_index);
        if let Some(aggro_to) = self.aggro_to {
            if let Some(aggro_to_entity) = engine.entities.get(&aggro_to) {
                let mut new_input = EntityInput::default();
                if aggro_to_entity.position().x > self.position.x {
                    new_input.move_right = true;
                } else {
                    new_input.move_left = true;
                }
                if aggro_to_entity.position().y < self.position.y {
                    new_input.jump = true;
                }
                engine.register_input(Some(step_index + 1), self.id, new_input);
            } else {
                // aggro target is no longer on map
                self.aggro_to = None;
            }
        } else if let Some(moving_until) = self.moving_until {
            if step_index >= &moving_until {
                self.moving_until = None;
                self.moving_sign = 0.0;
                engine.register_input(Some(step_index + 1), self.id, EntityInput::default());
            } else {
                let (can_move_left_without_falling, can_move_right_without_falling) =
                    can_move_left_right_without_falling(self.rect(), engine);
                let (can_move_left, can_move_right) = can_move_left_right(self.rect(), engine);
                let (can_move_left, can_move_right) = (
                    can_move_left_without_falling && can_move_left,
                    can_move_right_without_falling && can_move_right,
                );
                if (self.moving_sign == 1.0 && !can_move_right)
                    || (self.moving_sign == -1.0 && !can_move_left)
                {
                    let mut new_input = EntityInput::default();
                    new_input.move_right = self.moving_sign == -1.0 && can_move_right;
                    new_input.move_left = self.moving_sign == 1.0 && can_move_left;
                    engine.register_input(Some(step_index + 1), self.id, new_input);
                    self.moving_sign = self.moving_sign * -1.0;
                }
            }
        } else if rng.random_ratio(1, 300) {
            // start moving every so often
            let sign = if rng.random_bool(0.5) { 1. } else { -1. };
            let move_len_s: u64 = rng.random_range(3..10);
            let move_len_steps = move_len_s * STEPS_PER_SECOND;
            self.moving_until = Some(*step_index + move_len_steps);
            self.moving_sign = sign;
            let mut new_input = EntityInput::default();
            new_input.move_right = self.moving_sign == 1.0;
            new_input.move_left = self.moving_sign == -1.0;
            engine.register_input(Some(step_index + 1), self.id, new_input);
        }
    }
}

impl SEEntity for MobEntity {
    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
        let mut next_self = self.clone();
        next_self.prestep(engine, step_index);
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, engine);
        let (_input_step_index, input) = engine
            .latest_input(&self.id)
            .unwrap_or((*step_index, EntityInput::default()));

        if input.move_left {
            velocity.x -= 100.;
        }
        if input.move_right {
            velocity.x += 100.;
        }
        if !input.move_left && !input.move_right {
            // accelerate toward 0.0
            velocity.x = last_velocity.x.signum()
                * (last_velocity.x.abs() - last_velocity.x.abs().min(100.0));
        }
        if let Some(weightless_until) = self.weightless_until {
            if step_index >= &weightless_until {
                next_self.weightless_until = None;
            }
            velocity.y += -20.0;
        } else {
            velocity.y += -20.0;
        }
        // check if the player is standing on a platform
        if input.jump && can_jump && last_velocity.y.round() == 0.0 {
            velocity.y = 350.0;
            next_self.weightless_until = Some(step_index + 3);
        } else if can_jump && last_velocity.y.floor() < 0.0 {
            velocity.y = 0.;
        }

        let lower_speed_limit = Vec2::new(-150., -350.);
        let upper_speed_limit = Vec2::new(150., 700.);
        velocity = velocity.clamp(lower_speed_limit, upper_speed_limit);
        let x_pos = move_x(self.rect(), velocity.x * STEP_LEN_S_F32, &engine.map);
        let map_size = engine.map.size.clone();
        let y_pos = move_y(
            self.rect(),
            velocity.y * STEP_LEN_S_F32,
            engine
                .grouped_entities()
                .get(&discriminant(&EngineEntity::Platform(Default::default())))
                .map(|v| v.as_slice())
                .unwrap_or_else(|| &[]),
            map_size,
        );
        next_self.position.x = x_pos;
        next_self.position.y = y_pos;
        next_self.velocity = velocity;
        next_self
    }
}
