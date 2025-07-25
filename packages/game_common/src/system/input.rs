use serde::Deserialize;
use serde::Serialize;

use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InputSystem {
    // step_index, input
    pub latest_input: (u64, EntityInput),
}

impl Default for InputSystem {
    fn default() -> Self {
        Self {
            latest_input: (0, EntityInput::default()),
        }
    }
}

impl InputSystem {
    pub fn step<T: GameEngine>(&mut self, entity: &impl EEntity, engine: &T) {
        // let current_step_input_maybe = engine
        //     .engine_events_by_step
        //     .get(&engine.step_index)
        //     .and_then(|v| {
        //         for event in v {
        //             match event {
        //                 EngineEvent::Input {
        //                     input,
        //                     entity_id,
        //                     universal: _,
        //                 } => {
        //                     if *entity_id == entity.id() {
        //                         return Some(input.clone());
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         None
        //     });
        // if let Some(input) = current_step_input_maybe {
        //     self.latest_input = (engine.step_index, input);
        // }
    }
}
