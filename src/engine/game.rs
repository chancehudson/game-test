/// GameEvents are events that need to be persisted in the database
/// Things like experience, map changes, position changes, etc
use crate::{
    db::AbilityExpRecord,
    engine::{
        entity::player::PlayerEntity,
        game_event::{EngineEvent, GameEvent},
    },
};

use super::GameEngine;

pub fn default_handler(engine: &mut GameEngine, game_event: &GameEvent) {
    // handle game events that occurred during a step
    match game_event {
        GameEvent::PlayerEnterPortal {
            player_id: _,
            entity_id,
            from_map: _,
            to_map: _,
            requested_spawn_pos: _,
        } => {
            // player will be despawned ASAP step
            engine.register_event(
                None,
                EngineEvent::RemoveEntity {
                    id: rand::random(),
                    entity_id: *entity_id,
                    universal: false,
                },
            );
        }
        GameEvent::PlayerAbilityExp(player_entity_id, ability, amount) => {
            // we'll just handle synchronizing the player entities stats here
            // database logic lives in map_instance.rs or game.rs
            if let Some(player_entity) =
                engine.entity_by_id_mut::<PlayerEntity>(&player_entity_id, None)
            {
                player_entity.stats.increment(&AbilityExpRecord {
                    player_id: player_entity.player_id.clone(),
                    amount: *amount,
                    ability: ability.clone(),
                });
            } else {
                println!("WARNING: player entity does not exist in engine for ability exp!");
            }
        }
        GameEvent::PlayerHealth(_, _) => {}
    }
}
