use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::GameEngine;
use crate::engine::STEP_LEN_S_F32;

use super::entity::Entity;
use super::entity::EntityInput;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MobEntity {
    pub id: u128,
    pub position: Vec2,
    pub size: Vec2,
    pub mob_type: u64,
    pub velocity: Vec2,
    weightless_until: Option<u64>,
    moving_to_x: Option<f32>,
    aggro_to: Option<u128>,
}

impl MobEntity {
    fn prestep(&mut self, engine: &mut GameEngine, step_index: &u64) {
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
        } else if let Some(moving_to_x) = self.moving_to_x {
            if (self.velocity.x < 0. && self.position.x <= moving_to_x)
                || (self.velocity.x > 0. && self.position.x >= moving_to_x)
            {
                engine.register_input(Some(step_index + 1), self.id, EntityInput::default());
                self.moving_to_x = None;
            } else {
                let mut new_input = EntityInput::default();
                if moving_to_x > self.position.x {
                    new_input.move_right = true;
                } else {
                    new_input.move_left = true;
                }
                engine.register_input(Some(step_index + 1), self.id, new_input);
            }
        } else {
            // start moving every so often
            if rand::random_ratio(1, 600) {
                let sign = if rand::random_bool(0.5) { 1. } else { -1. };
                self.moving_to_x = Some(self.position.x + (sign * 150.0));
            }
        }
    }
}

impl Entity for MobEntity {
    fn id(&self) -> u128 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
        let mut next_self = self.clone();
        next_self.prestep(engine, step_index);
        let map = &engine.map;
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, map);
        let input = engine
            .latest_input(&self.id)
            .unwrap_or(EntityInput::default());

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
        let x_pos = move_x(self.rect(), velocity.x * STEP_LEN_S_F32, map);
        let y_pos = move_y(self.rect(), velocity.y * STEP_LEN_S_F32, map);
        next_self.position.x = x_pos;
        next_self.position.y = y_pos;
        next_self.velocity = velocity;
        next_self
    }
}
