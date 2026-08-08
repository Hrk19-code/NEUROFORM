//! System 7 — Neuromodulatory System (DESIGN.md §4.7).
//!
//! Eight biologically-inspired functional axes: dopamine-like (da), serotonin-like
//! (5ht), norepinephrine-like (ne), acetylcholine-like (ach), endocannabinoid-like
//! (ecb), cortisol-like (cort), oxytocin-like (oxt), vasopressin-like (avp).
//! Each axis: level, baseline, reactivity, decay, noise — ODE-style integration.
//! Levels modulate probabilities/gains downstream; they never determine behavior.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

pub const N_AXES: usize = 8;
pub const AXE_IDS: [&str; N_AXES] = ["da", "5ht", "ne", "ach", "ecb", "cort", "oxt", "avp"];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AxisSnapshot {
    pub id: String,
    pub level: f32,
    pub baseline: f32,
    pub reactivity: f32,
    pub decay: f32,
    pub noise: f32,
}

#[derive(Clone, Debug)]
pub struct Axis {
    pub level: f32,
    pub baseline: f32,
    pub reactivity: f32,
    pub decay: f32,
    pub noise: f32,
}

#[derive(Clone, Debug)]
pub struct ModulatorSystem {
    pub axes: [Axis; N_AXES],
}

impl ModulatorSystem {
    /// Initialize with seeded priors (neutral-ish; embodiment priors arrive with
    /// the hormonal system in M1 — DESIGN.md §4.8).
    pub fn new(rng: &mut Rng) -> Self {
        let mut axes = Vec::with_capacity(N_AXES);
        for id in AXE_IDS {
            let baseline = rng.next_f32_range(0.30, 0.60);
            axes.push(Axis {
                level: baseline + (rng.next_f32() - 0.5) * 0.2,
                baseline,
                reactivity: rng.next_f32_range(0.02, 0.08),
                decay: rng.next_f32_range(0.02, 0.10),
                noise: rng.next_f32_range(0.004, 0.015),
            });
            let _ = id;
        }
        ModulatorSystem {
            axes: axes.try_into().expect("8 axes"),
        }
    }

    pub fn restore(snaps: &[AxisSnapshot]) -> Self {
        let mut axes = Vec::with_capacity(N_AXES);
        for s in snaps.iter().take(N_AXES) {
            axes.push(Axis {
                level: s.level,
                baseline: s.baseline,
                reactivity: s.reactivity,
                decay: s.decay,
                noise: s.noise,
            });
        }
        while axes.len() < N_AXES {
            axes.push(Axis {
                level: 0.5,
                baseline: 0.5,
                reactivity: 0.05,
                decay: 0.05,
                noise: 0.01,
            });
        }
        ModulatorSystem {
            axes: axes.try_into().expect("8 axes"),
        }
    }

    pub fn snapshot(&self) -> Vec<AxisSnapshot> {
        self.axes
            .iter()
            .enumerate()
            .map(|(i, a)| AxisSnapshot {
                id: AXE_IDS[i].to_string(),
                level: a.level,
                baseline: a.baseline,
                reactivity: a.reactivity,
                decay: a.decay,
                noise: a.noise,
            })
            .collect()
    }

    /// L += (b − L)·decay·dt + noise·U(−1,1); clamped to [0, 1].
    pub fn step(&mut self, dt: f32, rng: &mut Rng) {
        for a in self.axes.iter_mut() {
            let k = (a.decay * dt).min(1.0);
            a.level += (a.baseline - a.level) * k + a.noise * (rng.next_f32() - 0.5) * 2.0;
            a.level = a.level.clamp(0.0, 1.0);
        }
    }

    pub fn level(&self, i: usize) -> f32 {
        self.axes[i].level
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for a in self.axes.iter() {
            for b in a.level.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_stay_bounded_and_deterministic() {
        let mut r1 = Rng::new(3);
        let mut r2 = Rng::new(3);
        let mut a = ModulatorSystem::new(&mut r1);
        let mut b = ModulatorSystem::new(&mut r2);
        for _ in 0..100_000 {
            a.step(0.1, &mut r1);
            b.step(0.1, &mut r2);
        }
        assert_eq!(a.digest(), b.digest());
        for ax in a.axes.iter() {
            assert!((0.0..=1.0).contains(&ax.level));
        }
    }
}
