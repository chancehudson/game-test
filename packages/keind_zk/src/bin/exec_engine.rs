use zkpo::prelude::*;

#[cfg(not(target_os = "zkvm"))]
fn main() -> anyhow::Result<()> {
    let program = keind_zk::ZKEngineProgram;
    println!("Initializing agent...");
    let agent = ZKSPOneAgent::default();
    println!("Executing...");
    let out = agent.execute(&[], &program)?;
    println!("Generated argument of execution");
    let out_data = agent.verify(&*out)?;
    println!("Verified argument of execution with output data: {out_data:?}");
    Ok(())
}
