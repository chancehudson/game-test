use std::collections::HashMap;

use bevy::math::Vec2;
use game_test::action::PlayerBody;

use super::mob::ServerMob;
use game_test::action::PlayerAction;
use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;

use crate::send_to_player_err;
use crate::PlayerRecord;
use crate::STATE;

use super::send_to_player;
use super::Actor;
use super::Player;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    mob_counter: u64,
    pub map: MapData,
    pub players: HashMap<String, Player>,
    pub mobs: Vec<ServerMob>,
    last_broadcast: f32,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            mob_counter: 0, // TODO: give each map it's own region of distinct id's
            map,
            players: HashMap::new(),
            mobs: vec![],
            last_broadcast: 0.,
        }
    }

    pub async fn broadcast(&self, r: Response, exclude: Option<&str>) {
        for player_notif in self.players.values() {
            let player_notif_id = player_notif.id.clone();
            if let Some(exclude) = exclude {
                if exclude == &player_notif_id {
                    continue;
                }
            }
            let r = r.clone();
            let map_name = self.map.name.clone();
            // TODO: figure out this stupid fucking borrow cycle nonsense
            tokio::spawn(async move {
                if let Err(e) = send_to_player_err(&player_notif_id, r).await {
                    println!("Error broadcasting on map {}: {:?}", map_name, e);
                }
            });
        }
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(&mut self, player_record: PlayerRecord) -> PlayerBody {
        let mut player = Player::new(player_record);
        player.position = self.map.spawn_location;
        player.position.y -= player.size.y / 2.;

        let player_id = player.id.clone();
        let body = player.body();
        // send new player position to themselves
        tokio::spawn(async move {
            send_to_player(&player_id, Response::PlayerChange(body, None)).await;
        });
        let player_id = player.id.clone();
        // send the map state to the new player
        let map_state =
            Response::MapState(self.mobs.clone().into_iter().map(|v| v.into()).collect());
        tokio::spawn(async move {
            send_to_player(&player_id, map_state).await;
        });
        let player_id = player.id.clone();
        // send other player positions to the new player
        for other_player in self.players.values() {
            let body = other_player.body();
            let state = other_player.state();
            let player_id = player_id.clone();
            tokio::spawn(async move {
                send_to_player(&player_id, Response::PlayerData(state, body)).await;
            });
        }
        // notify other players of the new player
        self.broadcast(Response::PlayerChange(player.body(), None), None)
            .await;
        self.broadcast(Response::PlayerData(player.state(), player.body()), None)
            .await;
        let body = player.body();
        self.players.insert(player_id, player);
        body
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);
        self.broadcast(Response::PlayerRemoved(player_id.to_string()), None)
            .await;
    }

    pub async fn set_player_action(
        &mut self,
        player_id: &str,
        player_action: PlayerAction,
        position: Vec2,
        velocity: Vec2,
    ) {
        let player = self.players.get_mut(player_id);
        if player.is_none() {
            println!("Player is not on this map: {player_id} !");
            return;
        }
        let player = player.unwrap();
        // broadcast input changes to the rest of the map
        player.position = position;
        player.velocity = velocity;
        player.action.update(player_action.clone());
        let mut body = player.body();
        body.action = Some(player.action.clone());
        let player_change = Some(Response::PlayerChange(body.clone(), None));
        // broadcast the change after stepping
        if let Some(player_change) = player_change {
            self.broadcast(player_change, Some(player_id)).await;
        }
    }

    pub async fn tick(&mut self) {
        for spawn in &self.map.mob_spawns {
            if timestamp() - spawn.last_spawn < 10.0 {
                continue;
            }
            if self.mobs.len() >= spawn.max_count {
                continue;
            }
            let spawn_count = rand::random_range(0..=spawn.max_count);
            for _ in 0..spawn_count {
                self.mob_counter += 1;
                let mut mob = ServerMob::new(self.mob_counter, spawn.mob_type);
                mob.position = Vec2::new(
                    rand::random_range(spawn.position.x..spawn.position.x + spawn.size.x),
                    rand::random_range(spawn.position.y..spawn.position.y + spawn.size.y),
                );
                mob.next_position = mob.position;
                self.mobs.push(mob);
            }
        }
        // step our mobs and send any relevant changes to the players
        let mut mob_changes = vec![];
        for mob in &mut self.mobs {
            // handle mob aggro to player
            if let Some(aggro_player_id) = mob.aggro_to.as_ref() {
                if let Some(player) = self.players.get(aggro_player_id) {
                    mob.moving_dir = if player.position.x > mob.next_position.x {
                        Some(1.)
                    } else {
                        Some(-1.)
                    };
                } else {
                    // player has moved off map, deaggro
                    mob.aggro_to = None;
                }
            }
            mob.tick(&self.map);
            // if is_moving != mob.moving_to.is_none() {
            // send to player
            mob_changes.push(Response::MobChange(mob.clone().into()));
            // }
        }
        for player in self.players.values() {
            let player_id = player.id.clone();
            let mob_changes = mob_changes.clone();
            tokio::spawn(async move {
                for change in mob_changes {
                    send_to_player(&player_id, change).await;
                }
            });
        }
        // TODO: in parallel
        for player in self.players.values_mut() {
            let enter_portal = player.action.enter_portal;
            // let attack = player.action.attack;
            // player.action = player.action.clone().step_action(player, step_len);
            // if attack {
            //     let direction_sign = if player.action.facing_left { -1.0 } else { 1.0 };
            //     // look for mobs nearby in the direction the player is facing
            //     let attack_range_start =
            //         player.body().position.clone() + Vec2::new(player.body().size.x / 2.0, 0.0);
            //     let attack_range_end = attack_range_start
            //         + Vec2::new(direction_sign * player.body().size.x, player.body().size.y);
            //     let attack_rect = bevy::math::Rect::new(
            //         attack_range_start.x,
            //         attack_range_start.y,
            //         attack_range_end.x,
            //         attack_range_end.y,
            //     );
            //     for mob in &mut self.mobs {
            //         if attack_rect.intersect(mob.rect()).is_empty() {
            //             continue;
            //         }
            //         // mob is in range, deal damage
            //         let damage_amount = 2;
            //         mob.hit(&player.body(), damage_amount);
            //         if mob.health == 0 {
            //             // mob has died
            //         }
            //         let player_id = player.id.clone();
            //         let mob_inner = mob.clone();
            //         tokio::spawn(async move {
            //             send_to_player(&player_id, Response::MobChange(mob_inner)).await;
            //         });
            //         let player_id = player.id.clone();
            //         let mob_id = mob.id;
            //         tokio::spawn(async move {
            //             send_to_player(&player_id, Response::MobDamage(mob_id, damage_amount))
            //                 .await;
            //         });
            //         break;
            //     }
            // }
            if enter_portal {
                // determine if the player is overlapping a portal
                for portal in &self.map.portals {
                    if portal
                        .rect()
                        .contains(player.position + Vec2::new(15., 15.))
                    {
                        // user is moving
                        let player_id = player.id.clone();
                        let to_map = portal.to.clone();
                        let from_map = self.map.name.clone();
                        let mut new_record = player.record.clone();
                        new_record.current_map = to_map.clone();
                        tokio::spawn(async move {
                            if let Err(e) =
                                PlayerRecord::change_map(player_id.clone(), &from_map, &to_map)
                                    .await
                            {
                                println!("Error changing map: {:?}", e);
                            } else {
                                STATE.player_change_map(&player_id, &to_map).await;
                                send_to_player(&player_id, Response::ChangeMap(to_map)).await;
                            }
                        });
                        break;
                    }
                }
            }
        }
        // for player in self.players.values_mut() {
        //     player.step_physics(step_len, &self.map);
        // }

        if timestamp() - self.last_broadcast > 1.0 {
            self.last_broadcast = timestamp();
            for player in self.players.values() {
                let player_id = player.id.clone();
                let map_state =
                    Response::MapState(self.mobs.clone().into_iter().map(|v| v.into()).collect());
                tokio::spawn(async move {
                    send_to_player(&player_id, map_state).await;
                });
            }
        }
    }
}
