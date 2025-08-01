#![no_main]

use bevy_math::IVec2;
use risc0_zkvm::guest::env;

use game_common::prelude::*;
use keind::prelude::*;

risc0_zkvm::guest::entry!(main);

fn main() {
    let engine_seed: u64 = env::read();

    let mut engine =
        GameEngine::<KeindGameLogic>::new_simple(IVec2 { x: 1000, y: 1000 }, engine_seed as u128);

    let platform = PlatformEntity::new(
        BaseEntityState {
            id: 1,
            position: IVec2::new(200, 200),
            size: IVec2::new(200, 25),
            ..Default::default()
        },
        vec![],
    );
    let mut mob_spawner = MobSpawnEntity::new(
        BaseEntityState {
            id: 2,
            position: platform.position() + IVec2::new(0, platform.size().y + 20),
            size: IVec2::new(200, 1),
            ..Default::default()
        },
        vec![],
    );
    mob_spawner.spawn_data.max_count = 30;
    mob_spawner.spawn_data.mob_type = 1;
    engine.register_event(
        None,
        EngineEvent::SpawnEntity {
            entity: RefPointer::new(platform.into()),
            is_non_determinism: true,
        },
    );
    engine.register_event(
        None,
        EngineEvent::SpawnEntity {
            entity: RefPointer::new(mob_spawner.into()),
            is_non_determinism: true,
        },
    );
    engine.step_to(&1000);
}
