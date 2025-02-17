use macroquad::prelude::Vec2;

use game_test::action::PlayerAction;
use game_test::action::Response;
use game_test::engine::Engine;
use game_test::map::MapData;
use game_test::player::Player;

use crate::send_to_player_err;
use crate::PlayerRecord;
use crate::STATE;

use super::send_to_player;
use super::WriteRequest;
use super::DB_HANDLER;

/// A distinct instance of a map. Each map is it's own game instance
/// responsible for player communication, mob management, and physics.
pub struct MapInstance {
    pub engine: Engine,
    last_broadcast: u64,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            // TODO: give each map it's own region of distinct id's
            engine: Engine::new(map, 0, 0),
            last_broadcast: 0,
        }
    }

    pub async fn broadcast(&self, r: Response, exclude: Option<&str>) {
        for player_notif in self.engine.players.values() {
            let player_notif_id = player_notif.id.clone();
            if let Some(exclude) = exclude {
                if exclude == &player_notif_id {
                    continue;
                }
            }
            let r = r.clone();
            let map_name = self.engine.map.name.clone();
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
        let player_id = player_record.id.clone();
        self.engine.action(Box::new(move |engine| {
            let mut player = Player::new(player_id.clone());
            player.position = engine.map.spawn_location;
            player.position.y -= player.size.y / 2.;
            engine.players.insert(player_id.clone(), player);
        }));

        let current_tick = self.engine.current_tick;
        let player_id = player_record.id.clone();
        let mut player = Player::new(player_id.clone());
        let body = player.body();
        // send new player position to themselves
        tokio::spawn(async move {
            send_to_player(&player_id, Response::PlayerChange(current_tick + 1, body)).await;
        });
        let player_id = player.id.clone();
        // send the map state to the new player
        let map_state = Response::MapState(
            current_tick,
            self.engine.mobs.clone(),
            self.engine.players.clone(),
        );
        tokio::spawn(async move {
            send_to_player(&player_id, map_state).await;
        });
        let player_id = player.id.clone();
        // send other player positions to the new player
        for other_player in self.engine.players.values() {
            let body = other_player.body();
            // let state = other_player.state();
            {
                let player_id = player_id.clone();
                tokio::spawn(async move {
                    send_to_player(&player_id, Response::PlayerChange(current_tick, body)).await;
                });
            }
            // let player_id = player_id.clone();
            // tokio::spawn(async move {
            //     send_to_player(&player_id, Response::PlayerData(state)).await;
            // });
        }
        // notify other players of the new player
        self.broadcast(
            Response::PlayerChange(current_tick + 1, player.body()),
            None,
        )
        .await;
        // self.broadcast(Response::PlayerData(player.state()), None)
        //     .await;
    }

    pub async fn remove_player(&mut self, player_id: &str) {
        self.engine.players.remove(player_id);
        self.broadcast(Response::PlayerRemoved(player_id.to_string()), None)
            .await;
    }

    pub async fn set_player_action(&mut self, player_id: &str, player_action: PlayerAction) {
        let player = self.engine.players.get_mut(player_id);
        if player.is_none() {
            println!("Player is not on this map: {player_id} !");
            return;
        }
        let player = player.unwrap();
        // if the player has begun moving or stopped moving broadcast
        // to the rest of the map
        let mut player_change = None;
        let action = player.current_action.clone();
        if action.move_left != player_action.move_left
            || action.move_right != player_action.move_right
            || action.jump != player_action.jump
            || action.downward_jump != player_action.downward_jump
        {
            let mut body = player.body();
            body.action = Some(player_action.clone());
            player_change = Some(Response::PlayerChange(
                self.engine.current_tick + 1,
                body.clone(),
            ));
        }
        println!(
            "action requested tick: {}, current tick: {}",
            player_action.tick, self.engine.current_tick
        );
        player.next_action = player_action;
        // broadcast the change after stepping
        if let Some(player_change) = player_change {
            self.broadcast(player_change, Some(player_id)).await;
        }
    }

    pub async fn tick(&mut self) {
        let old_mobs = self.engine.mobs.clone();
        self.engine.tick();
        // TODO: send mobs changes

        for player in self.engine.players.values_mut() {
            if player.current_action.enter_portal {
                // determine if the player is overlapping a portal
                for portal in &self.engine.map.portals {
                    if portal
                        .rect()
                        .contains(player.position + Vec2::new(15., 15.))
                    {
                        // user is moving
                        let player_id = player.id.clone();
                        let to_map = portal.to.clone();
                        let record = PlayerRecord::player_by_id(player_id.clone())
                            .await
                            .unwrap()
                            .unwrap();
                        let mut new_record = record.clone();
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
        if self.engine.current_tick - self.last_broadcast > 1 {
            self.last_broadcast = self.engine.current_tick;
            let current_tick = self.engine.current_tick;
            for player in self.engine.players.values() {
                let player_id1 = player.id.clone();
                let player_id2 = player.id.clone();
                let body = player.body();
                let map_state = Response::MapState(
                    current_tick,
                    self.engine.mobs.clone(),
                    self.engine.players.clone(),
                );
                tokio::spawn(async move {
                    send_to_player(&player_id1, Response::PlayerChange(current_tick, body)).await;
                });
                tokio::spawn(async move {
                    send_to_player(&player_id2, map_state).await;
                });
            }
        }
    }
}
