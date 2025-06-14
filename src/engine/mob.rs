use std::mem::discriminant;

use bevy_math::IVec2;
use rand::Rng;

use crate::actor::can_move_left_right;
use crate::actor::can_move_left_right_without_falling;
use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::entity::EEntity;
use crate::engine::entity::EngineEntity;
use crate::engine::entity::SEEntity;
use crate::engine::game_event::GameEvent;
use crate::engine::GameEngine;
use crate::engine::STEPS_PER_SECOND;
use crate::engine::STEPS_PER_SECOND_I32;
use crate::entity_struct;

use super::entity::EntityInput;

entity_struct!(
    pub struct MobEntity {
        pub mob_type: u64,
        weightless_until: Option<u64>,
        moving_sign: i32,
        moving_until: Option<u64>,
        aggro_to: Option<u128>,
    }
);

impl MobEntity {
    fn prestep(&mut self, engine: &mut GameEngine) {
        let step_index = engine.step_index;
        let mut rng = self.rng(&step_index);
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
                engine.register_event(
                    None,
                    GameEvent::Input {
                        id: rng.random(),
                        input: new_input,
                        entity_id: self.id,
                        universal: false,
                    },
                );
            } else {
                // aggro target is no longer on map
                self.aggro_to = None;
            }
        } else if let Some(moving_until) = self.moving_until {
            if step_index >= moving_until {
                self.moving_until = None;
                self.moving_sign = 0;
                engine.register_event(
                    None,
                    GameEvent::Input {
                        id: rng.random(),
                        input: EntityInput::default(),
                        entity_id: self.id,
                        universal: false,
                    },
                );
            } else {
                let (can_move_left_without_falling, can_move_right_without_falling) =
                    can_move_left_right_without_falling(self.rect(), engine);
                let (can_move_left, can_move_right) = can_move_left_right(self.rect(), engine);
                let (can_move_left, can_move_right) = (
                    can_move_left_without_falling && can_move_left,
                    can_move_right_without_falling && can_move_right,
                );
                if (self.moving_sign == 1 && !can_move_right)
                    || (self.moving_sign == -1 && !can_move_left)
                {
                    let mut new_input = EntityInput::default();
                    new_input.move_right = self.moving_sign == -1 && can_move_right;
                    new_input.move_left = self.moving_sign == 1 && can_move_left;
                    engine.register_event(
                        None,
                        GameEvent::Input {
                            id: rng.random(),
                            input: new_input,
                            entity_id: self.id,
                            universal: false,
                        },
                    );
                    self.moving_sign = self.moving_sign * -1;
                } else {
                    // do this so if we move for more than
                    // TRAILING_STATE_COUNT steps we can still replay
                    let (step_index, latest_input) = engine.latest_input(&self.id);
                    if engine.step_index > step_index && engine.step_index - step_index > 60 {
                        engine.register_event(
                            None,
                            GameEvent::Input {
                                id: rng.random(),
                                input: latest_input,
                                entity_id: self.id,
                                universal: false,
                            },
                        );
                    }
                }
            }
        } else if rng.random_ratio(1, 300) {
            // start moving every so often
            let sign = if rng.random_bool(0.5) { 1 } else { -1 };
            let move_len_s: u64 = rng.random_range(3..10);
            let move_len_steps = move_len_s * STEPS_PER_SECOND;
            self.moving_until = Some(step_index + move_len_steps);
            self.moving_sign = sign;
            let mut new_input = EntityInput::default();
            new_input.move_right = self.moving_sign == 1;
            new_input.move_left = self.moving_sign == -1;
            engine.register_event(
                None,
                GameEvent::Input {
                    id: rng.random(),
                    input: new_input,
                    entity_id: self.id,
                    universal: false,
                },
            );
        }
        if engine.step_index % 30 == 0 {
            // println!("step: {} {}", engine.step_index, rng.random::<u64>());
        }
    }
}

impl SEEntity for MobEntity {
    fn step(&self, engine: &mut GameEngine) -> Self {
        let step_index = engine.step_index;
        let mut next_self = self.clone();
        next_self.prestep(engine);
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, engine);
        let (_, input) = engine.latest_input(&self.id);

        if input.move_left {
            velocity.x -= 100;
        }
        if input.move_right {
            velocity.x += 100;
        }
        if !input.move_left && !input.move_right {
            // apply friction of 100 units per step
            velocity.x = if last_velocity.x.abs() <= 100 {
                0
            } else {
                last_velocity.x - last_velocity.x.signum() * 100
            };
        }
        if let Some(weightless_until) = self.weightless_until {
            if step_index >= weightless_until {
                next_self.weightless_until = None;
            }
            velocity.y += -20;
        } else {
            velocity.y += -20;
        }
        // check if the player is standing on a platform
        if input.jump && can_jump && last_velocity.y == 0 {
            velocity.y = 350;
            next_self.weightless_until = Some(step_index + 3);
        } else if can_jump && velocity.y < 0 {
            velocity.y = 0;
        }

        let lower_speed_limit = IVec2::new(-150, -350);
        let upper_speed_limit = IVec2::new(150, 700);
        velocity = velocity.clamp(lower_speed_limit, upper_speed_limit);
        let x_pos = move_x(
            self.rect(),
            last_velocity.x / STEPS_PER_SECOND_I32,
            &engine.map,
        );
        let map_size = engine.map.size.clone();
        let y_pos = move_y(
            self.rect(),
            last_velocity.y / STEPS_PER_SECOND_I32,
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

mod tests {

    use super::*;
    use crate::map::MapData;

    pub fn create_test_engine() -> GameEngine {
        let data_str =
            std::fs::read_to_string("assets/maps/digital_skyscrapers.map.json5").unwrap();
        let data = json5::from_str::<MapData>(&data_str).unwrap();
        GameEngine::new(data)
    }

    #[test]
    fn deterministic() {
        let mut engine = create_test_engine();
        engine.step_to(&100);
        let mut engine2 = engine.engine_at_step(&50).unwrap();
        engine2.step_to(&100);
        for _ in 0..1000 {
            engine.step();
            engine2.step();
            let r1 = engine
                .entities
                .first_key_value()
                .unwrap()
                .1
                .rng(&engine.step_index)
                .random::<u64>();
            let r2 = engine2
                .entities
                .first_key_value()
                .unwrap()
                .1
                .rng(&engine2.step_index)
                .random::<u64>();
            assert_eq!(r1, r2);
        }
        assert_eq!(engine.entities.len(), engine2.entities.len());
        for (id, entity) in engine.entities {
            if let Some(e) = engine2.entities.get(&id) {
                assert_eq!(e, &entity);
            } else {
                panic!("entity not exist");
            }
        }
    }
}
