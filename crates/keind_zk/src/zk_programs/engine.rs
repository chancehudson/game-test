use std::sync::OnceLock;

use sp1_sdk::HashableKey;
use sp1_sdk::ProverClient;
use zkpo::prelude::*;

pub struct ZKEngineProgram;
impl ZKProgram for ZKEngineProgram {
    fn id(&self) -> &[u8; 32] {
        static HASH: OnceLock<[u8; 32]> = OnceLock::new();
        HASH.get_or_init(|| {
            let client = ProverClient::from_env();
            let (_pk, vk) = client.setup(self.elf());
            vk.hash_bytes()
        })
    }

    fn elf(&self) -> &[u8] {
        include_bytes!("../../elf/engine")
    }

    fn name(&self) -> Option<&str> {
        Some("engine test program")
    }

    fn agent(&self) -> &dyn ZKAgent {
        ZKSPOneAgent::singleton()
    }
}
