use risc0_zkvm::{Executor, ExecutorEnv, Receipt, default_prover};
use zk_risczero_methods::PROVE_ELF;

fn main() {
    let env = ExecutorEnv::builder()
        .write(&1u64)
        .unwrap()
        .build()
        .unwrap();
    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let out = prover.prove(env, PROVE_ELF).unwrap();
    println!("{:?}", out.stats);
}
