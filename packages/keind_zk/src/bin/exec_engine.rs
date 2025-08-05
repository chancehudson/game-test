use zkpo::prelude::*;

fn main() -> anyhow::Result<()> {
    let program = keind_zk::ZKEngineProgram;
    println!("Executing zk program...");
    let exe = program.execute(&[], None)?;
    println!("Generated argument of execution");
    let out = program.agent().verify(&*exe)?;
    println!("Verified argument of execution with output data: {out:?}");
    Ok(())
}
