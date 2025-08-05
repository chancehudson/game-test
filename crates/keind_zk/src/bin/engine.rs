#![no_main]
sp1_zkvm::entrypoint!(main);

use game_common::prelude::*;
use keind::prelude::*;

pub fn main() {
    let input: Vec<u8> = sp1_zkvm::io::read_vec();
    let (step_count, events): (u64, Vec<EngineEvent<KeindGameLogic>>) =
        bincode::deserialize(&input).expect("failed to deserialize input");

    let mut engine = GameEngine::<KeindGameLogic>::default();
    engine.trailing_state_len = 0;
    for event in events {
        engine.register_event(None, event);
    }
    engine.step_to(&step_count);
    // TODO: step hash for engine, lazily computed based on each step events? non-determinism?
    // output an engine checksum
    //
    // let engine_out = bincode::serialize(&engine).expect("engine serialization failed");
    // sp1_zkvm::io::commit_slice(&engine_out);
    sp1_zkvm::io::commit_slice(&[engine.entity_count() as u8]);
}
