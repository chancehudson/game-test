/// GameEvents are events that need to be persisted in the database
/// Things like experience, map changes, position changes, etc
use db::AbilityExpRecord;

use crate::entity::EEntity;
use crate::entity::item::ItemEntity;

use super::GameEngine;
use super::entity::player::PlayerEntity;
use super::game_event::GameEvent;

/// The default handler may mutate the engine without using engine events
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
            // player will be despawned immediately
            engine.entities.remove(entity_id);
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
        GameEvent::PlayerPickUpRequest(player_entity_id) => {
            if let Some(player_entity) = engine.entities.get(player_entity_id).cloned() {
                let game_events_sender = engine.game_events.0.clone();
                // there are quirks with using entities_by_type in the default handler
                // see GameEngine::step
                for item in engine
                    .entities_by_type::<ItemEntity>()
                    .cloned()
                    .collect::<Vec<_>>()
                {
                    if item.rect().intersect(player_entity.rect()).is_empty() {
                        continue;
                    }
                    // otherwise pick up the item
                    engine.entities.remove(&item.id);
                    // mark user as having object
                    game_events_sender
                        .send(GameEvent::PlayerPickUp(
                            player_entity
                                .extract_ref::<PlayerEntity>()
                                .unwrap()
                                .player_id
                                .clone(),
                            item.item_type,
                            item.count,
                        ))
                        .unwrap();
                    return;
                }
            }
        }
        GameEvent::PlayerPickUp(_, _, _) => {}
        GameEvent::PlayerHealth(_, _) => {}
        GameEvent::Message(_, _) => {}
    }
}
