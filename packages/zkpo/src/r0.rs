use anyhow::Result;
use risc0_zkvm::Digest;
use risc0_zkvm::{ExecutorEnv, Receipt, default_prover};

use crate::ZKProgram;
use crate::ZKProgramDigest;
use crate::ZKProof;
use crate::ZKProver;

#[derive(Clone, Debug)]
pub struct ZKRiscZeroProof {
    pub receipt_bytes: Vec<u8>,
}

impl ZKProof for ZKRiscZeroProof {
    fn prover(&self) -> Box<dyn ZKProver> {
        Box::new(ZKRiscZeroProver::default())
    }

    fn program_digest(&self) -> &dyn ZKProgramDigest {
        panic!();
    }

    fn cipher_bytes(&self) -> &[u8] {
        &self.receipt_bytes
    }
}

/// TODO: serialize as well, so provers can be embedded in network data
/// for declarative zk proofs.
#[derive(Clone, Default, Debug)]
pub struct ZKRiscZeroProver;

impl ZKProver for ZKRiscZeroProver {
    fn prove(&self, input: &[u8], program: &dyn ZKProgram) -> Result<Box<dyn ZKProof>> {
        // build an executor with the supplied input
        let env = ExecutorEnv::builder().write_slice(input).build()?;
        // use the default risc0 prover
        let prover = default_prover();
        // Produce a receipt by proving the specified ELF binary.
        let receipt = prover.prove(env, program.elf())?.receipt;
        Ok(Box::new(ZKRiscZeroProof {
            receipt_bytes: bincode::serialize(&receipt)?,
        }))
    }

    fn verify(&self, proof: &dyn ZKProof) -> Result<Vec<u8>> {
        let receipt = bincode::deserialize::<Receipt>(proof.cipher_bytes())?;
        let digest = Digest::from_bytes(proof.program_digest().bytes().clone());
        receipt.verify(digest)?;
        Ok(receipt.journal.bytes)
    }
}

mod tests {
    #[test]
    fn build_prove() {}
}
