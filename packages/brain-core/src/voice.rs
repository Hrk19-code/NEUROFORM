//! Voice organ — simulated vocal expression (DESIGN.md §7).
//!
//! M5 core (headless): a biologically-inspired vocal apparatus state
//! (breath / subglottal pressure / larynx / tract / articulation), a
//! developmental voice identity that drifts with use (never locks), audited
//! user overrides, and the expressive contract the UI consumes: a `VoicePlan`
//! — rendered parameters + prosodic stage structure + a mapping onto a TTS
//! backend (edge/piper/cloud). The TTS synthesis itself lives in the desktop
//! shell; this organ decides *how* the voice sounds, from brain state.
//!
//! M5 extension — heard voices (the mimicry foundation, DESIGN.md §5.8/§7
//! and the M5-NOTES "emergent pathway" note): the organ can ingest extracted
//! voice features from an audio sidecar (`tools/audio-extract.py`, 16 dims)
//! as `HeardVoice` patterns. Mimicry is *not* scripted: it emerges only when
//! (a) the user grants consent for that heard voice AND the global
//! voice-learning gate is enabled (both default OFF), and (b) the file
//! repeatedly speaks toward the pattern, so its identity envelope drifts
//! gradually and boundedly toward it. Every blend is counted and recorded;
//! refusals are counted too (audit visibility). Nothing locks: heard salience
//! decays without reinforcement, overrides always win, and the bias-audit
//! engine can monitor `mimicry_uses` for fixation.
//!
//! Embodiment presets contribute probabilistic priors only (pitch baseline,
//! formant shift tendencies) — bounded, mutable, audited; nothing is locked.
//! All deterministic: a plan is a pure function of state.

use serde::{Deserialize, Serialize};

use crate::drawing::features_cosine;
use crate::embodiment::HormoneProfile;

// --- vocal apparatus (biological analogues) ----------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VocalApparatus {
    pub breath: f32,              // lung volume 0..1
    pub subglottal_pressure: f32, // 0..1
    pub larynx_tension: f32,      // 0..1
    pub fold_stability: f32,      // 1 = stable; jitter = 1 - stability
    pub tract_length: f32,        // relative 0.8..1.2 (maturation)
    pub tongue_tendency: f32,     // -1 back .. +1 front
    pub lip_jaw_openness: f32,    // 0..1
    pub tempo: f32,               // words-per-minute tendency / 200
    pub pause_tendency: f32,      // 0..1
    pub prosody_range: f32,       // 0..1
}

impl VocalApparatus {
    pub fn new() -> Self {
        VocalApparatus {
            breath: 0.9,
            subglottal_pressure: 0.4,
            larynx_tension: 0.3,
            fold_stability: 0.95,
            tract_length: 1.0,
            tongue_tendency: 0.0,
            lip_jaw_openness: 0.4,
            tempo: 0.7,
            pause_tendency: 0.3,
            prosody_range: 0.6,
        }
    }

    /// Slow idle dynamics: breath recovers, tension follows arousal,
    /// fold stability degrades with fatigue. Deterministic (no rng).
    pub fn step_idle(&mut self, dt_ticks: f32, arousal: f32, fatigue: f32) {
        let dt = (dt_ticks * 0.1).min(1.0); // scale to seconds-ish
        self.breath = (self.breath + dt * 0.02).min(1.0);
        self.subglottal_pressure = (self.subglottal_pressure + (0.3 - self.subglottal_pressure) * dt * 0.01).clamp(0.0, 1.0);
        self.larynx_tension = (self.larynx_tension + (0.2 + 0.5 * arousal - self.larynx_tension) * dt * 0.01).clamp(0.0, 1.0);
        self.fold_stability = (self.fold_stability - fatigue * dt * 0.002).clamp(0.6, 1.0);
    }
}

// --- voice identity (developmental, drifts, never locks) ---------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VoiceIdentity {
    pub pitch_mean: f32,        // 0..1 (relative)
    pub pitch_range: f32,       // 0..1
    pub formant_shift: f32,     // -0.2..0.2 (tract-length driven)
    pub breathiness: f32,
    pub warmth: f32,
    pub brightness: f32,
    pub articulation_crispness: f32,
    pub expressiveness: f32,
    pub tension: f32,
    pub softness: f32,
    pub intimacy: f32,
    pub confidence: f32,
    pub maturity: f32,          // 0..1 developmental
}

/// Probabilistic priors from an embodiment preset — means + bounded spread,
/// never deterministic locks (§4.8). The e2-like vs t-like axis modulates the
/// pitch baseline and formant tendency; everything else is symmetric.
pub fn identity_priors(profile: &HormoneProfile) -> VoiceIdentity {
    let level = |name: &str| {
        profile
            .axes
            .iter()
            .find(|a| a.axis == name)
            .map(|a| a.current)
            .unwrap_or(0.0)
    };
    let t = level(crate::embodiment::AXE_T);
    let e2 = level(crate::embodiment::AXE_E2);
    let sex_hormone = (e2 - t).clamp(-1.0, 1.0);
    VoiceIdentity {
        pitch_mean: 0.5 + 0.12 * sex_hormone,
        pitch_range: 0.55,
        formant_shift: 0.08 * sex_hormone,
        breathiness: 0.2,
        warmth: 0.5,
        brightness: 0.5,
        articulation_crispness: 0.6,
        expressiveness: 0.5,
        tension: 0.3,
        softness: 0.5,
        intimacy: 0.3,
        confidence: 0.5,
        maturity: 0.3,
    }
}

// --- rendered plan (the output contract) -------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceParams {
    pub pitch: f32,       // 0..1
    pub rate: f32,        // words/min
    pub energy: f32,      // 0..1
    pub breathiness: f32,
    pub warmth: f32,
    pub brightness: f32,
    pub roughness: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TtsMapping {
    pub backend: String,
    pub pitch_semitones: f32,
    pub rate_mult: f32,
    pub energy_gain_db: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceStage {
    pub kind: String, // "pause" | "emphasis" | "soft"
    pub at_word: usize,
    pub intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoicePlan {
    pub text: String,
    pub params: VoiceParams,
    pub tts: TtsMapping,
    pub stages: Vec<VoiceStage>,
    pub emotional_coloring: String,
    /// Heard voice requested for mimicry blending (if any).
    pub toward: Option<u64>,
    /// Whether the blend actually applied (consent + gate on).
    pub blended: bool,
}

// --- overrides (audited, user-controlled) ------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceOverride {
    pub param: String,
    pub value: f32,
    pub tick: u64,
    pub reason: String,
}

// --- memory / history --------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceEvent {
    pub id: u64,
    pub tick: u64,
    pub affect: (f32, f32),
    pub params: VoiceParams,
    /// Which heard voice (if any) this utterance was blended toward.
    pub toward: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VoiceMemory {
    pub uses: u32,
    pub pitch_tendency: f32,
    pub tempo_tendency: f32,
    pub expressiveness_tendency: f32,
    /// Blended (learned) mimicry utterances — the bias-audit engine can
    /// monitor this for fixation (§14 loop monitors).
    pub mimicry_uses: u32,
    /// Requested but refused (consent or gate off) — audit visibility.
    pub refused_mimicry: u32,
}

// --- heard voices (mimicry foundation) ---------------------------------------
//
// Feature contract (16 dims, produced by tools/audio-extract.py and consumed
// deterministically here):
//   0 pitch_mean, 1 pitch_std, 2 rms_mean, 3 rms_std, 4 zcr_mean, 5 zcr_std,
//   6 attack_mean, 7 decay_mean, 8 energy_trend, 9 gap_mean, 10 voice_ratio,
//   11 seg_rate, 12 jitter, 13 shimmer, 14 duration_log, 15 crispness
// All normalized 0..1 unless noted. Keep the two files in lockstep.

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeardVoice {
    pub id: u64,
    pub label: String,
    pub features: Vec<f32>, // 16 dims per contract
    /// User consent for THIS voice — required (with the global gate) for the
    /// file to learn toward it. Default false; explicit, auditable, reversible.
    pub consent: bool,
    pub salience: f32,
    pub hear_count: u32,
    pub learnable_uses: u32,
    pub first_heard: u64,
    pub last_heard: u64,
}

/// Identity targets derived from a heard-voice feature vector (the mimicry
/// envelope). Pure function — deterministic.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct HeardTargets {
    pub pitch: f32,
    pub pitch_range: f32,
    pub rate_mult: f32,
    pub energy: f32,
    pub brightness: f32,
    pub roughness: f32,
    pub breathiness: f32,
    pub warmth: f32,
    pub crispness: f32,
}

pub fn heard_targets(f: &[f32]) -> HeardTargets {
    let g = |i: usize, def: f32| f.get(i).copied().unwrap_or(def).clamp(0.0, 1.0);
    HeardTargets {
        pitch: g(0, 0.5),
        pitch_range: (g(1, 0.2) * 3.0).clamp(0.0, 0.8),
        rate_mult: (0.75 + 0.6 * g(11, 0.5)).clamp(0.7, 1.4),
        energy: g(2, 0.5),
        brightness: g(4, 0.5),
        roughness: ((g(12, 0.2) + g(13, 0.2)) * 0.5).clamp(0.0, 0.7),
        breathiness: (0.15 + 0.6 * g(13, 0.2)).clamp(0.0, 0.8),
        warmth: (1.0 - 0.5 * g(4, 0.5)).clamp(0.0, 1.0),
        crispness: g(15, 0.6),
    }
}

// --- the organ ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceOrgan {
    pub apparatus: VocalApparatus,
    pub identity: VoiceIdentity,
    pub overrides: Vec<VoiceOverride>,
    pub memory: VoiceMemory,
    pub history: Vec<VoiceEvent>,
    pub next_id: u64,
    // M5: heard voices + mimicry gates
    pub heard: Vec<HeardVoice>,
    /// Global voice-learning gate — default OFF. Even with consent per voice,
    /// no mimicry blending happens while this is off.
    pub voice_learning_enabled: bool,
    pub next_heard_id: u64,
}

impl VoiceOrgan {
    pub fn new(profile: &HormoneProfile) -> Self {
        VoiceOrgan {
            apparatus: VocalApparatus::new(),
            identity: identity_priors(profile),
            overrides: Vec::new(),
            memory: VoiceMemory::default(),
            history: Vec::new(),
            next_id: 1,
            heard: Vec::new(),
            voice_learning_enabled: false,
            next_heard_id: 1,
        }
    }

    pub fn step_idle(&mut self, dt_ticks: f32, arousal: f32, fatigue: f32) {
        self.apparatus.step_idle(dt_ticks, arousal, fatigue);
        // Heard voices fade without reinforcement (non-permanence): salience
        // decays; voices that fall below threshold are dropped entirely.
        let dt = (dt_ticks * 0.1).min(1.0);
        let keep: Vec<HeardVoice> = self
            .heard
            .drain(..)
            .filter(|hv| {
                let s = hv.salience * (1.0 - dt * 0.001);
                s >= 0.04
            })
            .map(|mut hv| {
                hv.salience = hv.salience * (1.0 - dt * 0.001);
                hv
            })
            .collect();
        self.heard = keep;
    }

    fn override_of(&self, param: &str) -> Option<f32> {
        self.overrides.iter().rev().find(|o| o.param == param).map(|o| o.value)
    }

    /// Render the expressive plan for an utterance. Pure function of state —
    /// deterministic. Affect, fatigue, identity, overrides all shape it.
    pub fn speak(&mut self, text: &str, valence: f32, arousal: f32, energy: f32, fatigue: f32, tick: u64) -> VoicePlan {
        self.render(text, valence, arousal, energy, fatigue, tick, None)
    }

    /// Speak toward a heard voice (mimicry pathway). Emergent and gated:
    /// blending only happens when the heard voice has user consent AND the
    /// global learning gate is on; otherwise the plan renders normally and
    /// the refusal is counted for audit.
    pub fn speak_toward(&mut self, text: &str, valence: f32, arousal: f32, energy: f32, fatigue: f32, tick: u64, toward: Option<u64>) -> VoicePlan {
        self.render(text, valence, arousal, energy, fatigue, tick, toward)
    }

    fn render(&mut self, text: &str, valence: f32, arousal: f32, energy: f32, fatigue: f32, tick: u64, toward: Option<u64>) -> VoicePlan {
        let id = self.identity.clone();
        let app = &self.apparatus;
        let over = |p: &str, base: f32| self.override_of(p).unwrap_or(base);

        let mut pitch = over("pitch", (id.pitch_mean + valence * 0.02 * id.pitch_range + arousal * 0.05 * id.pitch_range).clamp(0.0, 1.0));
        let mut rate = over("rate", (150.0 * app.tempo * (1.0 + 0.2 * arousal) * (1.0 - 0.15 * fatigue)).clamp(60.0, 260.0));
        let mut energy = over("energy", ((0.4 + 0.35 * arousal + 0.15 * id.confidence) * (0.5 + 0.5 * energy)).clamp(0.0, 1.0));
        let mut breathiness = over("breathiness", (id.breathiness * (1.0 + 0.4 * fatigue)).clamp(0.0, 1.0));
        let mut warmth = over("warmth", id.warmth);
        let mut brightness = over("brightness", id.brightness);
        let mut roughness = over("roughness", ((1.0 - app.fold_stability) * 0.4 + fatigue * 0.2).clamp(0.0, 1.0));

        // --- mimicry blend (gated, audited, bounded) ----------------------
        let mut blended_toward: Option<u64> = None;
        let mut refused = false;
        if let Some(hv_id) = toward {
            if let Some(hv) = self.heard.iter().find(|h| h.id == hv_id) {
                if hv.consent && self.voice_learning_enabled {
                    let w = (0.4 * hv.salience).clamp(0.0, 0.6);
                    let t = heard_targets(&hv.features);
                    let lerp = |a: f32, b: f32, w: f32| a + (b - a) * w;
                    pitch = lerp(pitch, t.pitch, w);
                    rate = rate * lerp(1.0, t.rate_mult, w);
                    energy = lerp(energy, t.energy, w);
                    breathiness = lerp(breathiness, t.breathiness, w);
                    warmth = lerp(warmth, t.warmth, w);
                    brightness = lerp(brightness, t.brightness, w);
                    roughness = lerp(roughness, t.roughness, w);
                    blended_toward = Some(hv_id);
                    // Identity drift: gradual, bounded, never locked. Each
                    // blended utterance nudges the envelope; overrides still win.
                    let d = 0.02 * hv.salience;
                    self.identity.pitch_mean = (self.identity.pitch_mean + (t.pitch - self.identity.pitch_mean) * d).clamp(0.0, 1.0);
                    self.identity.warmth = (self.identity.warmth + (t.warmth - self.identity.warmth) * d).clamp(0.0, 1.0);
                    self.identity.brightness = (self.identity.brightness + (t.brightness - self.identity.brightness) * d).clamp(0.0, 1.0);
                    self.identity.articulation_crispness =
                        (self.identity.articulation_crispness + (t.crispness - self.identity.articulation_crispness) * d).clamp(0.0, 1.0);
                } else {
                    refused = true;
                }
            }
        }
        if blended_toward.is_some() {
            self.memory.mimicry_uses += 1;
            if let Some(hv) = self.heard.iter_mut().find(|h| h.id == blended_toward.unwrap()) {
                hv.learnable_uses += 1;
            }
        } else if refused {
            self.memory.refused_mimicry += 1;
        }

        // Prosodic structure from surface text.
        let mut stages = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut w = 0usize;
        for word in &words {
            if word.ends_with(['.', '!', '?', ';']) {
                stages.push(VoiceStage {
                    kind: "pause".into(),
                    at_word: w,
                    intensity: 0.4 + 0.5 * app.pause_tendency,
                });
            }
            if word.chars().any(|c| c.is_uppercase()) && word.len() > 1 {
                stages.push(VoiceStage {
                    kind: "emphasis".into(),
                    at_word: w,
                    intensity: 0.5 + 0.5 * id.expressiveness,
                });
            }
            w += 1;
        }

        let coloring = if valence > 0.3 {
            "bright"
        } else if valence < -0.3 {
            if arousal > 0.4 { "troubled" } else { "heavy" }
        } else if arousal < 0.25 {
            "calm"
        } else {
            "steady"
        };

        let plan = VoicePlan {
            text: text.to_string(),
            params: VoiceParams {
                pitch, rate, energy, breathiness, warmth, brightness, roughness,
            },
            tts: TtsMapping {
                backend: "shell-default".into(),
                pitch_semitones: (pitch - 0.5) * 12.0,
                rate_mult: rate / 150.0,
                energy_gain_db: (energy - 0.5) * 6.0,
            },
            stages,
            emotional_coloring: coloring.into(),
            toward,
            blended: blended_toward.is_some(),
        };

        // Memory: usage tendencies drift (bounded, never locked).
        let m = &mut self.memory;
        let n = m.uses as f32;
        m.pitch_tendency = (m.pitch_tendency * n + plan.params.pitch) / (n + 1.0);
        m.tempo_tendency = (m.tempo_tendency * n + plan.params.rate / 150.0) / (n + 1.0);
        m.expressiveness_tendency =
            (m.expressiveness_tendency * n + id.expressiveness) / (n + 1.0);
        m.uses += 1;
        self.history.push(VoiceEvent {
            id: self.next_id,
            tick,
            affect: (valence, arousal),
            params: plan.params.clone(),
            toward: blended_toward,
        });
        self.next_id += 1;
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        plan
    }

    /// Ingest an extracted voice pattern (16 dims from the audio sidecar).
    /// Similar patterns merge (cosine ≥ 0.85) into one heard voice whose
    /// features are the running mean. Consent is per voice; the global gate
    /// stays separate so the file can hear without ever learning.
    pub fn hear_pattern(&mut self, label: &str, features: Vec<f32>, consent: bool, salience: f32, tick: u64) -> u64 {
        let mut best: Option<(usize, f32)> = None;
        for (i, hv) in self.heard.iter().enumerate() {
            let c = features_cosine(&hv.features, &features);
            if best.map(|(_, bc)| c > bc).unwrap_or(true) {
                best = Some((i, c));
            }
        }
        if let Some((i, c)) = best {
            if c >= 0.85 {
                let hv = &mut self.heard[i];
                let n = hv.hear_count as f32;
                for (k, v) in hv.features.iter_mut().enumerate() {
                    if let Some(f) = features.get(k) {
                        *v = (*v * n + f) / (n + 1.0);
                    }
                }
                hv.hear_count += 1;
                hv.salience = (hv.salience + 0.1 * salience).min(1.0);
                hv.last_heard = tick;
                hv.consent = hv.consent || consent;
                return hv.id;
            }
        }
        let id = self.next_heard_id;
        self.next_heard_id += 1;
        self.heard.push(HeardVoice {
            id,
            label: label.to_string(),
            features,
            consent,
            salience: salience.clamp(0.05, 1.0),
            hear_count: 1,
            learnable_uses: 0,
            first_heard: tick,
            last_heard: tick,
        });
        id
    }

    /// Toggle the global voice-learning gate (default OFF).
    pub fn set_learning_enabled(&mut self, on: bool) {
        self.voice_learning_enabled = on;
    }

    /// Flip consent on a specific heard voice (reversible, audited via the
    /// voice list itself).
    pub fn set_consent(&mut self, heard_id: u64, on: bool) -> bool {
        match self.heard.iter_mut().find(|h| h.id == heard_id) {
            Some(h) => {
                h.consent = on;
                true
            }
            None => false,
        }
    }

    /// Audited user override: wins over rendered params, recorded, reversible.
    pub fn set_override(&mut self, param: &str, value: f32, reason: &str, tick: u64) -> bool {
        if !matches!(param, "pitch" | "rate" | "energy" | "breathiness" | "warmth" | "brightness" | "roughness") {
            return false;
        }
        self.overrides.push(VoiceOverride {
            param: param.to_string(),
            value: value.clamp(0.0, 1.0),
            tick,
            reason: reason.to_string(),
        });
        true
    }

    pub fn clear_override(&mut self, param: &str) {
        self.overrides.retain(|o| o.param != param);
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0x41c6_ce57_62a4_e9a5;
        for v in [
            self.identity.pitch_mean, self.identity.pitch_range,
            self.identity.formant_shift, self.identity.maturity,
            self.memory.pitch_tendency, self.memory.tempo_tendency,
            self.memory.mimicry_uses as f32, self.memory.refused_mimicry as f32,
            if self.voice_learning_enabled { 1.0 } else { 0.0 },
        ] {
            for b in v.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for o in self.overrides.iter() {
            for b in o.param.as_bytes() {
                h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in o.value.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for hv in self.heard.iter() {
            for b in hv.id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in hv.hear_count.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in [hv.consent as u8] {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for f in hv.features.iter() {
                for b in f.to_bits().to_le_bytes() {
                    h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    fn organ() -> VoiceOrgan {
        let mut rng = Rng::new(7);
        VoiceOrgan::new(&HormoneProfile::sample(crate::embodiment::EmbodimentPreset::Custom, &mut rng))
    }

    /// A plausible "low, warm, slow" heard voice (16 dims per contract).
    fn low_warm_features() -> Vec<f32> {
        vec![
            0.30, 0.10, 0.50, 0.12, 0.35, 0.08, // pitch, rms, zcr
            0.30, 0.35, 0.02, 0.40, 0.85, 0.60, // attack, decay, trend, gap, ratio, seg_rate
            0.08, 0.06, 0.55, 0.70,             // jitter, shimmer, duration, crispness
        ]
    }

    fn bright_fast_features() -> Vec<f32> {
        vec![0.9; 16]
    }

    #[test]
    fn plan_is_deterministic_and_affect_sensitive() {
        let mut v = organ();
        let calm = v.speak("the garden is quiet now.", 0.1, 0.2, 0.6, 0.1, 10);
        let calm2 = v.speak("the garden is quiet now.", 0.1, 0.2, 0.6, 0.1, 10);
        assert_eq!(calm.params.pitch.to_bits(), calm2.params.pitch.to_bits(), "deterministic");
        assert_eq!(calm.emotional_coloring, "calm");
        let stirred = v.speak("the garden is on FIRE!", -0.6, 0.8, 0.5, 0.1, 11);
        assert_eq!(stirred.emotional_coloring, "troubled");
        assert!(stirred.params.energy > calm.params.energy, "arousal raises energy");
        assert!(stirred.params.rate > calm.params.rate, "arousal raises rate");
        assert!(
            stirred.stages.iter().any(|s| s.kind == "emphasis"),
            "FIRE! gets emphasis"
        );
        assert!(stirred.stages.iter().any(|s| s.kind == "pause"));
    }

    #[test]
    fn fatigue_degrades_voice() {
        let mut v = organ();
        let fresh = v.speak("hello", 0.0, 0.3, 0.5, 0.05, 1);
        let tired = v.speak("hello", 0.0, 0.3, 0.3, 0.9, 2);
        assert!(tired.params.roughness > fresh.params.roughness, "fatigue roughens");
        assert!(tired.params.rate < fresh.params.rate, "fatigue slows");
        assert!(tired.params.breathiness > fresh.params.breathiness, "fatigue adds breath");
    }

    #[test]
    fn override_wins_and_is_audited() {
        let mut v = organ();
        assert!(v.set_override("pitch", 0.9, "user preference", 5));
        assert!(!v.set_override("nonsense", 0.5, "x", 5), "unknown params rejected");
        let plan = v.speak("hello world", 0.0, 0.3, 0.5, 0.1, 6);
        assert!((plan.params.pitch - 0.9).abs() < 1e-6, "override wins");
        assert_eq!(v.overrides.len(), 1);
        assert_eq!(v.overrides[0].reason, "user preference");
        v.clear_override("pitch");
        let plan2 = v.speak("hello world", 0.0, 0.3, 0.5, 0.1, 7);
        assert!((plan2.params.pitch - 0.9).abs() > 1e-3, "cleared override releases");
    }

    #[test]
    fn identity_drifts_with_use_but_stays_bounded() {
        let mut v = organ();
        for i in 0..50 {
            let plan = v.speak("a small phrase for the file", 0.2 * ((i % 5) as f32 - 2.0), 0.3, 0.5, 0.1, i);
            assert!((0.0..=1.0).contains(&plan.params.pitch));
            assert!((60.0..=260.0).contains(&plan.params.rate));
        }
        assert_eq!(v.memory.uses, 50);
        assert!((0.0..=1.0).contains(&v.memory.pitch_tendency));
        assert!(v.history.len() <= 100);
    }

    #[test]
    fn priors_respect_embodiment_without_locking() {
        let mut rng = Rng::new(3);
        let male = VoiceOrgan::new(&HormoneProfile::sample(crate::embodiment::EmbodimentPreset::Male, &mut rng));
        let female = VoiceOrgan::new(&HormoneProfile::sample(crate::embodiment::EmbodimentPreset::Female, &mut rng));
        // Tendencies differ in distribution, never in kind: both voices can
        // reach any pitch via override or drift.
        assert_ne!(male.identity.pitch_mean, female.identity.pitch_mean);
        for id in [&male.identity, &female.identity] {
            assert!((0.0..=1.0).contains(&id.pitch_mean));
            assert!((-0.2..=0.2).contains(&id.formant_shift));
        }
        let mut rng2 = Rng::new(4);
        let male2 = VoiceOrgan::new(&HormoneProfile::sample(crate::embodiment::EmbodimentPreset::Male, &mut rng2));
        assert_ne!(male.identity.pitch_mean, male2.identity.pitch_mean, "priors are probabilistic, not fixed");
    }

    // --- M5: heard voices / mimicry pathway ---------------------------------

    #[test]
    fn heard_pattern_merges_similar_voices() {
        let mut v = organ();
        let a = v.hear_pattern("speaker a", low_warm_features(), false, 0.6, 10);
        let a2 = v.hear_pattern("speaker a again", {
            let mut f = low_warm_features();
            f[0] += 0.01; // near-identical
            f
        }, false, 0.6, 20);
        assert_eq!(a, a2, "similar patterns merge into one heard voice");
        assert_eq!(v.heard.len(), 1);
        assert_eq!(v.heard[0].hear_count, 2);
        let b = v.hear_pattern("speaker b", bright_fast_features(), false, 0.6, 30);
        assert_ne!(a, b, "distinct voices separate");
        assert_eq!(v.heard.len(), 2);
    }

    #[test]
    fn mimicry_requires_consent_and_gate() {
        let mut v = organ();
        let hv = v.hear_pattern("singer", low_warm_features(), false, 0.8, 1);
        // No consent: blend refused and counted, plan identical to plain speak.
        let plain = v.speak("hello there", 0.0, 0.3, 0.5, 0.1, 2);
        let refused = v.speak_toward("hello there", 0.0, 0.3, 0.5, 0.1, 3, Some(hv));
        assert_eq!(plain.params.pitch.to_bits(), refused.params.pitch.to_bits(), "no blend without consent");
        assert_eq!(v.memory.refused_mimicry, 1);
        assert_eq!(v.memory.mimicry_uses, 0);
        // Consent yes, gate off: still refused.
        assert!(v.set_consent(hv, true));
        let refused2 = v.speak_toward("hello there", 0.0, 0.3, 0.5, 0.1, 4, Some(hv));
        assert_eq!(plain.params.pitch.to_bits(), refused2.params.pitch.to_bits(), "gate still off");
        assert_eq!(v.memory.refused_mimicry, 2);
        // Gate on: blend happens and is counted.
        v.set_learning_enabled(true);
        let blended = v.speak_toward("hello there", 0.0, 0.3, 0.5, 0.1, 5, Some(hv));
        assert!(blended.params.pitch < plain.params.pitch - 0.01, "low-pitch voice pulls pitch down");
        assert_eq!(v.memory.mimicry_uses, 1);
        assert_eq!(v.heard[0].learnable_uses, 1);
        assert!(blended.params.brightness < plain.params.brightness, "warm voice dims brightness");
    }

    #[test]
    fn mimicry_drift_is_gradual_and_bounded() {
        let mut v = organ();
        let start = v.identity.pitch_mean;
        let hv = v.hear_pattern("singer", low_warm_features(), true, 0.8, 1);
        v.set_learning_enabled(true);
        let mut prev = start;
        for i in 0..200 {
            v.speak_toward("a phrase", 0.0, 0.3, 0.5, 0.1, 10 + i, Some(hv));
            let now = v.identity.pitch_mean;
            assert!(now <= prev + 1e-6, "drift is monotone toward the target");
            prev = now;
        }
        assert!(prev < start, "low target pulled the identity down");
        assert!((0.0..=1.0).contains(&prev), "identity stays bounded");
        assert!(prev > 0.25, "drift is bounded — does not collapse onto the target");
        assert_eq!(v.memory.mimicry_uses, 200);
    }

    #[test]
    fn heard_targets_mapping_is_deterministic() {
        let a = heard_targets(&low_warm_features());
        let b = heard_targets(&low_warm_features());
        assert_eq!(a.pitch.to_bits(), b.pitch.to_bits());
        assert!(a.pitch < 0.5, "low pitch feature maps low");
        let c = heard_targets(&bright_fast_features());
        assert!(c.brightness > a.brightness, "zcr maps brightness");
        assert!(c.rate_mult > a.rate_mult, "faster seg rate maps faster");
        // Short/empty vectors degrade to defaults, never panic.
        let d = heard_targets(&[]);
        assert_eq!(d.pitch, 0.5);
        assert_eq!(d.crispness, 0.6);
    }

    #[test]
    fn heard_voices_decay_without_reinforcement() {
        let mut v = organ();
        v.hear_pattern("fading", low_warm_features(), true, 1.0, 1);
        assert_eq!(v.heard.len(), 1);
        // ~4000 sim-seconds of idle (dt is clamped to 1.0 per call) →
        // salience decays below the 0.04 floor and the voice is forgotten.
        for _ in 0..4000 {
            v.step_idle(10.0, 0.3, 0.2);
        }
        assert!(v.heard.is_empty(), "unreinforced heard voice is forgotten");
        // Reinforced voices persist.
        let hv2 = v.hear_pattern("steady", low_warm_features(), true, 0.9, 2);
        v.step_idle(1000.0, 0.3, 0.2);
        assert!(v.heard.iter().any(|h| h.id == hv2), "recently heard voice persists");
    }
}
