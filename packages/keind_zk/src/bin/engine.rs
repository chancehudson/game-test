#![no_main]
sp1_zkvm::entrypoint!(main);

use game_common::prelude::*;
use keind::prelude::*;

pub fn main() {
    let engine_id = sp1_zkvm::io::read::<u128>();

    let mut engine = GameEngine::<KeindGameLogic>::default();

    let platform = PlatformEntity::new(
        BaseEntityState {
            id: engine.generate_id(),
            position: IVec2::new(200, 200),
            size: IVec2::new(200, 25),
            ..Default::default()
        },
        vec![],
    );
    let mut mob_spawner = MobSpawnEntity::new(
        BaseEntityState {
            id: engine.generate_id(),
            position: platform.position() + IVec2::new(0, platform.size().y + 20),
            size: IVec2::new(200, 20),
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
    engine.step_to(&5);
}
