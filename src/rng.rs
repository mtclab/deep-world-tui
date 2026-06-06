use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Seedable RNG wrapper. All generation flows from the seed via sub-seed
/// derivation. No thread_rng/time/entropy in gen/ or sim/.
pub struct SeedRng {
    rng: ChaCha8Rng,
}

impl SeedRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Derive a deterministic sub-RNG from the current state.
    /// Useful for isolating subsystems (world gen, NPC gen, sim).
    pub fn fork(&mut self) -> Self {
        let sub_seed = rand::Rng::random::<u64>(&mut self.rng);
        Self::new(sub_seed)
    }

    /// Generate a u64 from the RNG.
    pub fn next_u64(&mut self) -> u64 {
        rand::Rng::random::<u64>(&mut self.rng)
    }

    /// Generate a uniform u32 in [0, n).
    pub fn gen_range(&mut self, n: u32) -> u32 {
        rand::Rng::random_range(&mut self.rng, 0..n)
    }

    /// Generate a uniform f64 in [0, 1).
    pub fn gen_f64(&mut self) -> f64 {
        rand::Rng::random::<f64>(&mut self.rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_same_seed() {
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn determinism_different_seed() {
        let mut a = SeedRng::new(42);
        let mut b = SeedRng::new(99);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn fork_determinism() {
        let mut rng = SeedRng::new(42);
        let mut sub1 = rng.fork();
        let v1 = sub1.next_u64();
        let mut rng2 = SeedRng::new(42);
        let mut sub2 = rng2.fork();
        let v2 = sub2.next_u64();
        assert_eq!(v1, v2);
    }

    #[test]
    fn gen_range_and_f64() {
        let mut rng = SeedRng::new(123);
        let r = rng.gen_range(10);
        assert!(r < 10);
        let f = rng.gen_f64();
        assert!((0.0..1.0).contains(&f));
    }
}
