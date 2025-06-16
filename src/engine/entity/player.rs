use std::mem::discriminant;

use bevy_math::IVec2;
use rand::Rng;

use super::EEntity;
use super::emoji::EmojiEntity;
use super::portal::PortalEntity;
use super::rect::RectEntity;
use crate::actor::move_x;
use crate::actor::move_y;
use crate::actor::on_platform;
use crate::engine::GameEngine;
use crate::engine::STEPS_PER_SECOND_I32;
use crate::engine::entity::EngineEntity;
use crate::engine::entity::SEEntity;
use crate::engine::game_event::ServerEvent;
use crate::entity_struct;

entity_struct!(
    pub struct PlayerEntity {
        pub player_id: String, // the game id, not entity id
        weightless_until: Option<u64>,
        attacking_until: Option<u64>,
        pub facing_left: bool,
        pub showing_emoji_until: Option<u64>,
    }
);

impl PlayerEntity {
    pub fn new_with_ids(id: u128, player_id: String) -> Self {
        PlayerEntity {
            id,
            player_id,
            position: IVec2::new(100, 100),
            size: IVec2::new(52, 52),
            player_creator_id: Some(id),
            ..Default::default()
        }
    }
}

impl SEEntity for PlayerEntity {
    fn step(&self, engine: &mut GameEngine) -> Self {
        let step_index = engine.step_index;
        let mut rng = self.rng(&step_index);
        let mut next_self = self.clone();
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let mut velocity = last_velocity.clone();
        let can_jump = on_platform(body, engine);
        let (input_step_index, input) = engine.latest_input(&self.id);
        if input.admin_enable_debug_markers && input_step_index == step_index {
            engine.enable_debug_markers = !engine.enable_debug_markers;
        }
        if let Some(showing_emoji_until) = self.showing_emoji_until {
            if step_index >= showing_emoji_until {
                next_self.showing_emoji_until = None;
            }
        } else if input.show_emoji {
            let show_until = step_index + 120;
            next_self.showing_emoji_until = Some(show_until);
            let id = rng.random();
            let mut emoji = EmojiEntity::new(id, IVec2::MAX, IVec2::new(80, 80));
            emoji.id = id;
            emoji.player_creator_id = Some(self.id);
            emoji.attached_to = Some((self.id, self.size + IVec2::new(-self.size.x / 2, 5)));
            emoji.disappears_at_step_index = show_until;
            engine.spawn_entity(EngineEntity::Emoji(emoji), None, false);
        }

        if input.move_left {
            velocity.x -= 100;
            next_self.facing_left = true;
        }
        if input.move_right {
            velocity.x += 100;
            next_self.facing_left = false;
        }
        if input.enter_portal {
            // TODO: clean up storage/memory in this if clause
            let map_name = engine.map.name.clone();
            let events_channel = engine.game_events.0.clone();
            for entity in engine.entities_by_type(&discriminant(&EngineEntity::Portal(
                PortalEntity::default(),
            ))) {
                match entity {
                    EngineEntity::Portal(p) => {
                        if p.can_enter(self) {
                            //
                            //
                            //
                            // HERE we want to push the event outward into a channel ???
                            // to be consumed elsewhere
                            //
                            // TODO: debounce
                            // pending_events.push(ServerEvent::PlayerEnterPortal {
                            //     player_id: self.player_id.clone(),
                            //     entity_id: self.id,
                            //     from_map: map_name.clone(),
                            //     to_map: p.to.clone(),
                            // });
                            // engine.game_events.0.send()
                            events_channel
                                .send((
                                    step_index,
                                    ServerEvent::PlayerEnterPortal {
                                        player_id: self.player_id.clone(),
                                        entity_id: self.id,
                                        from_map: map_name.clone(),
                                        to_map: p.to.clone(),
                                    },
                                ))
                                .unwrap();
                            break;
                        }
                    }
                    _ => panic!("unexpected variant"),
                }
            }
        }
        if !input.move_left && !input.move_right {
            // accelerate toward 0.0
            velocity.x =
                last_velocity.x.signum() * (last_velocity.x.abs() - last_velocity.x.abs().min(100));
        }
        if let Some(weightless_until) = self.weightless_until {
            if step_index >= weightless_until {
                next_self.weightless_until = None;
            }
            velocity.y += -20;
        } else {
            velocity.y += -20;
        }
        if let Some(attacking_until) = self.attacking_until {
            if step_index >= attacking_until {
                next_self.attacking_until = None;
            }
        }
        // check if the player is standing on a platform
        if input.jump && can_jump && last_velocity.y == 0 {
            velocity.y = 380;
            next_self.weightless_until = Some(step_index + 4);
        } else if can_jump && last_velocity.y <= 0 {
            velocity.y = 0;
        }
        if input.attack && self.attacking_until.is_none() {
            // 15 is the step length of the attack animation
            next_self.attacking_until = Some(step_index + 10);
            let id = rng.random();
            let move_sign = if self.facing_left { -1 } else { 1 };
            let mut projectile = RectEntity::new(
                id,
                IVec2::new(
                    self.center().x + move_sign * self.size.x / 2,
                    self.center().y,
                ),
                IVec2::new(30, 5),
            );
            projectile.player_creator_id = Some(self.id);
            projectile.velocity.x = 800 * move_sign;
            projectile.disappears_at_step_index = Some(step_index + 30);
            engine.spawn_entity(EngineEntity::Rect(projectile), None, false);
        }

        let lower_speed_limit = IVec2::new(-250, -350);
        let upper_speed_limit = IVec2::new(250, 700);
        velocity = velocity.clamp(lower_speed_limit, upper_speed_limit);
        let x_pos = move_x(self.rect(), velocity.x / STEPS_PER_SECOND_I32, &engine.map);
        let map_size = engine.map.size.clone();
        let y_pos = move_y(
            self.rect(),
            velocity.y / STEPS_PER_SECOND_I32,
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
