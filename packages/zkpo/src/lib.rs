use anyhow::Result;

mod r0;

/// A program to be executed.
pub trait ZKProgram {
    fn elf(&self) -> &[u8];
}

/// Uniquely identifies a program.
pub trait ZKProgramDigest {
    fn bytes(&self) -> &[u8; 32];
    fn compare(&self, program: &dyn ZKProgram) -> Result<bool>;
}

/// A prover/arithmetization agnostic argument of knowledge.
pub trait ZKProof {
    /// Opaque proving system data.
    fn cipher_bytes(&self) -> &[u8];
    /// Prover implementation.
    // #[serde(skip)]
    fn prover(&self) -> Box<dyn ZKProver>;
    /// Application designated digest data.
    fn program_digest(&self) -> &dyn ZKProgramDigest;
}

/// A structure that can
/// - create proofs provided a ZKProgram
/// - verify proofs provided a ZKProof
///
/// Each prover can verify many different programs using
/// many different proving systems.
pub trait ZKProver {
    /// Generate a proof. Inputs are expected to be serialized outside
    /// of this implementation.
    fn prove(&self, input: &[u8], program: &dyn ZKProgram) -> Result<Box<dyn ZKProof>>;
    /// Verify a proof and return the output data.
    fn verify(&self, proof: &dyn ZKProof) -> Result<Vec<u8>>;
}
