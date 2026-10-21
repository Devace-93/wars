//! Seeded RNG and the world hash used for desync detection.

use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use serde::{Deserialize, Serialize};

/// A serialisable wrapper so the RNG state is part of the snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng {
    seed: u64,
    counter: u64,
    #[serde(skip)]
    inner: Option<Xoshiro256PlusPlus>,
}

impl PartialEq for Rng {
    fn eq(&self, o: &Rng) -> bool {
        self.seed == o.seed && self.counter == o.counter
    }
}
impl Eq for Rng {}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng {
            seed,
            counter: 0,
            inner: Some(Xoshiro256PlusPlus::seed_from_u64(seed)),
        }
    }
    fn ensure(&mut self) {
        if self.inner.is_none() {
            // Rebuild after deserialisation by replaying the draw count.
            let mut r = Xoshiro256PlusPlus::seed_from_u64(self.seed);
            for _ in 0..self.counter {
                r.next_u64();
            }
            self.inner = Some(r);
        }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.ensure();
        self.counter += 1;
        self.inner.as_mut().unwrap().next_u64()
    }
    /// Uniform integer in `0..n` (n > 0).
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as u32
    }
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + self.below((hi - lo) as u32) as i32
    }
    /// True with probability `pct` percent.
    pub fn chance(&mut self, pct: i32) -> bool {
        (self.below(100) as i32) < pct
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
}

/// FNV-1a over bytes: small, fast and identical on every platform.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
