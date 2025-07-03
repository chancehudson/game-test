use std::collections::HashMap;

use bevy::prelude::*;

use engine::GameEngine;
use engine::entity::EEntity;
use engine::entity::mob::MobEntity;
use game_common::STEP_DELAY;

pub struct Interpolation {
    pub from_step: u64,
    pub to_step: u64,
    pub start_position: IVec2,
    pub diff_position: IVec2,
}

// id keyed to start step, end step, relative distance remaining?
#[derive(Resource, Default)]
pub struct InterpolatingEntities(pub HashMap<u128, Interpolation>);

/// Update the InterpolatingEntities resource as needed
pub fn interpolate_mobs(
    last_mobs: Vec<MobEntity>,
    engine: &GameEngine,
    player_entity_id: u128,
    interpolating_entities: &mut ResMut<InterpolatingEntities>,
) {
    // now we look at our current entities and see if any mobs became aggro against
    // a different player
    // if so we need to start showing that mob in the past
    for mob in last_mobs {
        if let Some(current_mob_entity) = engine.entity_by_id::<MobEntity>(&mob.id, None) {
            if let Some(past_entity) =
                engine.entity_by_id::<MobEntity>(&mob.id, Some(engine.step_index - STEP_DELAY / 2))
            {
                // interpolate position over N steps
                if current_mob_entity.aggro_to.is_none()
                    && mob.aggro_to.is_some()
                    && mob.aggro_to.unwrap().0 != player_entity_id
                {
                    // TODO: handle deaggro (moving back to current time)
                    // let current_position = mob.position;
                }
                if current_mob_entity.aggro_to.is_some()
                    && current_mob_entity.aggro_to.unwrap().0 != player_entity_id
                    && mob.aggro_to.is_none()
                {
                    let interp_steps = STEP_DELAY / 2;
                    // handle aggro to another player
                    let current_position = current_mob_entity.position;
                    let past_position = past_entity.position();
                    let step_amount =
                        (past_position - current_position) / IVec2::splat(interp_steps as i32);
                    interpolating_entities.0.insert(
                        mob.id,
                        Interpolation {
                            from_step: engine.step_index,
                            to_step: engine.step_index + interp_steps,
                            start_position: current_position,
                            diff_position: step_amount,
                        },
                    );
                }
            }
        }
    }
}
