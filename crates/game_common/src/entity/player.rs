use bevy_math::IVec2;
use keind::prelude::*;
use rand::Rng;

use db::Ability;
use db::PlayerRecord;
use db::PlayerStats;

use crate::prelude::*;
use crate::system::weightless::WeightlessSystem;
use keind::prelude::*;

const DAMAGE_IFRAME_STEPS: u64 = 120;
const KNOCKBACK_STEPS: u64 = 10;

entity_struct!(
    KeindGameLogic,
    pub struct PlayerEntity {
        pub player_id: String, // the game id, not entity id
        pub record: PlayerRecord,
        attacking_until: Option<u64>,
        pub facing_left: bool,
        pub showing_emoji_until: Option<u64>,
        pub stats_ptr: RefPointer<PlayerStats>,
        pub received_damage_this_step: (bool, u64),
        // direction, until
        pub knockback_until: Option<(i32, u64)>,
    }
);

impl PlayerEntity {
    pub fn new_with_ids(id: u128, record: PlayerRecord, stats: PlayerStats) -> Self {
        PlayerEntity {
            state: BaseEntityState {
                id,
                position: IVec2::new(100, 100),
                size: IVec2::new(52, 52),
                player_creator_id: Some(id),
                ..Default::default()
            },
            player_id: record.id.clone(),
            stats_ptr: RefPointer::from(stats),
            record,
            systems: vec![
                RefPointer::new(GravitySystem::default().into()),
                RefPointer::new(AtomicMoveSystem::default().into()),
            ],
            ..Default::default()
        }
    }

    pub fn is_dead(&self) -> bool {
        self.record.current_health == 0
    }
}

impl SEEntity<KeindGameLogic> for PlayerEntity {
    fn step(&self, engine: &GameEngine<KeindGameLogic>, next_self: &mut Self) {
        let step_index = engine.step_index();
        let mut rng = self.rng(step_index);
        let input = engine.input_for_entity(&self.id());
        next_self.received_damage_this_step = (false, 0);
        if self.is_dead() {
            if input.respawn {
                let new_health = self.stats_ptr.max_health();
                engine.register_game_event(GameEvent::PlayerHealth(
                    self.player_id.clone(),
                    new_health,
                ));
                next_self.record.current_health = new_health;
            }
            return;
        }
        // velocity in the last frame based on movement
        let last_velocity = self.velocity().clone();
        let body = self.rect();
        let can_jump = actor::on_platform(body, engine);

        if !self.has_system::<InvincibleSystem>() {
            for entity in engine.entities_by_type::<MobEntity>() {
                if !entity.rect().intersect(self.rect()).is_empty() {
                    // receiving damage
                    let knockback_dir = if entity.center().x > self.center().x {
                        -1
                    } else {
                        1
                    };
                    next_self.knockback_until = Some((knockback_dir, step_index + KNOCKBACK_STEPS));
                    next_self.state.velocity.x += knockback_dir * 400;
                    next_self.state.velocity.y += 100;
                    engine.spawn_system(
                        self.id(),
                        RefPointer::new(
                            InvincibleSystem {
                                until_step: Some(step_index + DAMAGE_IFRAME_STEPS),
                            }
                            .into(),
                        ),
                    );
                    // engine.spawn_system(
                    //     self.id(),
                    //     RefPointer::new(
                    //         WeightlessSystem {
                    //             until_step: Some(step_index + 3),
                    //         }
                    //         .into(),
                    //     ),
                    // );
                    let damage_amount = damage_calc::compute_damage(
                        &Ability::Strength,
                        &PlayerStats::default(),
                        &*self.stats_ptr,
                        &mut rng,
                    );
                    next_self.received_damage_this_step = (true, damage_amount);
                    if damage_amount > 0 {
                        engine.register_game_event(GameEvent::PlayerAbilityExp(
                            self.id(),
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
            let emoji = EmojiEntity::new(
                BaseEntityState {
                    id,
                    position: IVec2::MAX,
                    size: IVec2::new(80, 80),
                    player_creator_id: Some(self.id()),
                    ..Default::default()
                },
                vec![
                    RefPointer::new(
                        AttachSystem {
                            attached_to: self.id(),
                            offset: self.size() + IVec2::new(-self.size().x / 2, 5),
                        }
                        .into(),
                    ),
                    RefPointer::new(
                        DisappearSystem {
                            at_step: show_until,
                        }
                        .into(),
                    ),
                ],
            );
            engine.spawn_entity(RefPointer::new(emoji.into()));
        }

        if let Some((direction, until)) = self.knockback_until {
            if step_index >= &until {
                next_self.knockback_until = None;
            }
        } else {
            if input.move_left {
                next_self.state.velocity.x -= if can_jump { 100 } else { 20 };
                next_self.facing_left = true;
            }
            if input.move_right {
                next_self.state.velocity.x += if can_jump { 100 } else { 20 };
                next_self.facing_left = false;
            }
            if !input.move_left && !input.move_right {
                // accelerate toward 0.0
                next_self.state.velocity.x -= next_self.state.velocity.x / 4;
            }
        }
        if input.enter_portal {
            for entity in engine.entities_by_type::<PortalEntity>() {
                if entity.can_enter(self) {
                    engine.register_game_event(GameEvent::PlayerEnterPortal {
                        player_id: self.player_id.clone(),
                        entity_id: self.id(),
                        from_map: entity.from.clone(),
                        to_map: entity.to.clone(),
                        requested_spawn_pos: None,
                    });
                    break;
                }
            }
        }
        if input.pick_up {
            engine.register_game_event(GameEvent::PlayerPickUpRequest(self.id()));
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
            next_self.state.velocity.y += 340;
            engine.spawn_system(
                self.id(),
                RefPointer::new(
                    WeightlessSystem {
                        until_step: Some(step_index + 2),
                    }
                    .into(),
                ),
            );
        }
        if jump_down {
            next_self.state.position.y = (next_self.state.position.y - 4).max(0);
        }
        if input.attack && self.attacking_until.is_none() {
            // 15 is the step length of the attack animation
            next_self.attacking_until = Some(step_index + 10);
            let id = rng.random();
            let move_sign = if self.facing_left { -1 } else { 1 };
            let projectile = RectEntity::new(
                BaseEntityState {
                    id,
                    position: IVec2::new(
                        self.center().x + move_sign * self.size().x / 2,
                        self.center().y,
                    ),
                    size: IVec2::new(30, 5),
                    player_creator_id: Some(self.id()),
                    velocity: IVec2::new(800 * move_sign, 0),
                    ..Default::default()
                },
                vec![
                    RefPointer::new(
                        DisappearSystem {
                            at_step: step_index + 30,
                        }
                        .into(),
                    ),
                    RefPointer::new(AtomicMoveSystem::default().into()),
                ],
            );
            let projectile = EngineEntity::from(projectile);
            let damage =
                MobDamageEntity::new_with_entity(rng.random(), &projectile, Ability::Strength);
            engine.spawn_entity(RefPointer::new(projectile));
            engine.spawn_entity(RefPointer::new(damage.into()));
        }
    }
}
