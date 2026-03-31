use rand::{RngExt, SeedableRng, seq::SliceRandom};
use rand_chacha::{ChaCha20Rng, ChaChaRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PRNG {
    inner: ChaChaRng,
}

impl PRNG {
    pub fn new(seed: u64) -> PRNG {
        PRNG { inner: ChaCha20Rng::seed_from_u64(seed) }
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.inner);
    }

    pub fn random_range(&mut self, low: i32, high: i32) -> i32 {
        self.inner.random_range(low..high)
    }
}
