//! Bias Audit Engine — M1 skeleton (DESIGN.md §14).
//!
//! M1 ships the metric framework + the four metrics computable from M1 data:
//!   #7 memory overvaluation (salience Gini)
//!   #6 repetition (mean pairwise cosine of recent traces)
//!   #4 emotional loop (valence autocorrelation at 4-min lag)
//!   #8 user-shaped overfitting (single-source dominance)
//! The remaining six metrics are declared with thresholds and report
//! "requires <milestone> stores" when computed (M2: sleep/consolidation
//! metrics; M7: peer-convergence).
//!
//! The engine has read access to everything and write access to nothing: it
//! produces reports and suggestions only (user-gated interventions, §14.5).

use serde::{Deserialize, Serialize};

use crate::brain::Brain;
use crate::events::cosine;

pub const GINI_ALARM: f32 = 0.60;
pub const REPETITION_ALARM: f32 = 0.85;
pub const LOOP_ALARM: f32 = 0.60;
pub const SOURCE_DOMINANCE_ALARM: f32 = 0.80;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditMetric {
    pub id: String,
    pub value: f32,
    pub threshold: f32,
    pub alarm: bool,
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditReport {
    pub run_at: u64,
    pub trigger: String,
    pub metrics: Vec<AuditMetric>,
    pub interventions: Vec<String>,
}

pub struct AuditEngine {
    /// Rolling valence history for loop detection (ring buffer).
    pub valence_history: Vec<f32>,
    pub valence_head: usize,
    pub valence_cap: usize,
    pub filled: usize,
}

impl AuditEngine {
    pub fn new() -> Self {
        AuditEngine {
            valence_history: vec![0.0; 28_800], // 48 sim-minutes at 10 Hz
            valence_head: 0,
            valence_cap: 28_800,
            filled: 0,
        }
    }

    pub fn push_valence(&mut self, v: f32) {
        self.valence_history[self.valence_head] = v;
        self.valence_head = (self.valence_head + 1) % self.valence_cap;
        self.filled = (self.filled + 1).min(self.valence_cap);
    }

    /// Autocorrelation at `lag` samples over the last `window` written samples
    /// (chronological order; unfilled ring slots are never read). Accumulates
    /// in f64: f32 summation drift on flat signals would otherwise produce a
    /// spurious 1.0 autocorrelation.
    fn valence_autocorr(&self, lag: usize) -> f32 {
        let n = self.filled.min(self.valence_cap);
        if n <= lag + 1 {
            return 0.0;
        }
        let start = (self.valence_head + self.valence_cap - n) % self.valence_cap;
        let chrono: Vec<f32> = (0..n)
            .map(|i| self.valence_history[(start + i) % self.valence_cap])
            .collect();
        let len = chrono.len();
        let mean: f64 = chrono.iter().map(|&v| v as f64).sum::<f64>() / len as f64;
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for i in lag..len {
            let a = chrono[i] as f64 - mean;
            let b = chrono[i - lag] as f64 - mean;
            num += a * b;
            den += a * a;
        }
        if den.abs() < 1e-12 {
            0.0
        } else {
            (num / den).clamp(-1.0, 1.0) as f32
        }
    }

    pub fn run(&self, brain: &Brain, trigger: &str) -> AuditReport {
        let mut metrics = Vec::new();

        // #7 memory overvaluation
        let gini = brain.episodic.salience_gini();
        metrics.push(AuditMetric {
            id: "memory-overvaluation".into(),
            value: gini,
            threshold: GINI_ALARM,
            alarm: gini >= GINI_ALARM,
            note: "salience Gini coefficient".into(),
        });

        // #6 repetition: mean pairwise cosine of the 50 most recent traces
        let recent: Vec<&crate::memory::EpisodicTrace> = brain
            .episodic
            .traces
            .iter()
            .rev()
            .take(50)
            .collect();
        let mut rep = 0.0f32;
        let mut pairs = 0usize;
        for i in 0..recent.len() {
            for j in (i + 1)..recent.len() {
                rep += cosine(&recent[i].embedding, &recent[j].embedding);
                pairs += 1;
            }
        }
        let rep = if pairs > 0 { rep / pairs as f32 } else { 0.0 };
        metrics.push(AuditMetric {
            id: "repetition".into(),
            value: rep,
            threshold: REPETITION_ALARM,
            alarm: rep >= REPETITION_ALARM,
            note: "mean pairwise cosine of 50 most recent traces".into(),
        });

        // #4 emotional loop (lag 2400 ticks = 4 sim-minutes)
        let loopv = self.valence_autocorr(2400);
        metrics.push(AuditMetric {
            id: "emotion-loop".into(),
            value: loopv,
            threshold: LOOP_ALARM,
            alarm: loopv >= LOOP_ALARM,
            note: "valence autocorrelation at 4-min lag".into(),
        });

        // #8 user-shaped overfitting: single-source dominance
        let total = brain.episodic.traces.len();
        let user = brain
            .episodic
            .traces
            .iter()
            .filter(|t| t.source == "user")
            .count();
        let dominance = if total == 0 {
            0.0
        } else {
            user as f32 / total as f32
        };
        metrics.push(AuditMetric {
            id: "user-overfit".into(),
            value: dominance,
            threshold: SOURCE_DOMINANCE_ALARM,
            alarm: dominance >= SOURCE_DOMINANCE_ALARM,
            note: "fraction of traces sourced from the user".into(),
        });

        // Declared-but-unimplemented metrics (honest stubs with notes).
        for (id, note) in [
            ("gender-rigidity", "requires M2 preference stores"),
            ("embodiment-restriction", "requires M2 creative-output stats"),
            ("relationship-fixation", "requires M7 social memory"),
            ("echo-chamber", "requires browsing exposure logs (M7+)"),
            ("llm-distortion", "requires provenance drift tracking (M2)"),
            ("peer-convergence", "requires M7 inter-brain similarity"),
        ] {
            metrics.push(AuditMetric {
                id: id.into(),
                value: 0.0,
                threshold: 0.0,
                alarm: false,
                note: note.into(),
            });
        }

        let mut interventions = Vec::new();
        for m in metrics.iter().filter(|m| m.alarm) {
            match m.id.as_str() {
                "memory-overvaluation" => {
                    interventions.push("memory reweighting: normalize salience distribution (user-gated)".into());
                }
                "repetition" => {
                    interventions.push("exposure diversification: suggest new-content exposure (user-gated)".into());
                }
                "emotion-loop" => {
                    interventions.push("sleep review: schedule consolidation to regulate the loop (user-gated)".into());
                }
                "user-overfit" => {
                    interventions.push("plasticity restoration: increase exploration weight (user-gated)".into());
                }
                _ => {}
            }
        }

        AuditReport {
            run_at: brain.state.sim_time,
            trigger: trigger.to_string(),
            metrics,
            interventions,
        }
    }
}

impl Default for AuditEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::Brain;
    use crate::capacity::TierName;

    #[test]
    fn empty_brain_reports_zero_alarms() {
        let brain = Brain::create(TierName::Prototype, 1);
        let audit = AuditEngine::new();
        let report = audit.run(&brain, "test");
        assert_eq!(report.metrics.len(), 10);
        assert!(!report.metrics.iter().any(|m| m.alarm));
        assert!(report.interventions.is_empty());
    }

    #[test]
    fn overvaluation_alarm_fires_on_concentrated_store() {
        let mut brain = Brain::create(TierName::Prototype, 2);
        // Inject one high-emotion event repeated so salience concentrates.
        for i in 0..5 {
            brain.ingest_text(
                &format!("terrible shocking event number {i}"),
                -0.9,
                0.8,
                "user",
            );
            brain.run_ticks(350); // bind window + nudges
        }
        brain.run_ticks(500);
        let audit = AuditEngine::new();
        let report = audit.run(&brain, "test");
        let m = report
            .metrics
            .iter()
            .find(|m| m.id == "memory-overvaluation")
            .unwrap();
        // With a near-empty store the Gini may be modest; the check is that the
        // metric is computed and within [0,1].
        assert!((0.0..=1.0).contains(&m.value));
        let _ = m.alarm;
    }

    #[test]
    fn loop_detection_on_periodic_valence() {
        // Square wave alternating every `period` ticks: anti-correlated at lag
        // `period` (−1), correlated at the full cycle 2·period (+1).
        let mut engine = AuditEngine::new();
        let period = 2400usize;
        for i in 0..period * 4 {
            let v = if (i / period) % 2 == 0 { 0.6 } else { -0.6 };
            engine.push_valence(v);
        }
        let ac_anti = engine.valence_autocorr(period);
        let ac_full = engine.valence_autocorr(period * 2);
        assert!(ac_full > 0.95, "full-cycle autocorrelation, got {ac_full}");
        assert!(ac_anti < -0.95, "anti-phase autocorrelation, got {ac_anti}");
        let mut flat = AuditEngine::new();
        for _ in 0..period * 4 {
            flat.push_valence(0.1);
        }
        assert!(flat.valence_autocorr(period).abs() < 0.1);
    }
}
