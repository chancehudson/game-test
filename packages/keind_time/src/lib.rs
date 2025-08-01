use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use keind::prelude::*;

/// Operate a keind instance at a certain speed
/// through time. e.g. 60 steps per second
///
/// This logic is not necessary in zk, there we want
/// :inf: steps per second.
#[derive(Debug, Clone)]
pub struct GameEngineTime {
    pub start_timestamp: f64,
    pub steps_per_second: u64,
}

impl Default for GameEngineTime {
    fn default() -> Self {
        Self {
            start_timestamp: Self::now(),
            steps_per_second: 60,
        }
    }
}

impl GameEngineTime {
    pub fn from_step(step_index: u64, steps_per_second: u64) -> Self {
        let step_len = 1.0 / steps_per_second as f64;
        Self {
            start_timestamp: Self::now() - (step_index as f64) * step_len,
            steps_per_second,
        }
    }

    /// Get unix timestamp as a `f64`.
    pub fn now() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    #[inline]
    pub fn step_len(&self) -> f64 {
        1.0 / (self.steps_per_second as f64)
    }

    /// Calculate the step the engine should be at assuming
    /// it steps `self.steps_per_second` times per second.
    pub fn expected_step_index(&self) -> u64 {
        let uptime = Self::now() - self.start_timestamp;
        (uptime / self.step_len()).floor() as u64
    }

    /// Tick a game engine by stepping it forward to
    /// `uptime / steps_per_second`.
    pub fn tick<G: GameLogic>(&self, engine: &mut GameEngine<G>) -> Vec<RefPointer<G::Event>> {
        let to_step = self.expected_step_index();
        if &to_step <= engine.step_index() {
            println!("noop tick: your tick rate is too high!");
            vec![]
        } else {
            engine.step_to(&to_step)
        }
    }
}
