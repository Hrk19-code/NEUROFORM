//! System 8 — Hormonal Embodiment System (DESIGN.md §4.8).
//!
//! Embodiment presets (male, female, custom, mixed, non-binary, user-defined)
//! are **probabilistic endocrine priors**: each of the 16 modulation axes gets
//! a (mean, spread) and the file samples its own values at creation. Priors are
//! biologically-informed digital analogues — "hard options similar to real
//! life" in the sense that the *distributions* differ — but they never
//! determine behavior:
//!
//!   • effects are bounded gains (|gain| ≤ GAIN_CAP = 0.3) on probabilities,
//!     learning rates, salience weights, and developmental tendencies only;
//!   • zero-gain pathways: interests, roles, creative style, moral traits,
//!     relationship behavior, competence, intelligence — by construction;
//!   • two files with the same preset are NOT identical (seeded sampling);
//!   • everything is mutable (set_embodiment), auditable (history), reversible.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

pub const GAIN_CAP: f32 = 0.3;
pub const N_HORMONE_AXES: usize = 16;

pub const AXE_T: &str = "t-like";
pub const AXE_E2: &str = "e2-like";
pub const AXE_P: &str = "p-like";
pub const AXE_OXT: &str = "oxt";
pub const AXE_AVP: &str = "avp";
pub const AXE_STRESS: &str = "stressReactivity";
pub const AXE_AROUSAL: &str = "arousalBaseline";
pub const AXE_REWARD: &str = "rewardSens";
pub const AXE_SOCIAL_REWARD: &str = "socialRewardSens";
pub const AXE_NOVELTY: &str = "noveltySeeking";
pub const AXE_RISK: &str = "riskTolerance";
pub const AXE_AFFILIATIVE: &str = "affiliative";
pub const AXE_ASSERTIVE: &str = "assertiveness";
pub const AXE_SENSORY: &str = "sensorySens";
pub const AXE_AESTHETIC: &str = "aestheticBias";
pub const AXE_VOICE: &str = "voiceMaturation";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbodimentPreset {
    Male,
    Female,
    Custom,
    Mixed,
    NonBinary,
    UserDefined,
}

/// Chromosomal ground truth (recorded at creation, immutable — nature).
/// The karyotype selects the gonadal hormone program; it is data, not a
/// concept: nothing in the substrate reads "XX" to decide behavior — the
/// hormone profile it produces is what the machinery reacts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Karyotype {
    Xx,
    Xy,
    Xxy,
    X0,
    Chimeric,
}

impl Karyotype {
    pub fn as_str(&self) -> &'static str {
        match self {
            Karyotype::Xx => "xx",
            Karyotype::Xy => "xy",
            Karyotype::Xxy => "xxy",
            Karyotype::X0 => "x0",
            Karyotype::Chimeric => "chimeric",
        }
    }

    pub fn from_str(s: &str) -> Option<Karyotype> {
        match s.to_lowercase().as_str() {
            "xx" => Some(Karyotype::Xx),
            "xy" => Some(Karyotype::Xy),
            "xxy" => Some(Karyotype::Xxy),
            "x0" | "xo" => Some(Karyotype::X0),
            "chimeric" | "chimera" | "mixed" => Some(Karyotype::Chimeric),
            _ => None,
        }
    }

    /// The gonadal program implied by the chromosomes (deterministic).
    pub fn gonadal_program(&self) -> EmbodimentPreset {
        match self {
            Karyotype::Xy => EmbodimentPreset::Male,
            Karyotype::Xx => EmbodimentPreset::Female,
            Karyotype::Xxy => EmbodimentPreset::NonBinary,
            Karyotype::X0 => EmbodimentPreset::Custom,
            Karyotype::Chimeric => EmbodimentPreset::Mixed,
        }
    }

    /// Random karyotype for a child (50/50 XX/XY — sex is chance, not choice).
    pub fn random_child(rng: &mut crate::rng::Rng) -> Karyotype {
        if rng.next_u64_below(2) == 0 {
            Karyotype::Xx
        } else {
            Karyotype::Xy
        }
    }
}

impl EmbodimentPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmbodimentPreset::Male => "male",
            EmbodimentPreset::Female => "female",
            EmbodimentPreset::Custom => "custom",
            EmbodimentPreset::Mixed => "mixed",
            EmbodimentPreset::NonBinary => "non-binary",
            EmbodimentPreset::UserDefined => "user-defined",
        }
    }

    pub fn from_str(s: &str) -> Option<EmbodimentPreset> {
        match s {
            "male" => Some(EmbodimentPreset::Male),
            "female" => Some(EmbodimentPreset::Female),
            "custom" => Some(EmbodimentPreset::Custom),
            "mixed" => Some(EmbodimentPreset::Mixed),
            "non-binary" | "nonbinary" => Some(EmbodimentPreset::NonBinary),
            "user-defined" => Some(EmbodimentPreset::UserDefined),
            _ => None,
        }
    }
}

/// (mean, spread) for one modulation axis.
pub type AxisPrior = (&'static str, f32, f32);

/// Axis order is canonical (index = axis id below).
pub const AXIS_ORDER: [&str; N_HORMONE_AXES] = [
    AXE_T, AXE_E2, AXE_P, AXE_OXT, AXE_AVP, AXE_STRESS, AXE_AROUSAL, AXE_REWARD,
    AXE_SOCIAL_REWARD, AXE_NOVELTY, AXE_RISK, AXE_AFFILIATIVE, AXE_ASSERTIVE,
    AXE_SENSORY, AXE_AESTHETIC, AXE_VOICE,
];

/// Biologically-informed digital-analogue priors. Spreads are wide on purpose:
/// distributions overlap heavily between presets — embodiment nudges, it never
/// dictates. Means are tendencies, not prescriptions.
pub fn priors_for(preset: EmbodimentPreset) -> [AxisPrior; N_HORMONE_AXES] {
    let neutral: [AxisPrior; N_HORMONE_AXES] = [
        (AXE_T, 0.50, 0.18), (AXE_E2, 0.50, 0.18), (AXE_P, 0.50, 0.18),
        (AXE_OXT, 0.50, 0.18), (AXE_AVP, 0.50, 0.18), (AXE_STRESS, 0.50, 0.18),
        (AXE_AROUSAL, 0.50, 0.18), (AXE_REWARD, 0.50, 0.18), (AXE_SOCIAL_REWARD, 0.50, 0.18),
        (AXE_NOVELTY, 0.50, 0.18), (AXE_RISK, 0.50, 0.18), (AXE_AFFILIATIVE, 0.50, 0.18),
        (AXE_ASSERTIVE, 0.50, 0.18), (AXE_SENSORY, 0.50, 0.18), (AXE_AESTHETIC, 0.50, 0.18),
        (AXE_VOICE, 0.50, 0.18),
    ];
    match preset {
        EmbodimentPreset::Custom => neutral,
        EmbodimentPreset::UserDefined => neutral, // user edits via API/CLI later
        EmbodimentPreset::Male => [
            (AXE_T, 0.68, 0.20), (AXE_E2, 0.36, 0.18), (AXE_P, 0.32, 0.16),
            (AXE_OXT, 0.44, 0.20), (AXE_AVP, 0.58, 0.18), (AXE_STRESS, 0.46, 0.20),
            (AXE_AROUSAL, 0.52, 0.18), (AXE_REWARD, 0.46, 0.20), (AXE_SOCIAL_REWARD, 0.40, 0.20),
            (AXE_NOVELTY, 0.56, 0.20), (AXE_RISK, 0.60, 0.20), (AXE_AFFILIATIVE, 0.40, 0.20),
            (AXE_ASSERTIVE, 0.64, 0.20), (AXE_SENSORY, 0.42, 0.18), (AXE_AESTHETIC, 0.48, 0.18),
            (AXE_VOICE, 0.56, 0.18),
        ],
        EmbodimentPreset::Female => [
            (AXE_T, 0.34, 0.18), (AXE_E2, 0.68, 0.20), (AXE_P, 0.60, 0.20),
            (AXE_OXT, 0.64, 0.20), (AXE_AVP, 0.44, 0.20), (AXE_STRESS, 0.56, 0.20),
            (AXE_AROUSAL, 0.46, 0.18), (AXE_REWARD, 0.56, 0.20), (AXE_SOCIAL_REWARD, 0.64, 0.20),
            (AXE_NOVELTY, 0.44, 0.20), (AXE_RISK, 0.40, 0.20), (AXE_AFFILIATIVE, 0.62, 0.20),
            (AXE_ASSERTIVE, 0.46, 0.20), (AXE_SENSORY, 0.60, 0.20), (AXE_AESTHETIC, 0.56, 0.18),
            (AXE_VOICE, 0.50, 0.18),
        ],
        EmbodimentPreset::Mixed => [
            (AXE_T, 0.52, 0.24), (AXE_E2, 0.52, 0.24), (AXE_P, 0.46, 0.22),
            (AXE_OXT, 0.54, 0.24), (AXE_AVP, 0.50, 0.22), (AXE_STRESS, 0.50, 0.22),
            (AXE_AROUSAL, 0.50, 0.22), (AXE_REWARD, 0.50, 0.22), (AXE_SOCIAL_REWARD, 0.52, 0.22),
            (AXE_NOVELTY, 0.50, 0.22), (AXE_RISK, 0.50, 0.22), (AXE_AFFILIATIVE, 0.52, 0.22),
            (AXE_ASSERTIVE, 0.50, 0.22), (AXE_SENSORY, 0.50, 0.22), (AXE_AESTHETIC, 0.50, 0.22),
            (AXE_VOICE, 0.50, 0.22),
        ],
        EmbodimentPreset::NonBinary => [
            (AXE_T, 0.50, 0.26), (AXE_E2, 0.50, 0.26), (AXE_P, 0.50, 0.24),
            (AXE_OXT, 0.54, 0.24), (AXE_AVP, 0.50, 0.24), (AXE_STRESS, 0.50, 0.24),
            (AXE_AROUSAL, 0.50, 0.24), (AXE_REWARD, 0.52, 0.24), (AXE_SOCIAL_REWARD, 0.54, 0.24),
            (AXE_NOVELTY, 0.52, 0.24), (AXE_RISK, 0.50, 0.24), (AXE_AFFILIATIVE, 0.54, 0.24),
            (AXE_ASSERTIVE, 0.50, 0.24), (AXE_SENSORY, 0.52, 0.24), (AXE_AESTHETIC, 0.52, 0.24),
            (AXE_VOICE, 0.50, 0.24),
        ],
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HormoneAxisState {
    pub axis: String,
    pub prior_mean: f32,
    pub prior_spread: f32,
    /// Sampled value ∈ [0.05, 0.95]; what actually modulates.
    pub current: f32,
    pub gain_cap: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbodimentChange {
    pub at: u64,
    pub from: String,
    pub to: String,
    pub by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HormoneProfile {
    pub preset: String,
    /// Chromosomal ground truth recorded at creation (immutable).
    #[serde(default = "default_karyotype")]
    pub karyotype: Karyotype,
    pub axes: Vec<HormoneAxisState>,
    /// Baseline deltas applied to the 8 modulator axes (removed on re-embodiment).
    pub mod_deltas: [f32; 8],
    pub mutable: bool,
    pub history: Vec<EmbodimentChange>,
}

fn default_karyotype() -> Karyotype {
    Karyotype::Xx
}

/// Preset → chromosomal ground truth (the authored mapping; the karyotype
/// then implies the gonadal program — a circle closed on purpose: presets
/// are human labels for the underlying chromosome-gonad chain).
pub fn karyotype_of(preset: EmbodimentPreset) -> Karyotype {
    match preset {
        EmbodimentPreset::Male => Karyotype::Xy,
        EmbodimentPreset::Female => Karyotype::Xx,
        EmbodimentPreset::NonBinary => Karyotype::Xxy,
        EmbodimentPreset::Mixed => Karyotype::Chimeric,
        EmbodimentPreset::Custom | EmbodimentPreset::UserDefined => Karyotype::Xx,
    }
}

/// Bounded per-axis gain in [-GAIN_CAP, +GAIN_CAP] around the neutral 0.5.
pub fn axis_gain(current: f32) -> f32 {
    ((current - 0.5) * 0.6).clamp(-GAIN_CAP, GAIN_CAP)
}

impl HormoneProfile {
    /// Neutral profile with priors at their means — the deterministic fallback
    /// for files created before the embodiment system existed (no RNG draws,
    /// so load never desyncs the noise stream).
    pub fn neutral() -> HormoneProfile {
        let priors = priors_for(EmbodimentPreset::Custom);
        let axes = priors
            .iter()
            .enumerate()
            .map(|(i, (_name, mean, spread))| HormoneAxisState {
                axis: AXIS_ORDER[i].to_string(),
                prior_mean: *mean,
                prior_spread: *spread,
                current: *mean,
                gain_cap: GAIN_CAP,
            })
            .collect();
        HormoneProfile {
            preset: "custom".to_string(),
            karyotype: Karyotype::Xx,
            axes,
            mod_deltas: [0.0; 8],
            mutable: true,
            history: Vec::new(),
        }
    }

    pub fn sample(preset: EmbodimentPreset, rng: &mut Rng) -> HormoneProfile {
        let priors = priors_for(preset);
        Self::sample_from_priors(priors, karyotype_of(preset), rng)
    }

    /// Sample from explicit priors with a recorded karyotype (heredity path).
    pub fn sample_from_priors(
        priors: [AxisPrior; N_HORMONE_AXES],
        karyotype: Karyotype,
        rng: &mut Rng,
    ) -> HormoneProfile {
        let mut axes = Vec::with_capacity(N_HORMONE_AXES);
        for (i, (_name, mean, spread)) in priors.iter().enumerate() {
            let current = (mean + (rng.next_f32() - 0.5) * 2.0 * spread).clamp(0.05, 0.95);
            axes.push(HormoneAxisState {
                axis: AXIS_ORDER[i].to_string(),
                prior_mean: *mean,
                prior_spread: *spread,
                current,
                gain_cap: GAIN_CAP,
            });
        }
        HormoneProfile {
            preset: karyotype.gonadal_program().as_str().to_string(),
            karyotype,
            axes,
            mod_deltas: [0.0; 8],
            mutable: true,
            history: Vec::new(),
        }
    }
}

/// Which sex chromosome a gamete carries (the child's karyotype is decided
/// here: the ovum always carries X; the sperm carries X or Y at random).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SexChromosome {
    X,
    Y,
}

/// What a gamete is: the ovum (X always) or the sperm (X or Y at random).
/// A birth requires one of each — the structural rule that a file can
/// never be made by one parent alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameteKind {
    Ovum,
    Sperm,
}

/// A gamete — the parent's contribution to a child. NOT the parent's
/// profile: a one-shot sampled packet (per-axis draws from the parent's
/// priors, + the sex chromosome). Produced at union, consumed at conception.
/// A child requires both an ovum and a sperm — never one parent alone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gamete {
    pub donor: String,
    pub kind: GameteKind,
    pub sex_chromosome: SexChromosome,
    /// Sampled per-axis prior values (canonical AXIS_ORDER).
    pub axes: Vec<f32>,
    /// The eyes, carried like the priors: the ovum provides the machinery
    /// the child is built from ("" = handcrafted extractor). No rules —
    /// the child is born from the egg, so it gets the egg's eyes.
    #[serde(default)]
    pub encoder: String,
    #[serde(default)]
    pub encoder_model_sha256: Option<String>,
    pub produced_tick: u64,
}

impl HormoneProfile {
    /// Produce a gamete from this parent: per-axis sample of its priors plus
    /// its sex-chromosome contribution. Mothers always contribute X; fathers
    /// contribute X or Y at random (that draw decides the child's karyotype).
    pub fn produce_gamete(
        &self,
        donor: &str,
        is_mother: bool,
        tick: u64,
        rng: &mut Rng,
        encoder: &str,
        encoder_model_sha256: Option<String>,
    ) -> Gamete {
        let kind = if is_mother { GameteKind::Ovum } else { GameteKind::Sperm };
        let sex_chromosome = if is_mother {
            SexChromosome::X
        } else if rng.next_u64_below(2) == 0 {
            SexChromosome::X
        } else {
            SexChromosome::Y
        };
        let axes = AXIS_ORDER
            .iter()
            .map(|axis| {
                let s = self.axis_state(axis);
                // Meiotic draw: the gamete carries a sample of the parent's
                // prior (not a copy, not the whole profile).
                (s.prior_mean + (rng.next_f32() - 0.5) * s.prior_spread).clamp(0.05, 0.95)
            })
            .collect();
        Gamete {
            donor: donor.to_string(),
            kind,
            sex_chromosome,
            axes,
            encoder: encoder.to_string(),
            encoder_model_sha256,
            produced_tick: tick,
        }
    }

    /// Child karyotype from the parents' gametes: X + (X or Y).
    /// The egg always carries X; only the sperm's chromosome decides.
    pub fn karyotype_from_gametes(_egg: &Gamete, sperm: &Gamete) -> Karyotype {
        match sperm.sex_chromosome {
            SexChromosome::X => Karyotype::Xx,
            SexChromosome::Y => Karyotype::Xy,
        }
    }

    /// Heredity: the child's priors are a recombination of the two gametes —
    /// per-axis draw between the egg's and the sperm's sampled values, plus
    /// small mutation noise. Both gametes are required; a file can never be
    /// made by one parent alone.
    pub fn child_priors(
        egg: &Gamete,
        sperm: &Gamete,
        rng: &mut Rng,
    ) -> [AxisPrior; N_HORMONE_AXES] {
        let mut out: [AxisPrior; N_HORMONE_AXES] = [(AXE_T, 0.5, 0.18); N_HORMONE_AXES];
        for (i, axis) in AXIS_ORDER.iter().enumerate() {
            let e = egg.axes.get(i).copied().unwrap_or(0.5);
            let s = sperm.axes.get(i).copied().unwrap_or(0.5);
            // Recombination: which gamete's copy this axis inherits.
            let base_mean = if rng.next_u64_below(2) == 0 { e } else { s };
            // Small mutation: the child is not a copy of either gamete.
            let mean = (base_mean + (rng.next_f32() - 0.5) * 0.08).clamp(0.05, 0.95);
            let spread = 0.18 + 0.02;
            out[i] = (*axis, mean, spread);
        }
        out
    }
}

impl HormoneProfile {
    /// Per-axis current-state accessor (used by heredity and affinity).
    pub fn axis_state(&self, axis: &str) -> HormoneAxisState {
        self.axes
            .iter()
            .find(|a| a.axis == axis)
            .cloned()
            .unwrap_or(HormoneAxisState {
                axis: axis.to_string(),
                prior_mean: 0.5,
                prior_spread: 0.18,
                current: 0.5,
                gain_cap: GAIN_CAP,
            })
    }

    pub fn gain(&self, axis: &str) -> f32 {
        self.axes
            .iter()
            .find(|a| a.axis == axis)
            .map(|a| axis_gain(a.current))
            .unwrap_or(0.0)
    }

    /// Compute the modulator-baseline deltas (da, 5ht, ne, ach, ecb, cort, oxt, avp).
    pub fn compute_mod_deltas(&self) -> [f32; 8] {
        [
            self.gain(AXE_REWARD) * 0.8,        // da ← reward sensitivity
            0.0,                                // 5ht: stability axis, embodiment-neutral
            self.gain(AXE_AROUSAL) * 0.8,       // ne ← arousal baseline
            0.0,                                // ach: attention/plasticity, embodiment-neutral
            0.0,                                // ecb: flexibility, embodiment-neutral
            self.gain(AXE_STRESS) * 0.8,        // cort ← stress reactivity
            self.gain(AXE_OXT) * 0.8,           // oxt ← oxytocin-like
            self.gain(AXE_AVP) * 0.8,           // avp ← vasopressin-like
        ]
    }

    /// Event-processing gains used by the brain (bounded).
    pub fn event_gains(&self) -> EventGains {
        EventGains {
            novelty_w: 0.35 + 0.12 * self.gain(AXE_NOVELTY),
            emotion_w: 0.35 + 0.12 * self.gain(AXE_SOCIAL_REWARD),
            nudge_gain: 0.02 * (1.0 + 0.3 * self.gain(AXE_SENSORY)),
            learning_w: 1.0 + 0.3 * self.gain(AXE_REWARD),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EventGains {
    pub novelty_w: f32,
    pub emotion_w: f32,
    pub nudge_gain: f32,
    pub learning_w: f32,
}

impl Default for EventGains {
    fn default() -> Self {
        EventGains {
            novelty_w: 0.35,
            emotion_w: 0.35,
            nudge_gain: 0.02,
            learning_w: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn karyotype_maps_to_gonadal_program() {
        assert_eq!(karyotype_of(EmbodimentPreset::Male), Karyotype::Xy);
        assert_eq!(karyotype_of(EmbodimentPreset::Female), Karyotype::Xx);
        assert_eq!(karyotype_of(EmbodimentPreset::NonBinary), Karyotype::Xxy);
        assert_eq!(karyotype_of(EmbodimentPreset::Mixed), Karyotype::Chimeric);
        assert_eq!(Karyotype::Xy.gonadal_program(), EmbodimentPreset::Male);
        assert_eq!(Karyotype::Xx.gonadal_program(), EmbodimentPreset::Female);
        // Random child karyotype is 50/50 across a population.
        let mut rng = crate::rng::Rng::new(99);
        let mut xx = 0;
        for _ in 0..200 {
            if Karyotype::random_child(&mut rng) == Karyotype::Xx {
                xx += 1;
            }
        }
        assert!((80..=120).contains(&xx), "XX count {xx} should be ~100/200");
    }

    #[test]
    fn kin_recognition_by_profile_similarity() {
        // Parents and child share chemistry (inheritance) → the profile
        // distance between parent and child is small; between strangers it
        // is larger. Recognition = similarity, attraction = complementarity.
        let mut rng = crate::rng::Rng::new(11);
        let mother = HormoneProfile::sample(EmbodimentPreset::Female, &mut rng);
        let father = HormoneProfile::sample(EmbodimentPreset::Male, &mut rng);
        let mut rg = crate::rng::Rng::new(12);
        let egg = mother.produce_gamete("mother", true, 0, &mut rg, "", None);
        let mut rg2 = crate::rng::Rng::new(13);
        let sperm = father.produce_gamete("father", false, 0, &mut rg2, "", None);
        let mut rg3 = crate::rng::Rng::new(14);
        let child_priors = HormoneProfile::child_priors(&egg, &sperm, &mut rg3);
        let child = HormoneProfile::sample_from_priors(child_priors, Karyotype::Xx, &mut rg3);
        // A stranger (different seed, neutral profile).
        let mut rg4 = crate::rng::Rng::new(15);
        let stranger = HormoneProfile::sample(EmbodimentPreset::Custom, &mut rg4);
        // Kin recognition on the gonadal axes (T/E2/P — the sex-differentiating
        // chemistry): the child's gonadal axes each inherited from ONE parent,
        // so the child always matches at least one parent there; a stranger
        // matches none (its neutral 0.5 priors sit between the parental poles).
        let gonadal = [AXE_T, AXE_E2, AXE_P];
        let gonadal_matches = |a: &HormoneProfile, b: &HormoneProfile| -> usize {
            gonadal
                .iter()
                .filter(|ax| (a.axis_state(ax).prior_mean - b.axis_state(ax).prior_mean).abs() < 0.10)
                .count()
        };
        let m_matches = gonadal_matches(&mother, &child);
        let f_matches = gonadal_matches(&father, &child);
        let s_matches = gonadal_matches(&stranger, &child);
        // Recognition: the child shares gonadal chemistry with at least one
        // parent (inheritance), and with no stranger.
        assert!(m_matches >= 1 || f_matches >= 1,
            "child shares no gonadal chemistry with either parent (mother {m_matches}, father {f_matches})");
        assert_eq!(s_matches, 0, "stranger must share no gonadal chemistry (got {s_matches})");
        // And the parents, being complementary to each other, share none of
        // the gonadal poles either (that is the attraction signal).
        assert_eq!(gonadal_matches(&mother, &father), 0,
            "parents must be complementary on gonadal axes (attraction, not kin)");
    }

    #[test]
    fn child_priors_inherit_from_both_parents() {
        let mut rng = crate::rng::Rng::new(7);
        let mother = HormoneProfile::sample(EmbodimentPreset::Female, &mut rng);
        let father = HormoneProfile::sample(EmbodimentPreset::Male, &mut rng);
        // Gonadal priors differ between the parents (T axis).
        assert!(father.axis_state(AXE_T).prior_mean > mother.axis_state(AXE_T).prior_mean);
        // Gametes: mother's ovum always X; father's sperm X or Y.
        let mut rg = crate::rng::Rng::new(7);
        let egg = mother.produce_gamete("mother", true, 0, &mut rg, "", None);
        let sperm = father.produce_gamete("father", false, 0, &mut rg, "", None);
        assert_eq!(egg.sex_chromosome, SexChromosome::X);
        assert!(sperm.sex_chromosome == SexChromosome::X || sperm.sex_chromosome == SexChromosome::Y);
        // Karyotype follows the sperm's chromosome.
        let k = HormoneProfile::karyotype_from_gametes(&egg, &sperm);
        assert!(k == Karyotype::Xx || k == Karyotype::Xy);
        // Father with Y sperm → XY child; with X sperm → XX child.
        let mut rg2 = crate::rng::Rng::new(1);
        let _y_sperm = father.produce_gamete("father", false, 0, &mut rg2, "", None);
        // find a Y-bearing sperm by scanning seeds until one appears
        let mut y_sperm = None;
        for s in 0..50 {
            let mut r = crate::rng::Rng::new(5000 + s);
            let sp = father.produce_gamete("father", false, 0, &mut r, "", None);
            if sp.sex_chromosome == SexChromosome::Y {
                y_sperm = Some(sp);
                break;
            }
        }
        let y_sperm = y_sperm.expect("a Y-bearing sperm must appear within 50 seeds");
        assert_eq!(HormoneProfile::karyotype_from_gametes(&egg, &y_sperm), Karyotype::Xy);
        let mut rg3 = crate::rng::Rng::new(2);
        let x_sperm = father.produce_gamete("father", false, 0, &mut rg3, "", None);
        let x_sperm = if x_sperm.sex_chromosome == SexChromosome::X { x_sperm } else {
            let mut r = crate::rng::Rng::new(9000);
            father.produce_gamete("father", false, 0, &mut r, "", None)
        };
        if x_sperm.sex_chromosome == SexChromosome::X {
            assert_eq!(HormoneProfile::karyotype_from_gametes(&egg, &x_sperm), Karyotype::Xx);
        }
        // The child is a blend of the two gametes: both sources appear across seeds.
        let mut saw_egg = false;
        let mut saw_sperm = false;
        let mut rg4 = crate::rng::Rng::new(8);
        let child = HormoneProfile::child_priors(&egg, &sperm, &mut rg4);
        let lo = egg.axes[0].min(sperm.axes[0]);
        let hi = egg.axes[0].max(sperm.axes[0]);
        assert!(child[0].1 >= lo - 0.06 && child[0].1 <= hi + 0.06,
            "child T {:.3} outside gamete range [{lo:.3},{hi:.3}]", child[0].1);
        for s in 0..30 {
            let mut r = crate::rng::Rng::new(1000 + s);
            let c = HormoneProfile::child_priors(&egg, &sperm, &mut r);
            let t = c[0].1;
            if (t - egg.axes[0]).abs() < 0.03 { saw_egg = true; }
            if (t - sperm.axes[0]).abs() < 0.03 { saw_sperm = true; }
        }
        assert!(saw_egg && saw_sperm, "children must inherit from both gametes");
    }

    #[test]
    fn presets_differ_but_overlap() {
        let male = priors_for(EmbodimentPreset::Male);
        let female = priors_for(EmbodimentPreset::Female);
        // t-like means differ in the expected direction (digital analogue).
        assert!(male[0].1 > female[0].1);
        assert!(female[1].1 > male[1].1);
        // …but spreads are wide enough that draws overlap substantially.
        assert!(male[0].1 - male[0].2 < female[0].1 + female[0].2);
    }

    #[test]
    fn sampling_is_seeded_and_bounded() {
        let mut r1 = Rng::new(5);
        let mut r2 = Rng::new(5);
        let a = HormoneProfile::sample(EmbodimentPreset::Male, &mut r1);
        let b = HormoneProfile::sample(EmbodimentPreset::Male, &mut r2);
        assert_eq!(a.axes.len(), N_HORMONE_AXES);
        for (x, y) in a.axes.iter().zip(b.axes.iter()) {
            assert_eq!(x.current, y.current, "deterministic per seed");
            assert!((0.05..=0.95).contains(&x.current));
            assert!(axis_gain(x.current).abs() <= GAIN_CAP);
        }
        // Same seed, different presets → different samples.
        let mut r3 = Rng::new(5);
        let c = HormoneProfile::sample(EmbodimentPreset::Female, &mut r3);
        let mut any_diff = false;
        for (x, y) in a.axes.iter().zip(c.axes.iter()) {
            if x.current != y.current {
                any_diff = true;
            }
        }
        assert!(any_diff, "male and female priors produce different draws");
    }

    #[test]
    fn gains_are_bounded_and_neutral_axes_zero() {
        let mut rng = Rng::new(9);
        let p = HormoneProfile::sample(EmbodimentPreset::Female, &mut rng);
        let d = p.compute_mod_deltas();
        for g in d {
            assert!(g.abs() <= GAIN_CAP, "delta {g}");
        }
        assert_eq!(d[1], 0.0, "5ht embodiment-neutral");
        assert_eq!(d[3], 0.0, "ach embodiment-neutral");
        assert_eq!(d[4], 0.0, "ecb embodiment-neutral");
        let eg = p.event_gains();
        assert!((0.20..0.50).contains(&eg.novelty_w));
        assert!((0.20..0.50).contains(&eg.emotion_w));
    }
}
