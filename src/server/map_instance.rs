use std::collections::HashMap;

use macroquad::prelude::Vec2;

use game_test::action::PlayerAction;
use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;
use game_test::Mob;

use crate::send_to_player_err;
use crate::PlayerRecord;
use crate::STATE;

use super::send_to_player;
use super::Actor;
use super::Player;
use super::WriteRequest;
use super::DB_HANDLER;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    mob_counter: u64,
    pub map: MapData,
    pub players: HashMap<String, Player>,
    pub mobs: Vec<Mob>,
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
    pub async fn add_player(&mut self, player_record: PlayerRecord) {
        let mut player = Player::new(player_record);
        player.position = self.map.spawn_location;
        player.position.y -= player.size.y / 2.;

        let player_id = player.id.clone();
        let body = player.body();
        // send new player position to themselves
        tokio::spawn(async move {
            send_to_player(&player_id, Response::PlayerChange(body)).await;
        });
        let player_id = player.id.clone();
        // send the map state to the new player
        let map_state = Response::MapState(self.mobs.clone());
        tokio::spawn(async move {
            send_to_player(&player_id, map_state).await;
        });
        let player_id = player.id.clone();
        // send other player positions to the new player
        for other_player in self.players.values() {
            let body = other_player.body();
            let state = other_player.state();
            {
                let player_id = player_id.clone();
                tokio::spawn(async move {
                    send_to_player(&player_id, Response::PlayerChange(body)).await;
                });
            }
            let player_id = player_id.clone();
            tokio::spawn(async move {
                send_to_player(&player_id, Response::PlayerData(state)).await;
            });
        }
        // notify other players of the new player
        self.broadcast(Response::PlayerChange(player.body()), None)
            .await;
        self.broadcast(Response::PlayerData(player.state()), None)
            .await;
        self.players.insert(player_id, player);
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);
        self.broadcast(Response::PlayerRemoved(player_id.to_string()), None)
            .await;
    }

    pub async fn set_player_action(&mut self, player_id: &str, player_action: PlayerAction) {
        let player = self.players.get_mut(player_id);
        if player.is_none() {
            println!("Player is not on this map: {player_id} !");
            return;
        }
        let player = player.unwrap();
        // broadcast input changes to the rest of the map
        player.position = player_action.position.unwrap();
        player.velocity = player_action.velocity.unwrap();
        player.action.update(player_action.clone());
        let mut body = player.body();
        body.action = Some(player.action.clone());
        let player_change = Some(Response::PlayerChange(body.clone()));
        // broadcast the change after stepping
        if let Some(player_change) = player_change {
            self.broadcast(player_change, Some(player_id)).await;
        }
    }

    pub async fn step(&mut self, step_len: f32) {
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
                self.mobs.push(Mob {
                    id: self.mob_counter,
                    mob_type: spawn.mob_type,
                    position: Vec2::new(
                        rand::random_range(spawn.position.x..spawn.position.x + spawn.size.x),
                        rand::random_range(spawn.position.y..spawn.position.y + spawn.size.y),
                    ),
                    velocity: Vec2::ZERO,
                    size: Vec2::new(50., 50.),
                    max_velocity: 200.,

                    moving_to: None,
                    move_start: 0.,
                });
            }
        }
        // step our mobs and send any relevant changes to the players
        let mut mob_changes = vec![];
        for mob in &mut self.mobs {
            let is_moving = mob.moving_to.is_none();
            mob.step_physics(step_len, &self.map);
            if is_moving != mob.moving_to.is_none() {
                // send to player
                mob_changes.push(Response::MobChange(mob.id, mob.moving_to));
            }
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
            player.action = player.action.clone().step_action(player, step_len);
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
                        let mut new_record = player.record.clone();
                        new_record.current_map = to_map.clone();
                        DB_HANDLER.write().await.write(WriteRequest {
                            table: "players".to_string(),
                            key: player.id.clone(),
                            // TODO: handle this unwrap more cleanly
                            value: bincode::serialize(&new_record).unwrap(),
                            callback: Some(Box::pin(async move {
                                STATE.player_change_map(&player_id, &to_map).await;
                                send_to_player(&player_id, Response::ChangeMap(to_map)).await;
                            })),
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
                let map_state = Response::MapState(self.mobs.clone());
                tokio::spawn(async move {
                    send_to_player(&player_id, map_state).await;
                });
            }
        }
    }
}
