//! Intuitive physics learner (DESIGN.md §4.12, master prompt §11).
//!
//! A blank-slate prediction-error learner: it starts with ZERO knowledge —
//! every rate at maximum uncertainty — and learns from raw observation
//! frames only. No labels, no taught rules, no Newton. It watches entities
//! (position, motion, support, containment, contact) and accumulates
//! frequencies; its expectations ARE the learned rates; violations produce
//! surprise (prediction error) which feeds salience and curiosity.
//!
//! Qualitative, approximate, human-like on purpose: it learns *that*
//! unsupported things tend to fall, *that* contained things stay, *that*
//! contact changes motion — never the equations. Deterministic.

use serde::{Deserialize, Serialize};

/// One raw observation frame of one entity (unlabeled — features, not names).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PhysicsFrame {
    pub tick: u64,
    pub entity: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub moving: bool,
    pub supported: bool,   // has something beneath it
    pub contained: bool,   // inside a container
    pub contact: bool,     // touching another entity
}

/// Learned rate with confidence (beta-count style: confidence rises with
/// observations; rate = observed frequency). Starts neutral.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LearnedRate {
    pub rate: f32,      // 0..1 observed frequency
    pub observations: u32,
}

impl LearnedRate {
    fn new() -> Self {
        LearnedRate { rate: 0.5, observations: 0 } // maximum uncertainty
    }
    fn update(&mut self, outcome: bool) {
        self.observations += 1;
        let n = self.observations as f32;
        self.rate = (self.rate * (n - 1.0) + outcome as u32 as f32) / n;
    }
    pub fn confidence(&self) -> f32 {
        (self.observations as f32 / (self.observations as f32 + 4.0)).clamp(0.0, 1.0)
    }
}

/// The learner's implicit world model — expectations learned from exposure.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhysicsModel {
    /// P(unsupported → moves down next frame): the learned "gravity".
    pub fall_when_unsupported: LearnedRate,
    /// P(supported → stays still).
    pub stay_when_supported: LearnedRate,
    /// P(contained → position unchanged).
    pub stay_when_contained: LearnedRate,
    /// P(moving → keeps moving): learned inertia.
    pub continue_moving: LearnedRate,
    /// P(contact → velocity changes): learned collision.
    pub change_on_contact: LearnedRate,
    /// P(contained → still contained next frame): object permanence-ish.
    pub permanence_contained: LearnedRate,
    /// Mean |velocity| while "still" (learned stillness threshold).
    pub still_speed: f32,
}

impl PhysicsModel {
    fn new() -> Self {
        PhysicsModel {
            fall_when_unsupported: LearnedRate::new(),
            stay_when_supported: LearnedRate::new(),
            stay_when_contained: LearnedRate::new(),
            continue_moving: LearnedRate::new(),
            change_on_contact: LearnedRate::new(),
            permanence_contained: LearnedRate::new(),
            still_speed: 0.01,
        }
    }
}

/// A single learned prediction vs. what actually happened.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PredictionError {
    pub tick: u64,
    pub entity: u32,
    pub surprise: f32,     // 0..1
    pub rule: String,      // which expectation was violated
    pub expected: f32,
    pub actual: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhysicsLearner {
    pub model: PhysicsModel,
    /// Last frame per entity (what we predicted from).
    pub last: std::collections::HashMap<u32, PhysicsFrame>,
    /// Recent prediction errors (bounded).
    pub errors: Vec<PredictionError>,
    /// Running surprise (decays); feeds curiosity/salience.
    pub surprise: f32,
    /// Total observations seen.
    pub observations: u64,
    /// Determinism seed material.
    pub history: Vec<String>,
}

impl PhysicsLearner {
    pub fn new() -> Self {
        PhysicsLearner {
            model: PhysicsModel::new(),
            last: std::collections::HashMap::new(),
            errors: Vec::new(),
            surprise: 0.0,
            observations: 0,
            history: Vec::new(),
        }
    }

    fn speed(f: &PhysicsFrame) -> f32 {
        (f.vx * f.vx + f.vy * f.vy).sqrt()
    }

    /// Predict whether the entity will be moving next frame.
    fn predict(&self, f: &PhysicsFrame) -> f32 {
        let m = &self.model;
        if f.supported {
            // supported things stay still (usually)
            if m.stay_when_supported.rate > 0.5 { 0.0 } else { f.moving as u32 as f32 }
        } else if f.contained {
            // contained things stay
            0.0
        } else if f.moving {
            // unsupported + moving: keep moving (inertia) or fall
            (m.continue_moving.rate > 0.5) as u32 as f32
        } else {
            // unsupported + still: the learned-gravity rule — may start falling
            (m.fall_when_unsupported.rate > 0.5) as u32 as f32
        }
    }

    /// Observe a frame: predict from the previous frame of the same entity,
    /// score surprise, update the model. Returns surprise (0..1).
    pub fn observe(&mut self, f: &PhysicsFrame) -> f32 {
        self.observations += 1;
        let m = &mut self.model;
        let mut surprise = 0.0f32;
        let mut rule = String::new();

        let mut expected = 0.0f32;
        if let Some(prev) = self.last.get(&f.entity) {
            let speed = Self::speed(f);
            // 1. gravity rule: unsupported→moving-down (fall) vs staying
            if !prev.supported && !prev.contained && !prev.moving && f.moving {
                m.fall_when_unsupported.update(true);
                rule = "fall".into();
            } else if !prev.supported && !prev.contained && !prev.moving && !f.moving && speed < m.still_speed {
                m.fall_when_unsupported.update(false);
                rule = "hover".into();
            }
            // 2. support rule: supported → stays still
            if prev.supported {
                let stayed = !f.moving && speed < m.still_speed;
                m.stay_when_supported.update(stayed);
                if !stayed {
                    rule = "support-violation".into();
                }
            }
            // 3. containment rule: contained → position unchanged
            if prev.contained {
                let stayed = !f.moving && speed < m.still_speed;
                m.stay_when_contained.update(stayed);
                m.permanence_contained.update(f.contained);
                if !stayed {
                    rule = "containment-violation".into();
                } else if !f.contained {
                    rule = "disappeared".into(); // object permanence violation
                }
            }
            // 4. inertia: moving → keeps moving
            if prev.moving && !prev.contained {
                let kept = f.moving;
                m.continue_moving.update(kept);
                if !kept {
                    rule = "stopped".into();
                }
            }
            // 5. collision: contact → velocity changes
            if prev.contact {
                let changed = (Self::speed(prev) - speed).abs() > m.still_speed.max(0.02);
                m.change_on_contact.update(changed);
                if changed {
                    rule = "collision".into();
                }
            }
            // stillness threshold learning
            if !f.moving {
                m.still_speed = m.still_speed * 0.99 + speed * 0.01;
            }

            // Model-based surprise: does what happened contradict the model?
            // Qualitative only — the learner predicts *whether* things move,
            // not how fast (magnitude learning is beyond its scope).
            let pred_moving = self.predict(prev);
            let moving_surprise = if pred_moving != f.moving as u32 as f32 { 0.7f32 } else { 0.0f32 };
            surprise = moving_surprise.clamp(0.0, 1.0);
            if surprise > 0.55 && rule.is_empty() {
                rule = "model".into();
            }
            expected = pred_moving;
        }

        self.last.insert(f.entity, f.clone());
        self.surprise = (self.surprise * 0.9 + surprise * 0.1).clamp(0.0, 1.0);
        if surprise > 0.3 {
            let actual = if f.moving { 1.0 } else { 0.0 };
            self.errors.push(PredictionError {
                tick: f.tick,
                entity: f.entity,
                surprise,
                rule: rule.clone(),
                expected,
                actual,
            });
            if self.errors.len() > 64 {
                self.errors.remove(0);
            }
        }
        if surprise > 0.55 {
            self.history.push(format!(
                "[t={}] entity {}: {} surprise {:.2} (expected {}, saw {})",
                f.tick, f.entity, rule, surprise, expected, if f.moving { 1.0 } else { 0.0 }
            ));
            if self.history.len() > 40 {
                self.history.remove(0);
            }
        }
        surprise
    }

    /// Idle: surprise decays; expectations stay (the model only changes by
    /// observation — nature is learned, never decayed away wholesale).
    pub fn step_idle(&mut self, _dt: f32) {
        self.surprise *= 0.995;
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0x5050_0000_0000_0001;
        let mut mix = |v: u64| h = (h ^ v).wrapping_mul(0x0000_0100_0000_01b3);
        mix(self.model.fall_when_unsupported.rate.to_bits() as u64);
        mix(self.model.stay_when_supported.rate.to_bits() as u64);
        mix(self.model.stay_when_contained.rate.to_bits() as u64);
        mix(self.model.continue_moving.rate.to_bits() as u64);
        mix(self.model.change_on_contact.rate.to_bits() as u64);
        mix(self.model.permanence_contained.rate.to_bits() as u64);
        mix(self.surprise.to_bits() as u64);
        mix(self.observations);
        h
    }
}

impl Default for PhysicsLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(tick: u64, entity: u32, x: f32, y: f32, moving: bool, supported: bool, contained: bool, contact: bool) -> PhysicsFrame {
        PhysicsFrame {
            tick, entity, x, y,
            vx: if moving { 0.5 } else { 0.0 },
            vy: if moving { 0.4 } else { 0.0 },
            moving, supported, contained, contact,
        }
    }

    #[test]
    fn blank_slate_starts_maximally_uncertain() {
        let p = PhysicsLearner::new();
        assert_eq!(p.model.fall_when_unsupported.rate, 0.5);
        assert_eq!(p.model.fall_when_unsupported.observations, 0);
        assert_eq!(p.model.fall_when_unsupported.confidence(), 0.0);
        assert_eq!(p.observations, 0);
    }

    #[test]
    fn learns_gravity_from_unsupported_falls() {
        let mut p = PhysicsLearner::new();
        // A ball is still & unsupported, then falls — repeated 20x.
        for i in 0..20 {
            let t = i * 2;
            p.observe(&frame(t, 1, 10.0, 20.0, false, false, false, false));
            p.observe(&frame(t + 1, 1, 10.0, 24.0, true, false, false, false));
        }
        assert!(p.model.fall_when_unsupported.rate > 0.9, "rate {}", p.model.fall_when_unsupported.rate);
        assert!(p.model.fall_when_unsupported.confidence() > 0.8);
    }

    #[test]
    fn learns_support_and_containment() {
        let mut p = PhysicsLearner::new();
        // Supported box stays put, 20x.
        for i in 0..20 {
            let t = i * 2;
            p.observe(&frame(t, 2, 5.0, 5.0, false, true, false, false));
            p.observe(&frame(t + 1, 2, 5.0, 5.0, false, true, false, false));
        }
        assert!(p.model.stay_when_supported.rate > 0.9);
        // Contained object stays, 20x.
        for i in 0..20 {
            let t = 100 + i * 2;
            p.observe(&frame(t, 3, 30.0, 30.0, false, false, true, false));
            p.observe(&frame(t + 1, 3, 30.0, 30.0, false, false, true, false));
        }
        assert!(p.model.stay_when_contained.rate > 0.9);
        assert!(p.model.permanence_contained.rate > 0.9);
    }

    #[test]
    fn containment_violation_surprises() {
        let mut p = PhysicsLearner::new();
        // Learned: contained things stay.
        for i in 0..6 {
            let t = i * 2;
            p.observe(&frame(t, 4, 7.0, 7.0, false, false, true, false));
            p.observe(&frame(t + 1, 4, 7.0, 7.0, false, false, true, false));
        }
        p.surprise = 0.0;
        // Now the contained thing is gone (moved wildly / disappeared).
        let s = p.observe(&frame(999, 4, 7.0, 7.0, true, false, false, false));
        assert!(s > 0.3, "surprise {s}");
        assert!(p.errors.iter().any(|e| e.rule == "disappeared" || e.rule == "model" || e.rule == "containment-violation"));
    }

    #[test]
    fn familiar_scenarios_surprise_less_after_learning() {
        let mut p = PhysicsLearner::new();
        // Train on falls.
        for i in 0..20 {
            let t = i * 2;
            p.observe(&frame(t, 5, 1.0, 1.0, false, false, false, false));
            p.observe(&frame(t + 1, 5, 1.0, 3.0, true, false, false, false));
        }
        p.surprise = 0.0;
        // A familiar fall now.
        let s = p.observe(&frame(500, 6, 2.0, 2.0, false, false, false, false));
        let s2 = p.observe(&frame(501, 6, 2.0, 4.0, true, false, false, false));
        assert!(s2 < 0.3, "surprise after learning {s2} (first frame {s})");
    }

    #[test]
    fn deterministic_digest() {
        let mut a = PhysicsLearner::new();
        let mut b = PhysicsLearner::new();
        for i in 0..10 {
            let t = i * 2;
            a.observe(&frame(t, 1, 1.0, 1.0, false, false, false, false));
            a.observe(&frame(t + 1, 1, 1.0, 3.0, true, false, false, false));
            b.observe(&frame(t, 1, 1.0, 1.0, false, false, false, false));
            b.observe(&frame(t + 1, 1, 1.0, 3.0, true, false, false, false));
        }
        assert_eq!(a.digest(), b.digest());
    }
}
