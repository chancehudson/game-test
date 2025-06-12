use std::collections::HashMap;
use std::mem::discriminant;

use bevy_math::Vec2;
use rand::Rng;

use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::entity::EngineEntity;
use crate::engine::entity::SEEntity;
use crate::engine::game_event::GameEvent;
use crate::engine::portal::PortalEntity;
use crate::engine::rect::RectEntity;
use crate::engine::GameEngine;
use crate::engine::STEP_LEN_S_F32;
use crate::entity_struct;

use super::entity::{EEntity, EntityInput};

entity_struct!(
    pub struct PlayerEntity {
        pub player_id: String, // the game id, not entity id
        weightless_until: Option<u64>,
        attacking_until: Option<u64>,
        pub created_ids: HashMap<u128, u64>,
        pub facing_left: bool,
        pub is_active: bool,
    }
);

impl PlayerEntity {
    pub fn new_with_ids(id: u128, player_id: String) -> Self {
        PlayerEntity {
            id,
            player_id,
            position: Vec2::new(100., 100.),
            size: Vec2::new(52., 52.),
            ..Default::default()
        }
    }
}

impl SEEntity for PlayerEntity {
    fn step(&self, engine: &mut GameEngine, step_index: &u64) -> Self {
        #[cfg(feature = "server")]
        let mut rng = self.rng(step_index);
        #[cfg(not(feature = "server"))]
        let mut rng = self.rng_client(step_index);
        let mut next_self = self.clone();
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, engine);
        let (input_step_index, input) = engine
            .latest_input(&self.id)
            .unwrap_or((*step_index, EntityInput::default()));
        if input.admin_enable_debug_markers && &input_step_index == step_index {
            engine.enable_debug_markers = !engine.enable_debug_markers;
        }

        if input.move_left {
            velocity.x -= 100.;
            next_self.facing_left = true;
        }
        if input.move_right {
            velocity.x += 100.;
            next_self.facing_left = false;
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
                                step_index: *step_index,
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
            next_self.attacking_until = Some(step_index + 10);
            let id = rng.random();
            next_self.created_ids.insert(id, step_index + 1);
            let move_sign = if self.facing_left { -1.0 } else { 1.0 };
            let mut projectile = RectEntity::new(
                id,
                Vec2::new(
                    self.center().x + move_sign * self.size.x / 2.0,
                    self.center().y,
                ),
                Vec2::new(30., 5.),
            );
            if self.is_active {
                projectile.pure = true;
            }
            projectile.velocity.x = 800. * move_sign;
            projectile.disappears_at_step_index = Some(step_index + 30);
            engine.spawn_entity(EngineEntity::Rect(projectile), None, false);
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
