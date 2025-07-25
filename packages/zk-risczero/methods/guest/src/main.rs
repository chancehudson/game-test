#![no_main]

use bevy_math::IVec2;
use game_common::prelude::*;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let engine_seed: u64 = env::read();

    let mut engine = SimpleGameEngine::new(IVec2 { x: 1000, y: 1000 }, engine_seed);

    let platform = PlatformEntity::new(
        engine.generate_id(),
        IVec2::new(200, 200),
        IVec2::new(200, 25),
    );
    let mut mob_spawner = MobSpawnEntity::new(
        engine.generate_id(),
        platform.position + IVec2::new(0, platform.size.y + 20),
        IVec2::new(200, 1),
    );
    mob_spawner.spawn_data.max_count = 30;
    mob_spawner.spawn_data.mob_type = 1;
    engine.register_event(
        None,
        EngineEvent::SpawnEntity {
            entity: EngineEntity::Platform(platform),
            universal: true,
        },
    );
    engine.register_event(
        None,
        EngineEvent::SpawnEntity {
            entity: EngineEntity::MobSpawner(mob_spawner),
            universal: true,
        },
    );
    engine.step_to(&1000);
}
