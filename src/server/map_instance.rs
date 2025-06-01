use std::collections::HashMap;
use std::sync::Arc;

use bevy::math::Vec2;
use game_test::action::PlayerBody;
use game_test::TICK_RATE_MS;
use game_test::TICK_RATE_S_F32;

use super::mob::ServerMob;
use game_test::action::PlayerAction;
use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;

use crate::game::MapGameAction;
use crate::network;
use crate::PlayerRecord;

use super::Actor;
use super::Player;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    network_server: Arc<network::Server>,
    mob_counter: u64,
    pub map: MapData,
    pub players: HashMap<String, Player>,
    pub mobs: Vec<ServerMob>,
    last_broadcast: f64,
}

impl MapInstance {
    pub fn new(map: MapData, network_server: Arc<network::Server>) -> Self {
        Self {
            network_server,
            mob_counter: 0, // TODO: give each map it's own region of distinct id's
            map,
            players: HashMap::new(),
            mobs: vec![],
            last_broadcast: 0.,
        }
    }

    pub async fn check_for_disconnects(&mut self) {
        let mut disconnected_ids = vec![];
        for player_id in self.players.keys() {
            if self
                .network_server
                .socket_by_player_id(player_id)
                .await
                .is_none()
            {
                disconnected_ids.push(player_id.clone());
            }
        }
        for id in disconnected_ids {
            self.remove_player(&id).await;
        }
    }

    pub async fn broadcast(&self, r: Response, exclude: Option<String>) {
        let player_ids = self.players.keys().cloned().collect::<Vec<String>>();
        let network_server = self.network_server.clone();
        let map_name = self.map.name.clone();
        tokio::spawn(async move {
            for id in player_ids {
                if let Some(exclude) = exclude.clone() {
                    if exclude == id {
                        continue;
                    }
                }
                if let Err(e) = network_server.send_to_player_err(&id, r.clone()).await {
                    println!("Error broadcasting on map {}: {:?}", map_name, e);
                }
            }
        });
    }

    /// insert our new player into the map and send the current state
    pub async fn add_player(&mut self, player_record: PlayerRecord) -> PlayerBody {
        let mut player = Player::new(player_record);
        player.position = self.map.spawn_location;
        player.position.y -= player.size.y / 2.;

        let player_id = player.id.clone();
        let body = player.body();
        // send new player position to themselves
        let network_server = self.network_server.clone();
        tokio::spawn(async move {
            network_server
                .send_to_player(&player_id, Response::PlayerChange(body, None))
                .await;
        });
        let player_id = player.id.clone();
        // send the map state to the new player
        let map_state =
            Response::MapState(self.mobs.clone().into_iter().map(|v| v.into()).collect());
        let network_server = self.network_server.clone();
        tokio::spawn(async move {
            network_server.send_to_player(&player_id, map_state).await;
        });
        let player_id = player.id.clone();
        // send other player positions to the new player
        for other_player in self.players.values() {
            let body = other_player.body();
            let state = other_player.state();
            let player_id = player_id.clone();
            let network_server = self.network_server.clone();
            tokio::spawn(async move {
                network_server
                    .send_to_player(&player_id, Response::PlayerData(state, body))
                    .await;
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
    ) -> Option<MapGameAction> {
        let player = self.players.get_mut(player_id);
        if player.is_none() {
            println!("Player is not on this map: {player_id} !");
            return None;
        }
        // client authoritatively provides player position
        // TODO: validate that positions make sense, player isn't moving
        // too fast, flying, teleporting, etc
        let player = player.unwrap();
        player.position = position;
        player.velocity = velocity;
        player.action.update(player_action.clone());
        let mut body = player.body();
        body.action = Some(player.action.clone());
        let player_change = Some(Response::PlayerChange(body.clone(), None));
        // HACK: step the action by 1 frame to simulate key up
        // events for e.g. the jump action
        let enter_portal = player.action.enter_portal;
        let attack = player.action.attack;
        player.action = player.action.clone().step_action(player, 16. / 1000.);
        // handle actions taken by the player
        if attack {
            let direction_sign = if player.action.facing_left { -1.0 } else { 1.0 };
            // look for mobs nearby in the direction the player is facing
            let attack_range_start =
                player.body().position.clone() + Vec2::new(player.body().size.x / 2.0, 0.0);
            let attack_range_end = attack_range_start
                + Vec2::new(direction_sign * player.body().size.x, player.body().size.y);
            let attack_rect = bevy::math::Rect::new(
                attack_range_start.x,
                attack_range_start.y,
                attack_range_end.x,
                attack_range_end.y,
            );
            for mob in &mut self.mobs {
                if attack_rect.intersect(mob.rect()).is_empty() {
                    continue;
                }
                // don't allow multiple players to attack the same mob
                if let Some(aggro_to) = mob.aggro_to.as_ref() {
                    if aggro_to != &body.id {
                        continue;
                    }
                }
                // mob is in range, deal damage
                let damage_amount = 2;
                mob.hit(&player.body(), damage_amount);
                if mob.health == 0 {
                    // mob has died
                } else {
                    let player_id = player.id.clone();
                    let mob_inner = mob.clone();
                    let network_server = self.network_server.clone();
                    tokio::spawn(async move {
                        network_server
                            .send_to_player(&player_id, Response::MobChange(mob_inner.into()))
                            .await;
                    });
                }
                let player_id = player.id.clone();
                let mob_id = mob.id;
                let network_server = self.network_server.clone();
                tokio::spawn(async move {
                    network_server
                        .send_to_player(&player_id, Response::MobDamage(mob_id, damage_amount))
                        .await;
                });
                break;
            }
        }
        if enter_portal {
            // determine if the player is overlapping a portal
            for portal in &self.map.portals {
                if portal
                    .rect()
                    .contains(player.position + Vec2::new(15., 15.))
                {
                    return Some(MapGameAction::EnterPortal(
                        self.map.name.clone(),
                        portal.to.clone(),
                    ));
                }
            }
        }
        // broadcast the change to other players on the map
        if let Some(player_change) = player_change {
            self.broadcast(player_change, Some(player_id.to_string()))
                .await;
        }
        None
    }

    pub async fn tick(&mut self) {
        self.check_for_disconnects().await;
        self.mobs = self
            .mobs
            .iter()
            .cloned()
            .filter(|mob| mob.health > 0)
            .collect::<Vec<_>>();
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
            let network_server = self.network_server.clone();
            tokio::spawn(async move {
                for change in mob_changes {
                    network_server.send_to_player(&player_id, change).await;
                }
            });
        }
        for player in self.players.values_mut() {
            // TODO: better player position sync
            if !player.action.move_left && !player.action.move_right {
                player.velocity = player.velocity.move_towards(Vec2::ZERO, 300.);
            }
            player.step_physics(TICK_RATE_S_F32, &self.map);
        }

        if timestamp() - self.last_broadcast > 1.0 {
            self.last_broadcast = timestamp();
            for player in self.players.values() {
                let player_id = player.id.clone();
                let map_state =
                    Response::MapState(self.mobs.clone().into_iter().map(|v| v.into()).collect());
                let network_server = self.network_server.clone();
                tokio::spawn(async move {
                    network_server.send_to_player(&player_id, map_state).await;
                });
            }
        }
    }
}
