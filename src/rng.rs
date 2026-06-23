/// Seedable xorshift32 PRNG.
///
/// Deterministic per seed; no global state. The effects layer (STR-002) threads
/// a caller-owned `&mut Rng` into every spawn call so burst sequences are
/// reproducible given the same seed.
pub struct Rng {
    state: u32,
}

impl Rng {
    /// Construct a new `Rng` from a 64-bit seed.
    ///
    /// The seed is folded to 32 bits using splitmix64 to avoid the all-zero
    /// state restriction of xorshift32 when a naive cast would produce 0.
    pub fn new(seed: u64) -> Self {
        // splitmix64 fold: guarantees a non-zero state even for seed == 0.
        let state = splitmix64(seed) as u32;
        // xorshift32 must not start at 0; fall back to a fixed odd constant.
        let state = if state == 0 { 0x9e37_79b9 } else { state };
        Self { state }
    }

    /// Advance the state and return the next pseudo-random `u32`.
    pub fn next_u32(&mut self) -> u32 {
        // Standard xorshift32 triple.
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    /// Return the next pseudo-random `f32` in `[0.0, 1.0)`.
    ///
    /// Derived from the high 24 bits of `next_u32` to avoid modulo bias.
    pub fn next_f32(&mut self) -> f32 {
        let bits = self.next_u32();
        (bits >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Return a pseudo-random `f32` uniformly distributed in `[min, max)`.
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

/// One step of splitmix64, used to fold a u64 seed to 32 bits.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests;
