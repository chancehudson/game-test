#[cfg(not(target_os = "zkvm"))]
pub mod zk_programs;

#[cfg(not(target_os = "zkvm"))]
pub mod lib {
    use super::zk_programs;
    pub use zk_programs::engine::ZKEngineProgram;
    pub use zk_programs::noop::ZKNoopProgram;
}

#[cfg(not(target_os = "zkvm"))]
pub use lib::*;
