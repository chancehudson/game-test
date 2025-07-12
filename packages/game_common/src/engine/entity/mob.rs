use bevy_math::IVec2;
use rand::Rng;

use db::Ability;
use db::PlayerStats;

use crate::GameEngine;
use crate::STEPS_PER_SECOND;
use crate::STEPS_PER_SECOND_I32;
use crate::actor::can_move_left_right;
use crate::actor::can_move_left_right_without_falling;
use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::damage_calc::compute_damage;
use crate::data::map::DropTableData;
use crate::entity::EEntity;
use crate::entity::EngineEntity;
use crate::entity::SEEntity;
use crate::entity::item::ItemEntity;
use crate::entity::mob_damage::MobDamageEntity;
use crate::entity::platform::PlatformEntity;
use crate::entity::player::PlayerEntity;
use crate::entity_struct;
use crate::game_event::EngineEvent;
use crate::game_event::GameEvent;
use crate::system::input::InputSystem;

use super::EntityInput;

const KNOCKBACK_STEPS: u64 = 20;

entity_struct!(
    pub struct MobEntity {
        pub mob_type: u64,
        pub drop_table: Vec<DropTableData>,
        weightless_until: Option<u64>,
        pub moving_sign: i32,
        moving_until: Option<u64>,
        // entity it, last hit step index
        pub aggro_to: Option<(u128, u64)>,
        pub received_damage_this_step: Vec<u64>,
        pub receiving_damage_until: Option<u64>,
        // direction, until
        pub knockback_until: Option<(i32, u64)>,
        pub current_health: u64,
        pub is_dead: bool,
        input_system: InputSystem,
    }
);

impl MobEntity {
    // handle movement calculations
    fn prestep(&mut self, engine: &mut GameEngine) {
        let step_index = engine.step_index;
        let mut rng = self.rng(&step_index);
        if let Some((aggro_to, _last_hit_step)) = self.aggro_to {
            if let Some(aggro_to_entity) = engine.entities.get(&aggro_to) {
                let mut new_input = EntityInput::default();
                if aggro_to_entity.position().x > self.position.x {
                    new_input.move_right = true;
                    self.moving_sign = 1;
                } else {
                    new_input.move_left = true;
                    self.moving_sign = -1;
                }
                if aggro_to_entity.position().y > self.position.y && rng.random_bool(0.01) {
                    new_input.jump = true;
                }
                engine.register_event(
                    None,
                    EngineEvent::Input {
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
                    EngineEvent::Input {
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
                        EngineEvent::Input {
                            input: new_input,
                            entity_id: self.id,
                            universal: false,
                        },
                    );
                    self.moving_sign = self.moving_sign * -1;
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
                EngineEvent::Input {
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
        let mut next_self = self.clone();
        // render a single frame with is_dead=true to trigger frontend animations
        if self.is_dead {
            next_self.received_damage_this_step = vec![];
            engine.register_event(
                None,
                EngineEvent::RemoveEntity {
                    entity_id: self.id,
                    universal: false,
                },
            );
            return next_self;
        }
        next_self.received_damage_this_step = vec![];
        next_self.prestep(engine);
        let mut rng = self.rng(&engine.step_index);
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, engine);
        next_self.input_system.step(self, engine);
        let (_, input) = &next_self.input_system.latest_input;

        let step_index = engine.step_index;
        let game_events_sender = engine.game_events.0.clone();

        // look for damage the mob is receiving
        for entity in engine
            .entities_by_type::<MobDamageEntity>()
            .cloned()
            .collect::<Vec<_>>()
        {
            if entity.contacted_mob_id.is_none() {
                continue;
            }
            let mob_id = entity.contacted_mob_id.unwrap();
            if mob_id != self.id {
                continue;
            }
            if let Some((aggro_to, _)) = next_self.aggro_to {
                if aggro_to != entity.player_creator_id().unwrap() {
                    // don't allow multiple players to attack the same mob at the same time
                    continue;
                }
            }
            if entity.player_creator_id().is_none() {
                println!("WARNING: mob damage entity has not player creator!");
                continue;
            }
            let player_entity_id = entity.player_creator_id.unwrap();
            if let Some(player_entity) =
                engine.entity_by_id::<PlayerEntity>(&player_entity_id, None)
            {
                next_self.aggro_to = Some((entity.player_creator_id().unwrap(), step_index));
                // receiving damage
                let knockback_dir = if entity.center().x > self.center().x {
                    -1
                } else {
                    1
                };
                next_self.knockback_until = Some((knockback_dir, step_index + KNOCKBACK_STEPS));
                next_self.weightless_until = Some(step_index + (KNOCKBACK_STEPS / 2));
                let damage_amount = compute_damage(
                    &Ability::Strength,
                    &player_entity.stats,
                    &PlayerStats::default(),
                    &mut rng,
                );
                next_self.received_damage_this_step.push(damage_amount);
                if damage_amount > 0 {
                    game_events_sender
                        .send(GameEvent::PlayerAbilityExp(
                            entity.player_creator_id().unwrap(),
                            entity.ability.clone(),
                            damage_amount,
                        ))
                        .unwrap();
                }
                if next_self.current_health <= damage_amount {
                    next_self.is_dead = true;
                    let mut x_offset = 0i32;
                    for drop in self
                        .drop_table
                        .iter()
                        .filter_map(|drop_data| drop_data.drop(&mut rng))
                        .collect::<Vec<_>>()
                    {
                        // drop an item
                        let id = engine.generate_id();
                        engine.spawn_entity(
                            EngineEntity::Item(ItemEntity::new_item(
                                id,
                                self.center() + IVec2::new(x_offset, 0),
                                drop.0, // item type
                                drop.1, // amount
                                player_entity_id,
                                engine.step_index,
                            )),
                            None,
                            false,
                        );
                        x_offset += 10;
                    }
                    break;
                } else {
                    next_self.current_health -= damage_amount;
                }
            }
        }

        if let Some((_, last_damage_step)) = next_self.aggro_to {
            if last_damage_step < engine.step_index && engine.step_index - last_damage_step >= 600 {
                // de-aggro
                next_self.aggro_to = None;
            }
        }

        if let Some((direction, until)) = self.knockback_until {
            if engine.step_index >= until {
                next_self.knockback_until = None;
            } else {
                // linear decay
                velocity.x += direction * ((until - engine.step_index) as i32) * 100;
            }
        } else {
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
        let x_pos = move_x(self.rect(), last_velocity.x / STEPS_PER_SECOND_I32, &engine);
        let map_size = engine.size.clone();
        let platforms = engine.entities_by_type::<PlatformEntity>();
        let y_pos = move_y(
            self.rect(),
            last_velocity.y / STEPS_PER_SECOND_I32,
            &platforms.collect::<Vec<_>>(),
            map_size,
        );
        next_self.position.x = x_pos;
        next_self.position.y = y_pos;
        next_self.velocity = velocity;
        next_self
    }
}
