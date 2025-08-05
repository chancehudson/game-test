#[cfg(not(target_os = "zkvm"))]
pub mod lib {
    use std::sync::OnceLock;

    use sp1_sdk::{HashableKey, ProverClient};
    use zkpo::prelude::*;

    pub struct ZKNoopProgram;

    impl ZKProgram for ZKNoopProgram {
        fn id(&self) -> &[u8; 32] {
            static HASH: OnceLock<[u8; 32]> = OnceLock::new();
            HASH.get_or_init(|| {
                let client = ProverClient::from_env();
                let (_pk, vk) = client.setup(self.elf());
                vk.hash_bytes()
            })
        }

        fn elf(&self) -> &[u8] {
            include_bytes!("../elf/noop")
        }

        fn name(&self) -> Option<&str> {
            Some("noop test program")
        }

        fn agent(&self) -> Option<&dyn zkpo::ZKAgent> {
            None
        }
    }
}

#[cfg(not(target_os = "zkvm"))]
pub use lib::*;
