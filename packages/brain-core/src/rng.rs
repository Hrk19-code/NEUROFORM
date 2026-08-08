//! Deterministic seeded RNG (SplitMix64).
//!
//! Every stochastic draw in the core comes from this generator so that a Brain File
//! replays bit-identically for a given seed (DESIGN.md §3.3, §22.10).

pub struct Rng {
    s: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            s: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1) with 24 bits of mantissa precision (deterministic).
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 * (1.0 / (1u64 << 24) as f32)
    }

    #[inline]
    pub fn next_f32_range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    #[inline]
    pub fn next_u64_below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// Internal stream state — persisted so a loaded file resumes the exact
    /// noise sequence (deterministic continuity, DESIGN.md §3.3).
    pub fn state(&self) -> u64 {
        self.s
    }

    pub fn from_state(s: u64) -> Self {
        Rng { s }
    }

    /// Fill a buffer with raw deterministic bytes.
    pub fn fill_bytes(&mut self, out: &mut [u8]) {
        let mut i = 0;
        while i < out.len() {
            let v = self.next_u64().to_le_bytes();
            let n = (out.len() - i).min(8);
            out[i..i + n].copy_from_slice(&v[..n]);
            i += n;
        }
    }

    /// Random v4 UUID formatted from this generator (no OS entropy needed).
    pub fn next_uuid4(&mut self) -> String {
        let mut b = [0u8; 16];
        self.fill_bytes(&mut b);
        b[6] = (b[6] & 0x0F) | 0x40; // version 4
        b[8] = (b[8] & 0x3F) | 0x80; // variant 10
        let h = hex::encode(b);
        format!(
            "{}-{}-{}-{}-{}",
            &h[0..8],
            &h[8..12],
            &h[12..16],
            &h[16..20],
            &h[20..32]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_bounded() {
        let mut a = Rng::new(123);
        let mut b = Rng::new(123);
        for _ in 0..10_000 {
            let x = a.next_f32();
            let y = b.next_f32();
            assert_eq!(x, y);
            assert!((0.0..1.0).contains(&x));
        }
        let mut c = Rng::new(456);
        let mut differs = false;
        for _ in 0..100 {
            if a.next_f32() != c.next_f32() {
                differs = true;
            }
        }
        assert!(differs);
    }
}
