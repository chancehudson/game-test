use std::mem::discriminant;

use bevy_math::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::entity::EngineEntity;
use crate::engine::game_event::GameEvent;
use crate::engine::portal::PortalEntity;
use crate::engine::GameEngine;
use crate::engine::STEP_LEN_S_F32;

use super::entity::{Entity, EntityInput};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerEntity {
    pub id: u128,
    pub player_id: String, // the game id, not entity id
    pub position: Vec2,
    pub size: Vec2,
    velocity: Vec2,
    weightless_until: Option<u64>,
    attacking_until: Option<u64>,
}

impl PlayerEntity {
    pub fn new(id: u128, player_id: String) -> Self {
        PlayerEntity {
            id,
            player_id,
            position: Vec2::new(100., 100.),
            size: Vec2::new(52., 52.),
            velocity: Vec2::new(0.0, 0.0),
            weightless_until: None,
            attacking_until: None,
        }
    }
}

impl Entity for PlayerEntity {
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
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, &engine.map);
        let input = engine
            .latest_input(&self.id)
            .unwrap_or(EntityInput::default());

        if input.move_left {
            velocity.x -= 100.;
        }
        if input.move_right {
            velocity.x += 100.;
        }
        if input.enter_portal {
            // TODO: clean up storage/memory in this if clause
            let mut pending_events = vec![];
            let map_name = engine.map.name.clone();
            for entity in engine.entities_by_type(&discriminant(&EngineEntity::Portal(
                PortalEntity::default(),
            ))) {
                match entity {
                    EngineEntity::Portal(p) => {
                        if p.can_enter(self) {
                            pending_events.push(GameEvent::PlayerEnterPortal {
                                player_id: self.player_id.clone(),
                                entity_id: self.id,
                                from_map: map_name.clone(),
                                to_map: p.to.clone(),
                            });
                        }
                    }
                    _ => panic!("unexpected variant"),
                }
            }
            for event in pending_events.into_iter() {
                engine.emit_event(event);
            }
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
        if let Some(attacking_until) = self.attacking_until {
            if step_index >= &attacking_until {
                next_self.attacking_until = None;
            }
        }
        // check if the player is standing on a platform
        if input.jump && can_jump && last_velocity.y.round() == 0.0 {
            velocity.y = 350.0;
            next_self.weightless_until = Some(step_index + 3);
        } else if can_jump && last_velocity.y.floor() <= 0.0 {
            velocity.y = 0.;
        }
        if input.attack && self.attacking_until.is_none() {
            // 15 is the step length of the attack animation
            next_self.attacking_until = Some(step_index + 15);
            // look for a mob that we can hit
            for (id, entity) in &engine.entities {
                match entity {
                    EngineEntity::Mob(mob) => {
                        if !mob.rect().inflate(5.0).intersect(self.rect()).is_empty() {
                            //
                        }
                    }
                    _ => {}
                }
            }
        }

        let lower_speed_limit = Vec2::new(-250., -350.);
        let upper_speed_limit = Vec2::new(250., 700.);
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
