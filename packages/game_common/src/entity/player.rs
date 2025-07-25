use bevy_math::IVec2;
use rand::Rng;

use db::Ability;
use db::PlayerRecord;
use db::PlayerStats;

use crate::prelude::*;
use crate::system::input::InputSystem;

const DAMAGE_IFRAME_STEPS: u64 = 120;
const KNOCKBACK_STEPS: u64 = 10;

entity_struct!(
    pub struct PlayerEntity {
        pub player_id: String, // the game id, not entity id
        pub record: PlayerRecord,
        weightless_until: Option<u64>,
        attacking_until: Option<u64>,
        pub facing_left: bool,
        pub showing_emoji_until: Option<u64>,
        pub stats: PlayerStats,
        pub received_damage_this_step: (bool, u64),
        pub receiving_damage_until: Option<u64>,
        // direction, until
        pub knockback_until: Option<(i32, u64)>,
        pub input_system: InputSystem,
    }
);

impl PlayerEntity {
    pub fn new_with_ids(id: u128, record: PlayerRecord, stats: PlayerStats) -> Self {
        PlayerEntity {
            id,
            player_id: record.id.clone(),
            position: IVec2::new(100, 100),
            size: IVec2::new(52, 52),
            player_creator_id: Some(id),
            stats,
            record,
            ..Default::default()
        }
    }

    pub fn is_dead(&self) -> bool {
        self.record.current_health == 0
    }
}

impl SEEntity for PlayerEntity {
    fn step<T: GameEngine>(&self, engine: &T) -> Self {
        let step_index = engine.step_index();
        let mut rng = self.rng(step_index);
        let mut next_self = self.clone();
        next_self.input_system.step(self, engine);
        let (input_step_index, input) = &next_self.input_system.latest_input;
        next_self.received_damage_this_step = (false, 0);
        if self.is_dead() {
            next_self.receiving_damage_until = None;
            if input.respawn {
                let new_health = self.stats.max_health();
                engine.register_game_event(GameEvent::PlayerHealth(
                    self.player_id.clone(),
                    new_health,
                ));
                next_self.record.current_health = new_health;
            }
            return next_self;
        }
        // velocity in the last frame based on movement
        let last_velocity = self.velocity.clone();
        let body = self.rect();
        let can_jump = actor::on_platform(body, engine);

        if let Some(receiving_damage_until) = self.receiving_damage_until {
            if step_index >= &receiving_damage_until {
                next_self.receiving_damage_until = None;
            }
        } else {
            for entity in engine.entities_by_type::<MobEntity>() {
                if !entity.rect().intersect(self.rect()).is_empty() {
                    // receiving damage
                    let knockback_dir = if entity.center().x > self.center().x {
                        -1
                    } else {
                        1
                    };
                    next_self.knockback_until = Some((knockback_dir, step_index + KNOCKBACK_STEPS));
                    next_self.receiving_damage_until = Some(step_index + DAMAGE_IFRAME_STEPS);
                    next_self.weightless_until = Some(step_index + (KNOCKBACK_STEPS / 2));
                    let damage_amount = damage_calc::compute_damage(
                        &Ability::Strength,
                        &PlayerStats::default(),
                        &self.stats,
                        &mut rng,
                    );
                    next_self.received_damage_this_step = (true, damage_amount);
                    if damage_amount > 0 {
                        engine.register_game_event(GameEvent::PlayerAbilityExp(
                            self.id,
                            Ability::Health,
                            damage_amount,
                        ));
                    }
                    if next_self.record.current_health <= damage_amount {
                        next_self.record.current_health = 0;
                        // player has died
                        // TODO: move to respawn map
                        engine.register_game_event(GameEvent::PlayerHealth(
                            next_self.player_id.clone(),
                            0,
                        ));
                    } else {
                        next_self.record.current_health -= damage_amount;
                        engine.register_game_event(GameEvent::PlayerHealth(
                            next_self.player_id.clone(),
                            next_self.record.current_health,
                        ));
                    }
                    break;
                }
            }
        }
        if let Some(showing_emoji_until) = self.showing_emoji_until {
            if step_index >= &showing_emoji_until {
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

        if let Some((direction, until)) = self.knockback_until {
            next_self.velocity.x += direction * 100;
            next_self.velocity.y = 200;
            if step_index >= &until {
                next_self.knockback_until = None;
            }
        } else {
            if input.move_left {
                next_self.velocity.x -= 100;
                next_self.facing_left = true;
            }
            if input.move_right {
                next_self.velocity.x += 100;
                next_self.facing_left = false;
            }
            if !input.move_left && !input.move_right {
                // accelerate toward 0.0
                next_self.velocity.x = last_velocity.x.signum()
                    * (last_velocity.x.abs() - last_velocity.x.abs().min(100));
            }
        }
        if input.enter_portal {
            for entity in engine.entities_by_type::<PortalEntity>() {
                if entity.can_enter(self) {
                    engine.register_game_event(GameEvent::PlayerEnterPortal {
                        player_id: self.player_id.clone(),
                        entity_id: self.id,
                        from_map: entity.from.clone(),
                        to_map: entity.to.clone(),
                        requested_spawn_pos: None,
                    });
                    break;
                }
            }
        }
        if input.pick_up {
            engine.register_game_event(GameEvent::PlayerPickUpRequest(self.id));
        }
        if let Some(weightless_until) = self.weightless_until {
            if step_index >= &weightless_until {
                next_self.weightless_until = None;
            }
            next_self.velocity.y += -20;
        } else {
            next_self.velocity.y += -20;
        }
        if let Some(attacking_until) = self.attacking_until {
            if step_index >= &attacking_until {
                next_self.attacking_until = None;
            }
        }

        // check if the player is standing on a platform
        let jump = input.jump && can_jump && last_velocity.y == 0;
        let jump_down = input.jump_down && can_jump && last_velocity.y == 0;
        if jump {
            next_self.velocity.y = 380;
            next_self.weightless_until = Some(step_index + 4);
        } else if can_jump && last_velocity.y <= 0 {
            next_self.velocity.y = 0;
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
            let projectile = EngineEntity::Rect(projectile);
            let damage =
                MobDamageEntity::new_with_entity(rng.random(), &projectile, Ability::Strength);
            engine.spawn_entity(projectile, None, false);
            engine.spawn_entity(EngineEntity::MobDamage(damage), None, false);
        }

        let lower_speed_limit = IVec2::new(-250, -350);
        let upper_speed_limit = IVec2::new(250, 700);
        next_self.velocity = next_self
            .velocity
            .clamp(lower_speed_limit, upper_speed_limit);

        if jump_down {
            next_self.position.y = (self.position.y - 4).max(0);
            next_self
        } else {
            let x_pos = actor::move_x(
                self.rect(),
                next_self.velocity.x / STEPS_PER_SECOND_I32,
                engine,
            );
            let map_size = engine.size().clone();
            let platforms = engine.entities_by_type::<PlatformEntity>();
            let y_pos = actor::move_y(
                self.rect(),
                next_self.velocity.y / STEPS_PER_SECOND_I32,
                &platforms.collect::<Vec<_>>(),
                map_size,
            );
            next_self.position.x = x_pos;
            next_self.position.y = y_pos;
            next_self
        }
    }
}
