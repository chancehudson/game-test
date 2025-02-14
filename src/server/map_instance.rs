use std::collections::HashMap;

use macroquad::prelude::Vec2;

use game_test::action::Response;
use game_test::map::MapData;
use game_test::timestamp;
use game_test::Mob;

use super::send_to_player;
use super::Actor;
use super::Player;
use super::PLAYERS;

// spawn regions, rect with a type of mob to spawn?
pub struct MapInstance {
    mob_counter: u64,
    pub map: MapData,
    // pub player_ids: HashMap<String, ()>,
    pub actors: Vec<Box<dyn Actor + Sync + Send>>,
    pub mobs: Vec<Mob>,
}

impl MapInstance {
    pub fn new(map: MapData) -> Self {
        Self {
            mob_counter: 0, // TODO: give each map it's own region of distinct id's
            map,
            // player_ids: HashMap::new(),
            actors: vec![],
            mobs: vec![],
        }
    }

    pub fn step(&mut self, _players: &mut HashMap<String, Player>, step_len: f32) {
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
        for mob in &mut self.mobs {
            let is_moving = mob.moving_to.is_none();
            mob.step_physics(step_len, &self.map);
            if is_moving != mob.moving_to.is_none() {
                // send to player
                let map_name = self.map.name.clone();
                let mob = mob.clone();
                tokio::spawn(async move {
                    for (id, player) in PLAYERS.read().await.iter() {
                        if player.record.current_map != map_name {
                            continue;
                        }
                        let r = Response::MobChange(mob.id, mob.moving_to);
                        send_to_player(&id, r).await;
                    }
                });
            }
        }
        // step the physics
        for actor in &mut self.actors {
            actor.step_physics(step_len, &self.map);
        }
    }
}
