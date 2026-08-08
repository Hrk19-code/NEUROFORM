//! System 1 — Global Latent State (DESIGN.md §4.1).
//!
//! A persistent whole-brain state vector `g ∈ R^dim` with named sub-blocks:
//! affect (8), vigilance (4), stress (3), social (4), development (4), embodied (3),
//! plus a reserved block for future modalities. Integration is a damped
//! baseline-attracting integrator with seeded noise — fully deterministic.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

// --- Named sub-block layout -------------------------------------------------

pub const D_AFFECT: usize = 8;
pub const D_VIGILANCE: usize = 4;
pub const D_STRESS: usize = 3;
pub const D_SOCIAL: usize = 4;
pub const D_DEVELOPMENT: usize = 4;
pub const D_EMBODIED: usize = 3;
pub const N_NAMED: usize = D_AFFECT + D_VIGILANCE + D_STRESS + D_SOCIAL + D_DEVELOPMENT + D_EMBODIED;

pub mod affect {
    pub const VALENCE: usize = 0;
    pub const AROUSAL: usize = 1;
    pub const DOMINANCE: usize = 2;
    pub const WARMTH: usize = 3;
    pub const IRRITABILITY: usize = 4;
    pub const CALM: usize = 5;
    pub const LONELINESS: usize = 6;
    pub const SAFETY: usize = 7;
}

pub mod vigilance {
    pub const ENERGY: usize = 0;
    pub const ATTENTION_FOCUS: usize = 1;
    pub const ALERTNESS: usize = 2;
    pub const FATIGUE: usize = 3;
}

pub mod stress {
    pub const LOAD: usize = 0;
    pub const REGULATION_CAPACITY: usize = 1;
    pub const SENSORY_SATURATION: usize = 2;
}

pub mod social {
    pub const OPENNESS: usize = 0;
    pub const AFFILIATIVE_DRIVE: usize = 1;
    pub const BOUNDARY_TIGHTNESS: usize = 2;
    pub const PEER_PRESENCE: usize = 3;
}

pub mod development {
    pub const POSTURE: usize = 0;
    pub const CURIOSITY: usize = 1;
    pub const PLASTICITY_WINDOW: usize = 2;
    pub const CREATIVE_READINESS: usize = 3;
}

pub mod embodied {
    pub const BODY_COMFORT: usize = 0;
    pub const MOTION_COMFORT: usize = 1;
    pub const INTEROCEPTIVE_LOAD: usize = 2;
}

/// Baseline (attractor) for each named channel.
const BASELINES: [f32; N_NAMED] = [
    // affect
    0.05, 0.35, 0.40, 0.30, 0.15, 0.50, 0.20, 0.70,
    // vigilance
    0.60, 0.50, 0.45, 0.20,
    // stress
    0.20, 0.60, 0.25,
    // social
    0.40, 0.35, 0.40, 0.00,
    // development
    0.10, 0.60, 0.80, 0.40,
    // embodied
    0.70, 0.60, 0.20,
];

/// Time constants in seconds (affect fast-ish, development slow).
const TAUS: [f32; N_NAMED] = [
    1200.0, 900.0, 1800.0, 1800.0, 1500.0, 1200.0, 2400.0, 1500.0, // affect
    7200.0, 300.0, 600.0, 7200.0,                                  // vigilance
    3600.0, 3600.0, 1800.0,                                        // stress
    3600.0, 3600.0, 7200.0, 600.0,                                 // social
    2_592_000.0, 3600.0, 2_592_000.0, 1800.0,                      // development
    3600.0, 1800.0, 3600.0,                                        // embodied
];

/// Per-tick noise amplitude (additive, seeded, deterministic).
const NOISE: [f32; N_NAMED] = [
    0.004, 0.004, 0.002, 0.003, 0.002, 0.003, 0.002, 0.003, // affect
    0.003, 0.002, 0.002, 0.002,                             // vigilance
    0.002, 0.002, 0.002,                                    // stress
    0.003, 0.002, 0.002, 0.001,                             // social
    0.0005, 0.001, 0.0005, 0.002,                           // development
    0.002, 0.002, 0.002,                                    // embodied
];

/// Serializable snapshot (what persists in the STATE shard).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StateSnapshot {
    pub schema_version: u32,
    pub sim_time: u64,
    pub dim: usize,
    pub named: Vec<f32>,
    pub reserved: Vec<f32>,
}

pub struct GlobalState {
    pub sim_time: u64,
    pub dim: usize,
    pub named: [f32; N_NAMED],
    pub reserved: Vec<f32>,
}

impl GlobalState {
    pub fn new(dim: usize, rng: &mut Rng) -> Self {
        let mut named = BASELINES;
        for v in named.iter_mut() {
            // Small seeded perturbation around baseline (the file is not born at
            // exactly the population mean).
            *v = (*v + (rng.next_f32() - 0.5) * 0.2).clamp(0.0, 1.0);
        }
        let reserved = (0..dim.saturating_sub(N_NAMED))
            .map(|_| (rng.next_f32() - 0.5) * 0.1)
            .collect();
        GlobalState {
            sim_time: 0,
            dim,
            named,
            reserved,
        }
    }

    /// Restore from a persisted snapshot (baselines/taus/noise come from the
    /// constant tables — they are not stored in the file).
    pub fn restore(snap: &StateSnapshot) -> Self {
        let mut named = [0.0f32; N_NAMED];
        for (i, v) in snap.named.iter().enumerate().take(N_NAMED) {
            named[i] = *v;
        }
        GlobalState {
            sim_time: snap.sim_time,
            dim: snap.dim,
            named,
            reserved: snap.reserved.clone(),
        }
    }

    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            schema_version: 1,
            sim_time: self.sim_time,
            dim: self.dim,
            named: self.named.to_vec(),
            reserved: self.reserved.clone(),
        }
    }

    /// Advance one simulation step. `dt` is in seconds (0.1 at 10 Hz).
    pub fn step(&mut self, dt: f32, rng: &mut Rng) {
        for i in 0..N_NAMED {
            let k = (dt / TAUS[i]).min(1.0);
            let noise = NOISE[i] * (rng.next_f32() - 0.5) * 2.0;
            self.named[i] += (BASELINES[i] - self.named[i]) * k + noise;
        }
        // valence lives in [-1, 1]; everything else in [0, 1]
        self.named[affect::VALENCE] = self.named[affect::VALENCE].clamp(-1.0, 1.0);
        for v in self.named.iter_mut().skip(1) {
            *v = v.clamp(0.0, 1.0);
        }
        for r in self.reserved.iter_mut() {
            *r += (0.0 - *r) * (dt / 3600.0) + 0.001 * (rng.next_f32() - 0.5) * 2.0;
            *r = r.clamp(-0.5, 0.5);
        }
    }

    /// The whole-brain vector g (named + reserved concatenated).
    pub fn g(&self) -> Vec<f32> {
        self.named.iter().chain(self.reserved.iter()).cloned().collect()
    }

    /// FNV-1a 64 digest over the full state — the determinism check.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.sim_time.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for x in self.named.iter().chain(self.reserved.iter()) {
            for b in x.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }

    pub fn affect(&self) -> &[f32] {
        &self.named[0..D_AFFECT]
    }
    pub fn vigilance(&self) -> &[f32] {
        &self.named[D_AFFECT..D_AFFECT + D_VIGILANCE]
    }
    pub fn stress(&self) -> &[f32] {
        &self.named[D_AFFECT + D_VIGILANCE..D_AFFECT + D_VIGILANCE + D_STRESS]
    }
    pub fn social(&self) -> &[f32] {
        &self.named[D_AFFECT + D_VIGILANCE + D_STRESS
            ..D_AFFECT + D_VIGILANCE + D_STRESS + D_SOCIAL]
    }
    pub fn development(&self) -> &[f32] {
        &self.named[N_NAMED - D_EMBODIED - D_DEVELOPMENT..N_NAMED - D_EMBODIED]
    }
    pub fn embodied(&self) -> &[f32] {
        &self.named[N_NAMED - D_EMBODIED..N_NAMED]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_bounds_and_determinism() {
        let mut r1 = Rng::new(9);
        let mut r2 = Rng::new(9);
        let mut a = GlobalState::new(256, &mut r1);
        let mut b = GlobalState::new(256, &mut r2);
        for _ in 0..100_000 {
            a.step(0.1, &mut r1);
            b.step(0.1, &mut r2);
        }
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.g().len(), 256);
        assert!((-1.0..=1.0).contains(&a.named[affect::VALENCE]));
        for v in a.named.iter().skip(1) {
            assert!((0.0..=1.0).contains(v), "out of range: {v}");
        }
    }

    #[test]
    fn snapshot_roundtrip_exact() {
        let mut r1 = Rng::new(5);
        let mut a = GlobalState::new(192, &mut r1);
        for _ in 0..50_000 {
            a.step(0.1, &mut r1);
        }
        let snap = a.snapshot();
        let b = GlobalState::restore(&snap);
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.sim_time, b.sim_time);
    }
}
