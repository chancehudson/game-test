use rand::RngCore;
use rand::rand_core::impls::fill_bytes_via_next;
use serde::Deserialize;
use serde::Serialize;

/// A cheap RNG based on Xorshift.
/// NOT a CSPRNG or even a good RNG
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    pub fn new(seed: u64) -> Self {
        assert_ne!(seed, 0);
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        const A1: u64 = 21;
        const A2: u64 = 35;
        const A3: u64 = 4;
        // Simple Xorshift RNG, constants taken from
        // https://numerical.recipes/book.html
        self.state = self.state ^ (self.state >> A1);
        self.state = self.state ^ (self.state << A2);
        self.state = self.state ^ (self.state >> A3);
        self.state
    }
}

impl RngCore for XorShiftRng {
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        fill_bytes_via_next(self, dst);
    }

    fn next_u64(&mut self) -> u64 {
        let low = self.next() as u64;
        let high = self.next() as u64;
        (high << 32) + low
    }

    fn next_u32(&mut self) -> u32 {
        self.next() as u32
    }
}
