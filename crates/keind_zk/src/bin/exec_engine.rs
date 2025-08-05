use game_common::prelude::*;
use keind::prelude::*;
use zkpo::prelude::*;

/// A program that runs the keind game engine
/// in zk for some number of steps with a few
/// entities present.
fn main() -> anyhow::Result<()> {
    // TODO: take id as input to program
    // maybe export this as a lib
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
            size: IVec2::new(200, 20),
            ..Default::default()
        },
        vec![],
    );
    mob_spawner.spawn_data.max_count = 30;
    mob_spawner.spawn_data.mob_type = 1;
    let engine_events: Vec<EngineEvent<KeindGameLogic>> = vec![
        EngineEvent::SpawnEntity {
            entity: RefPointer::new(platform.into()),
            is_non_determinism: true,
        },
        EngineEvent::SpawnEntity {
            entity: RefPointer::new(mob_spawner.into()),
            is_non_determinism: true,
        },
    ];
    let step_count: u64 = 3;

    let program = keind_zk::ZKEngineProgram;
    println!("Executing zk program...");
    let exe = program.execute(&bincode::serialize(&(step_count, engine_events))?, None)?;
    println!("Generated argument of execution");
    let out = program.agent().verify(&*exe)?;
    println!("Verified argument of execution with output data: {out:?}");
    Ok(())
}
