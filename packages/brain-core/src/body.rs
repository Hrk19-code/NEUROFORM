//! Body organ — sensory embodiment (DESIGN.md §5, §6; master prompt Tab 5).
//!
//! M6 core (headless): a persistent Body Schema with per-channel calibration
//! and explicit unavailable senses, receptor-class touch decomposition
//! (§5.1), vestibular/motion ingestion with canal/otolith analogues (§5.2),
//! posture estimation, interoception from telemetry (§5.4), the eight-step
//! novel-sense integration sequence (§6.2), and **dormant motor hooks** —
//! actuator placeholders exist in the schema but `motor_enabled` is always
//! false and no motor code path exists (verified by test).
//!
//! Mechanisms only, no scripted reactions: the channel decomposition,
//! affective priors, calibration dynamics and integration sequence are fixed
//! machinery; what the file *feels* about a touch or a new sense emerges from
//! its state (temperament, mood, embodiment gains, prior history). Everything
//! is deterministic — ingestion is a pure function of the event + state.
//!
//! Touch memory stores compressed channel features only (never raw sensor
//! data), salience-tagged and decayable — the file remembers how it is
//! usually touched, and "different today" is a prediction error (§4.11).

use serde::{Deserialize, Serialize};

use crate::drawing::features_cosine;

// --- channels, permissions, calibration --------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelKind {
    Touch,
    Motion,
    Orientation,
    Vision,
    Audition,
    Interoception,
    Ui,
}

impl ChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelKind::Touch => "touch",
            ChannelKind::Motion => "motion",
            ChannelKind::Orientation => "orientation",
            ChannelKind::Vision => "vision",
            ChannelKind::Audition => "audition",
            ChannelKind::Interoception => "interoception",
            ChannelKind::Ui => "ui",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Granted,
    Denied,
    Degraded,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CalibrationState {
    Uncalibrated,
    Calibrating,
    Calibrated,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Calibration {
    pub state: CalibrationState,
    pub confidence: f32, // 0..1
    pub error_rate: f32, // 0..1 (observed outliers / samples)
    #[serde(default)]
    pub samples: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorChannel {
    pub kind: ChannelKind,
    pub permission: Permission,
    pub calibration: Calibration,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Posture {
    Still,
    Upright,
    Lying,
    Moving,
    Transport,
}

// --- body schema (§18.7) -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Actuator {
    pub joint_id: String,
    /// Always false in the MVP — dormant motor hook (§5.3, §15.6). A test
    /// asserts no actuator can ever be enabled.
    pub motor_enabled: bool,
    pub state: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodySchema {
    pub profile: String, // "device" | "custom" | "robot-placeholder"
    pub available: Vec<SensorChannel>,
    /// Explicitly listed — the file knows it cannot see if no permission
    /// (absence is modeled, not ignored, §6.1).
    pub unavailable: Vec<(ChannelKind, String)>, // (channel, reason)
    /// Normalized touch map, 4×4 grid (0..1 intensity per cell).
    pub touch_map: Vec<f32>,
    pub extent: f32, // body extent estimate (normalized 0..1)
    pub gravity: [f32; 3],
    pub tilt: f32, // radians, 0 = upright
    pub posture: Posture,
    pub rotational: [f32; 3],
    pub linear: [f32; 3],
    pub ownership_confidence: f32,    // 0..1
    pub calibration_confidence: f32,  // 0..1 (aggregate)
    pub actuators: Vec<Actuator>,
    /// Cortex map (§4.9/§3.5): organ → region attachments + activations.
    pub cortex: Vec<CortexRegion>,
}

impl BodySchema {
    pub fn device() -> Self {
        let channel = |kind, perm, state, conf, err| SensorChannel {
            kind,
            permission: perm,
            calibration: Calibration { state, confidence: conf, error_rate: err, samples: 0 },
        };
        // Device body: touch/motion/orientation/interoception/ui present but
        // uncalibrated; vision/audition explicitly absent (no permission).
        let available = vec![
            channel(ChannelKind::Touch, Permission::Granted, CalibrationState::Uncalibrated, 0.0, 0.0),
            channel(ChannelKind::Motion, Permission::Granted, CalibrationState::Uncalibrated, 0.0, 0.0),
            channel(ChannelKind::Orientation, Permission::Granted, CalibrationState::Uncalibrated, 0.0, 0.0),
            channel(ChannelKind::Interoception, Permission::Granted, CalibrationState::Uncalibrated, 0.0, 0.0),
            channel(ChannelKind::Ui, Permission::Granted, CalibrationState::Calibrated, 1.0, 0.0),
        ];
        let unavailable = vec![
            (ChannelKind::Vision, "no-permission".to_string()),
            (ChannelKind::Audition, "no-permission".to_string()),
        ];
        // Dormant future robot joints — motor_enabled: false, state: None.
        let actuators = vec![
            Actuator { joint_id: "head_yaw".into(), motor_enabled: false, state: None },
            Actuator { joint_id: "torso_pitch".into(), motor_enabled: false, state: None },
            Actuator { joint_id: "arm_l_shoulder".into(), motor_enabled: false, state: None },
            Actuator { joint_id: "arm_r_shoulder".into(), motor_enabled: false, state: None },
        ];
        BodySchema {
            profile: "device".into(),
            available,
            unavailable,
            touch_map: vec![0.0; 16],
            extent: 0.3,
            gravity: [0.0, 0.0, -1.0],
            tilt: 0.0,
            posture: Posture::Upright,
            rotational: [0.0; 3],
            linear: [0.0; 3],
            ownership_confidence: 0.5,
            calibration_confidence: 0.0,
            actuators,
            cortex: BodySchema::cortex_regions(),
        }
    }

    pub fn channel_mut(&mut self, kind: ChannelKind) -> Option<&mut SensorChannel> {
        self.available.iter_mut().find(|c| c.kind == kind)
    }

    pub fn channel(&self, kind: ChannelKind) -> Option<&SensorChannel> {
        self.available.iter().find(|c| c.kind == kind)
    }
}

// --- touch: receptor decomposition + affective priors (§5.1) -----------------

/// Receptor-class decomposition of one contact event (pure, deterministic).
/// Order: [fa, sa, sa2, fa1, sa2b] — matches the touch-memory feature layout.
pub fn decompose_touch(pressure: f32, velocity: f32, area: f32, duration_ms: f32, contacts: f32) -> [f32; 5] {
    let p = pressure.clamp(0.0, 1.0);
    let v = velocity.clamp(0.0, 1.0);
    let a = area.clamp(0.0, 1.0);
    let d = (duration_ms / 2000.0).clamp(0.0, 1.0); // 0..1 over 2 s
    let c = contacts.clamp(0.0, 8.0) / 8.0;
    [
        // FA-like: fast-adapting vibration — responds to onset/offset
        // (high-pass: velocity × (1 - duration) → transient energy).
        (v * (1.0 - d)).clamp(0.0, 1.0),
        // SA-like: slow-adapting pressure — integrates *sustained* contact
        // (pressure × duration: a brief hard tap is transient, not SA).
        (p * d).clamp(0.0, 1.0),
        // SA2-like: motion/stretch — directional stroking over time.
        (v * d).clamp(0.0, 1.0),
        // FA1-like: fine detail — high-frequency spatial modulation
        // (velocity on a small contact area).
        (v * (1.0 - a)).clamp(0.0, 1.0),
        // SA2b-like: broad contact — whole-hand/grasp aggregation.
        (c * (0.5 + 0.5 * a)).clamp(0.0, 1.0),
    ]
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TouchAffect {
    Soothing,
    Neutral,
    Unpleasant,
    Playful,
    Intrusive,
    Grounding,
    Calming,
    Alerting,
    Intimate,
    Harsh,
    Familiar,
    Unfamiliar,
}

/// Affective interpretation — priors only, deterministic. The classifier
/// (weights) would be learned; these are the initial priors of §5.1.
/// Note: SA is duration-weighted (sustained pressure), so "hard press"
/// for brief contacts reads via FA (transient energy), not SA.
pub fn interpret_touch(d: &[f32; 5], duration_ms: f32) -> TouchAffect {
    let (fa, sa, sa2, _fa1, sa2b) = (d[0], d[1], d[2], d[3], d[4]);
    let slow = duration_ms > 600.0;
    let fast = duration_ms < 250.0;
    if slow && sa > 0.3 && sa2 > 0.3 {
        TouchAffect::Intimate // sustained + stroking
    } else if slow && sa < 0.25 {
        TouchAffect::Soothing // light + sustained
    } else if slow {
        TouchAffect::Grounding // steady + moderate
    } else if fast && fa > 0.6 {
        TouchAffect::Alerting // abrupt transient (onset energy)
    } else if fast && fa > 0.4 {
        TouchAffect::Harsh // hard + quick
    } else if fast && d[3] > 0.5 {
        TouchAffect::Playful // quick fine detail
    } else if sa > 0.5 {
        TouchAffect::Harsh // sustained hard press
    } else if sa2b > 0.6 && sa > 0.4 {
        TouchAffect::Grounding // broad embrace-like
    } else {
        TouchAffect::Neutral
    }
}

/// Affective prior → (valence, arousal) guess (bounded, deterministic).
pub fn affect_guess(a: TouchAffect) -> (f32, f32) {
    match a {
        TouchAffect::Soothing => (0.35, -0.25),
        TouchAffect::Calming => (0.25, -0.2),
        TouchAffect::Grounding => (0.2, -0.1),
        TouchAffect::Intimate => (0.45, 0.15),
        TouchAffect::Playful => (0.3, 0.3),
        TouchAffect::Familiar => (0.15, -0.05),
        TouchAffect::Neutral => (0.0, 0.0),
        TouchAffect::Unfamiliar => (0.0, 0.15),
        TouchAffect::Unpleasant => (-0.3, 0.2),
        TouchAffect::Intrusive => (-0.25, 0.3),
        TouchAffect::Alerting => (-0.1, 0.45),
        TouchAffect::Harsh => (-0.4, 0.5),
    }
}

// --- touch memory (compressed, decayable) ------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TouchMemoryEntry {
    pub id: u64,
    /// 5 dims: [fa, sa, sa2, fa1, sa2b] — compressed summary, no raw data.
    pub features: [f32; 5],
    pub salience: f32,
    pub count: u32,
    pub first_tick: u64,
    pub last_tick: u64,
}

// --- interoception (§5.4) -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct InteroceptiveState {
    pub energy_load: f32,        // battery/uptime analogue
    pub processing_pressure: f32, // cpu/memory of the app
    pub memory_pressure: f32,    // capacity ledger fullness
    pub session_minutes: f32,    // elapsed interaction
    pub interaction_load: f32,   // event rate / peer traffic
}

impl Default for InteroceptiveState {
    fn default() -> Self {
        InteroceptiveState { energy_load: 0.2, processing_pressure: 0.2, memory_pressure: 0.1, session_minutes: 0.0, interaction_load: 0.1 }
    }
}

impl InteroceptiveState {
    /// Aggregate interoceptive load (0..1) from the telemetry analogues.
    pub fn load(&self) -> f32 {
        (0.3 * self.energy_load + 0.25 * self.processing_pressure + 0.2 * self.memory_pressure
            + 0.15 * (self.session_minutes / 240.0).min(1.0) + 0.1 * self.interaction_load)
            .clamp(0.0, 1.0)
    }
}

// --- novel-sense integration state (§6.2) ------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NovelIntegration {
    pub channel: ChannelKind,
    /// 1 detection, 2 prediction-error, 3 salience-tagging, 4 calibration,
    /// 5 schema-expansion, 6 memory-formation, 7 sleep-integration,
    /// 8 long-term adaptation.
    pub stage: u8,
    pub started_tick: u64,
}

// --- cortex map (sensory cortex analogues, DESIGN.md §4.9 / §3.5) ------------
//
// Organs attach where they'd attach in a real brain: the anatomy table is
// fixed structure (authored — anatomy is structure); what emerges is the
// activation dynamics (which regions light up with experience, decay when
// idle, and integrate during sleep).

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CortexRegion {
    pub region: String,      // e.g. "visual", "auditory", "somatosensory"
    pub activation: f32,     // 0..1, fed by its channel(s), decays idle
    pub last_tick: u64,
}

/// Channel → cortical region attachment (the "where eyes go" table).
pub fn channel_region(kind: ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Vision => "visual",
        ChannelKind::Audition => "auditory",
        ChannelKind::Touch => "somatosensory",
        ChannelKind::Motion | ChannelKind::Orientation => "vestibular",
        ChannelKind::Interoception => "interoceptive",
        ChannelKind::Ui => "visual",
    }
}

impl BodySchema {
    pub fn cortex_regions() -> Vec<CortexRegion> {
        // The standard attachment map; prefrontal/motor exist dormant until
        // their organs arrive (PFC milestone / motor never enabled).
        [
            "visual", "auditory", "somatosensory", "vestibular", "interoceptive",
            "language", "motor", "prefrontal", "parietal",
        ]
        .iter()
        .map(|r| CortexRegion { region: r.to_string(), activation: 0.0, last_tick: 0 })
        .collect()
    }

    /// Raise a region's activation from its channel's activity (bounded).
    pub fn note_activity(&mut self, kind: ChannelKind, amount: f32, tick: u64) {
        let region = channel_region(kind);
        if let Some(r) = self.cortex.iter_mut().find(|r| r.region == region) {
            r.activation = (r.activation + amount).clamp(0.0, 1.0);
            r.last_tick = tick;
        }
    }

    /// Feed a non-sensory region directly (e.g. language from teacher use).
    pub fn note_region_activity(&mut self, region: &str, amount: f32, tick: u64) {
        if let Some(r) = self.cortex.iter_mut().find(|r| r.region == region) {
            r.activation = (r.activation + amount).clamp(0.0, 1.0);
            r.last_tick = tick;
        }
    }

    pub fn region_activation(&self, region: &str) -> f32 {
        self.cortex.iter().find(|r| r.region == region).map(|r| r.activation).unwrap_or(0.0)
    }
}

// --- the organ ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyOrgan {
    pub schema: BodySchema,
    pub touch_memory: Vec<TouchMemoryEntry>,
    pub next_touch_id: u64,
    pub novel: Option<NovelIntegration>,
    pub integrations_done: u32,
    pub intero: InteroceptiveState,
    /// Bounded sensory history (last events, inspectable).
    pub history: Vec<BodyLogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyLogEntry {
    pub tick: u64,
    pub kind: String, // "touch" | "motion" | "interoception" | "novel"
    pub summary: String,
}

pub struct TouchPercept {
    pub affect: TouchAffect,
    pub familiarity: f32, // best cosine vs touch memory (0..1)
    pub features: [f32; 5],
}

pub struct MotionPercept {
    pub posture: Posture,
    pub abruptness: f32,
    pub rhythmicity: f32,
}

pub struct InteroPercept {
    pub load: f32, // aggregate interoceptive load 0..1
}

impl BodyOrgan {
    pub fn new() -> Self {
        BodyOrgan {
            schema: BodySchema::device(),
            touch_memory: Vec::new(),
            next_touch_id: 1,
            novel: None,
            integrations_done: 0,
            intero: InteroceptiveState::default(),
            history: Vec::new(),
        }
    }

    // --- idle dynamics ----------------------------------------------------

    pub fn step_idle(&mut self, dt_ticks: f32, _arousal: f32, _fatigue: f32) {
        let dt = (dt_ticks * 0.1).min(1.0);
        // Touch memory decays without reinforcement (non-permanence).
        let keep: Vec<TouchMemoryEntry> = self
            .touch_memory
            .drain(..)
            .filter(|m| m.salience * (1.0 - dt * 0.0005) >= 0.05)
            .map(|mut m| {
                m.salience *= 1.0 - dt * 0.0005;
                m
            })
            .collect();
        self.touch_memory = keep;
        // Ownership confidence drifts toward baseline; calibration
        // confidence decays slowly when idle (sensor drift, §6.1).
        self.schema.ownership_confidence =
            (self.schema.ownership_confidence + (0.5 - self.schema.ownership_confidence) * dt * 0.001).clamp(0.0, 1.0);
        self.schema.calibration_confidence =
            (self.schema.calibration_confidence - dt * 0.0002).clamp(0.0, 1.0);
        // Touch map relaxes.
        for v in self.schema.touch_map.iter_mut() {
            *v *= 1.0 - dt * 0.01;
        }
        // Cortex activations decay toward baseline (regions quiet when idle).
        // dt is already sim-seconds (0.01/tick); 0.1/s ≈ ~10s time constant.
        for r in self.schema.cortex.iter_mut() {
            r.activation = (r.activation * (1.0 - dt * 0.1)).max(0.0);
        }
    }

    // --- touch ingestion (§5.1) -------------------------------------------

    pub fn ingest_touch(&mut self, pressure: f32, velocity: f32, area: f32, duration_ms: f32, contacts: f32, tick: u64) -> TouchPercept {
        let d = decompose_touch(pressure, velocity, area, duration_ms, contacts);
        let mut affect = interpret_touch(&d, duration_ms);
        // Familiarity: compare against compressed touch memory (cosine).
        // Familiar touches are remembered as familiar (prediction match);
        // novel patterns carry a soft "different today" signal.
        let mut familiarity = 0.0f32;
        if let Some((idx, c)) = self
            .touch_memory
            .iter()
            .enumerate()
            .map(|(i, m)| (i, features_cosine(&m.features, &d)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            familiarity = c;
            if c >= 0.8 {
                let m = &mut self.touch_memory[idx];
                m.count += 1;
                m.salience = (m.salience + 0.15).min(1.0);
                m.last_tick = tick;
                if affect == TouchAffect::Neutral {
                    affect = TouchAffect::Familiar;
                }
            } else if c < 0.4 && affect == TouchAffect::Neutral {
                // "Different today" is a prediction error — but only relabels
                // weak signals; strong affective priors (Alerting, Harsh…)
                // are never masked by unfamiliarity (§5.1 soft labels).
                affect = TouchAffect::Unfamiliar;
            }
        }
        if familiarity < 0.8 {
            self.touch_memory.push(TouchMemoryEntry {
                id: self.next_touch_id,
                features: d,
                salience: 0.5,
                count: 1,
                first_tick: tick,
                last_tick: tick,
            });
            self.next_touch_id += 1;
        }
        // Touch map: activate center cells, gentle spatial blur.
        let map = &mut self.schema.touch_map;
        let intensity = (0.3 + 0.7 * d[1]).min(1.0);
        for (i, v) in map.iter_mut().enumerate() {
            let r = (i / 4) as f32;
            let c = (i % 4) as f32;
            let dist = ((r - 1.5).powi(2) + (c - 1.5).powi(2)).sqrt();
            *v = (*v + intensity * (-dist * 0.8).exp()).min(1.0);
        }
        self.schema.extent = (self.schema.extent + d[4] * 0.01).min(1.0);
        self.schema.ownership_confidence =
            (self.schema.ownership_confidence + 0.005).clamp(0.0, 1.0); // touch binds the body
        self.schema.note_activity(ChannelKind::Touch, 0.3, tick); // somatosensory lights up
        self.log(tick, "touch", format!("{:?} (familiarity {:.2})", affect, familiarity));
        TouchPercept { affect, familiarity, features: d }
    }

    // --- motion ingestion (§5.2) ------------------------------------------

    pub fn ingest_motion(&mut self, linear: [f32; 3], rotational: [f32; 3], tick: u64) -> MotionPercept {
        // Canal-like: rotational high-pass → abruptness (rotation onset).
        let rot_mag = (rotational[0].powi(2) + rotational[1].powi(2) + rotational[2].powi(2)).sqrt();
        // Otolith-like: gravity estimate from linear (low-pass proxy).
        let lin_mag = (linear[0].powi(2) + linear[1].powi(2) + linear[2].powi(2)).sqrt();
        // Posture from gravity direction (device = body). Check excess
        // linear energy (transport/moving) before gravity alignment so a
        // resting upright device (gravity magnitude ≈ 1) isn't "moving".
        let gz = linear[2].clamp(-1.0, 1.0);
        let posture = if lin_mag > 1.6 {
            Posture::Transport // sustained high linear energy
        } else if rot_mag > 1.2 || (lin_mag > 1.0 && gz.abs() < 0.7) {
            Posture::Moving // rotation onset or off-gravity acceleration
        } else if gz > 0.6 {
            Posture::Lying // gravity along +Z → device on its back
        } else if gz < -0.4 {
            Posture::Upright
        } else {
            Posture::Still
        };
        self.schema.gravity = [linear[0] * 0.1, linear[1] * 0.1, gz];
        self.schema.tilt = gz.abs().acos().min(std::f32::consts::FRAC_PI_2);
        self.schema.posture = posture;
        self.schema.rotational = rotational;
        self.schema.linear = linear;
        self.schema.note_activity(ChannelKind::Motion, 0.25, tick); // vestibular lights up
        // Rhythmicity proxy: rotational sign-change rate over the last few
        // events — deterministic placeholder using current magnitudes.
        let rhythmicity = (1.0 - (rot_mag - 0.2).abs().min(1.0)).clamp(0.0, 1.0);
        self.log(tick, "motion", format!("{posture:?} rot {rot_mag:.2} lin {lin_mag:.2}"));
        MotionPercept { posture, abruptness: rot_mag.clamp(0.0, 1.0), rhythmicity }
    }

    // --- interoception ingestion (§5.4) ------------------------------------

    pub fn ingest_interoception(&mut self, energy_load: f32, processing_pressure: f32, memory_pressure: f32, session_minutes: f32, interaction_load: f32, tick: u64) -> InteroPercept {
        self.intero = InteroceptiveState {
            energy_load: energy_load.clamp(0.0, 1.0),
            processing_pressure: processing_pressure.clamp(0.0, 1.0),
            memory_pressure: memory_pressure.clamp(0.0, 1.0),
            session_minutes: session_minutes.max(0.0),
            interaction_load: interaction_load.clamp(0.0, 1.0),
        };
        let s = &self.intero;
        let load = s.load();
        self.schema.note_activity(ChannelKind::Interoception, 0.2 * load, tick); // interoceptive region
        self.log(tick, "interoception", format!("load {load:.2}"));
        InteroPercept { load }
    }

    // --- novel-sense integration (§6.2, 8 steps) --------------------------

    /// Step 1+5: detect a newly available channel and expand the schema.
    /// Returns false if already present. The reaction (what the file feels)
    /// is not scripted — the sequence is machinery only.
    pub fn attach_novel_channel(&mut self, kind: ChannelKind, tick: u64) -> bool {
        if self.schema.channel(kind).is_some() {
            return false;
        }
        self.schema.available.push(SensorChannel {
            kind,
            permission: Permission::Granted,
            calibration: Calibration { state: CalibrationState::Calibrating, confidence: 0.0, error_rate: 0.0, samples: 0 },
        });
        // Remove from unavailable if listed.
        self.schema.unavailable.retain(|(k, _)| *k != kind);
        self.novel = Some(NovelIntegration { channel: kind, stage: 2, started_tick: tick });
        self.log(tick, "novel", format!("channel {} detected → calibrating", kind.as_str()));
        true
    }

    /// Steps 3–5: calibration samples — passive statistics; confidence
    /// rises as distributions stabilize (deterministic).
    pub fn calibration_sample(&mut self, kind: ChannelKind, outlier: bool, tick: u64) -> f32 {
        let (confidence, samples, became_calibrated) = {
            let Some(ch) = self.schema.channel_mut(kind) else { return 0.0 };
            ch.calibration.samples += 1;
            // Running mean over ALL samples (outliers dilute with time).
            let n = ch.calibration.samples as f32;
            ch.calibration.error_rate =
                (ch.calibration.error_rate * (n - 1.0) + if outlier { 1.0 } else { 0.0 }) / n;
            // Confidence: 1 - 1/(1+n/25); calibrated at ≥ 0.85 (≈ 142 samples).
            ch.calibration.confidence = (1.0 - 1.0 / (1.0 + n / 25.0)).clamp(0.0, 1.0);
            let became = ch.calibration.confidence >= 0.85 && ch.calibration.state == CalibrationState::Calibrating;
            if became {
                ch.calibration.state = CalibrationState::Calibrated;
            }
            (ch.calibration.confidence, ch.calibration.samples, became)
        };
        if became_calibrated {
            if let Some(nv) = self.novel.as_mut() {
                nv.stage = 6; // memory-formation
            }
            self.log(tick, "novel", format!("channel {} calibrated ({samples} samples)", kind.as_str()));
        }
        // Aggregate calibration confidence across channels.
        let avail = &self.schema.available;
        let sum: f32 = avail.iter().map(|c| c.calibration.confidence).sum();
        self.schema.calibration_confidence = sum / avail.len().max(1) as f32;
        confidence
    }

    /// Step 7: sleep-based integration — finalizes calibrating channels,
    /// raises ownership confidence, counts the integration.
    pub fn sleep_integration(&mut self) -> u32 {
        let mut finalized = 0;
        for ch in self.schema.available.iter_mut() {
            if ch.calibration.state == CalibrationState::Calibrating && ch.calibration.confidence >= 0.5 {
                ch.calibration.state = CalibrationState::Calibrated;
                finalized += 1;
            }
        }
        if let Some(nv) = self.novel.as_mut() {
            nv.stage = 8; // long-term adaptation
        }
        self.schema.ownership_confidence = (self.schema.ownership_confidence + 0.1).clamp(0.0, 1.0);
        self.integrations_done += 1;
        self.log(self.schema.available.len() as u64, "novel", format!("sleep integration: {finalized} channel(s) finalized"));
        finalized
    }

    /// Motor hooks are dormant: no actuator is ever enabled in the MVP.
    pub fn motor_enabled_count(&self) -> usize {
        self.schema.actuators.iter().filter(|a| a.motor_enabled).count()
    }

    fn log(&mut self, tick: u64, kind: &str, summary: String) {
        self.history.push(BodyLogEntry { tick, kind: kind.into(), summary });
        if self.history.len() > 64 {
            self.history.remove(0);
        }
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xB0D7_5EED_0000_0001;
        let mut mix = |x: u64| h = (h ^ x).wrapping_mul(0x0000_0100_0000_01b3);
        for v in [
            self.schema.ownership_confidence,
            self.schema.calibration_confidence,
            self.schema.extent,
            self.schema.tilt,
        ] {
            for b in v.to_bits().to_le_bytes() {
                mix(b as u64);
            }
        }
        mix(self.schema.posture as u64);
        for ch in self.schema.available.iter() {
            mix(ch.kind as u64);
            mix(ch.permission as u64);
            for b in ch.calibration.confidence.to_bits().to_le_bytes() {
                mix(b as u64);
            }
            mix(ch.calibration.samples as u64);
        }
        for m in self.touch_memory.iter() {
            mix(m.id);
            mix(m.salience.to_bits() as u64);
            mix(m.count as u64);
            for f in m.features.iter() {
                mix(f.to_bits() as u64);
            }
        }
        for r in self.schema.cortex.iter() {
            mix(r.activation.to_bits() as u64);
        }
        if let Some(nv) = &self.novel {
            mix(nv.channel as u64);
            mix(nv.stage as u64);
        }
        mix(self.integrations_done as u64);
        h
    }
}

impl Default for BodyOrgan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn organ() -> BodyOrgan {
        BodyOrgan::new()
    }

    #[test]
    fn touch_decomposition_is_deterministic_and_separates_events() {
        let gentle = decompose_touch(0.2, 0.1, 0.3, 1500.0, 1.0);
        let abrupt = decompose_touch(0.9, 0.9, 0.6, 80.0, 1.0);
        assert_eq!(gentle, decompose_touch(0.2, 0.1, 0.3, 1500.0, 1.0), "deterministic");
        // Gentle sustained: SA-like pressure integrates high, FA low.
        assert!(gentle[1] > abrupt[1], "sustained pressure → SA channel high");
        assert!(abrupt[0] > gentle[0], "fast transient → FA channel high");
        assert!(abrupt[3] > gentle[3], "fast small contact → FA1 detail high");
    }

    #[test]
    fn affective_priors_map_gentle_to_soothing_abrupt_to_alerting() {
        let d1 = decompose_touch(0.2, 0.1, 0.3, 1500.0, 1.0);
        assert_eq!(interpret_touch(&d1, 1500.0), TouchAffect::Soothing);
        let d2 = decompose_touch(0.9, 0.9, 0.6, 80.0, 1.0);
        assert_eq!(interpret_touch(&d2, 80.0), TouchAffect::Alerting);
        // Affect priors map to bounded, opposite arousal guesses.
        let (_, a1) = affect_guess(TouchAffect::Soothing);
        let (_, a2) = affect_guess(TouchAffect::Alerting);
        assert!(a2 > a1, "alerting raises arousal more than soothing");
    }

    #[test]
    fn repeated_touch_becomes_familiar_novel_stays_unfamiliar() {
        let mut b = organ();
        let p1 = b.ingest_touch(0.2, 0.1, 0.3, 1500.0, 1.0, 10);
        assert_eq!(p1.affect, TouchAffect::Soothing);
        assert!(p1.familiarity < 0.8, "first touch is not familiar");
        let p2 = b.ingest_touch(0.2, 0.1, 0.3, 1500.0, 1.0, 20);
        assert!(p2.familiarity >= 0.8, "same touch again is familiar: {}", p2.familiarity);
        let p3 = b.ingest_touch(0.9, 0.9, 0.6, 80.0, 1.0, 30);
        assert!(p3.familiarity < 0.4, "different touch is unfamiliar");
        assert_eq!(p3.affect, TouchAffect::Alerting, "strong prior not masked by unfamiliarity");
        assert_eq!(b.touch_memory.len(), 2, "two distinct patterns remembered");
    }

    #[test]
    fn touch_memory_decays_without_reinforcement() {
        let mut b = organ();
        b.ingest_touch(0.2, 0.1, 0.3, 1500.0, 1.0, 10);
        assert_eq!(b.touch_memory.len(), 1);
        for _ in 0..20_000 {
            b.step_idle(10.0, 0.3, 0.2);
        }
        assert!(b.touch_memory.is_empty(), "unreinforced touch patterns are forgotten");
    }

    #[test]
    fn motion_posture_and_abruptness() {
        let mut b = organ();
        let upright = b.ingest_motion([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 1);
        assert_eq!(upright.posture, Posture::Upright);
        let lying = b.ingest_motion([0.0, 0.0, 0.9], [0.0, 0.0, 0.0], 2);
        assert_eq!(lying.posture, Posture::Lying);
        let moving = b.ingest_motion([0.5, 0.5, -0.5], [0.0, 0.0, 1.5], 3);
        assert_eq!(moving.posture, Posture::Moving);
        assert!(moving.abruptness > upright.abruptness, "rotation onset → abrupt");
        assert_eq!(b.schema.posture, Posture::Moving, "schema tracks posture");
    }

    #[test]
    fn interoception_aggregates_load() {
        let mut b = organ();
        let low = b.ingest_interoception(0.1, 0.1, 0.1, 5.0, 0.1, 1);
        let high = b.ingest_interoception(0.9, 0.9, 0.8, 300.0, 0.9, 2);
        assert!(high.load > low.load, "high telemetry → high interoceptive load");
        assert!(high.load <= 1.0 && low.load >= 0.0);
    }

    #[test]
    fn calibration_confidence_tracks_samples_and_reaches_calibrated() {
        let mut b = organ();
        assert!(b.attach_novel_channel(ChannelKind::Vision, 1), "novel channel detected");
        let ch = b.schema.channel(ChannelKind::Vision).unwrap();
        assert_eq!(ch.calibration.state, CalibrationState::Calibrating);
        let mut conf = 0.0;
        for i in 0..300 {
            conf = b.calibration_sample(ChannelKind::Vision, i % 50 == 0, 10 + i);
        }
        assert!(conf >= 0.85, "confidence tracks calibration samples: {conf}");
        let ch = b.schema.channel(ChannelKind::Vision).unwrap();
        assert_eq!(ch.calibration.state, CalibrationState::Calibrated);
        assert!(ch.calibration.error_rate > 0.0, "outliers recorded");
        assert_eq!(b.novel.as_ref().map(|n| n.stage), Some(6), "stage → memory-formation");
    }

    #[test]
    fn novel_channel_integration_sequence() {
        let mut b = organ();
        assert!(b.attach_novel_channel(ChannelKind::Vision, 1));
        assert!(!b.attach_novel_channel(ChannelKind::Vision, 2), "already present");
        assert!(b.schema.unavailable.iter().all(|(k, _)| *k != ChannelKind::Vision), "removed from unavailable");
        for i in 0..300 {
            b.calibration_sample(ChannelKind::Vision, false, 10 + i);
        }
        let finalized = b.sleep_integration();
        assert_eq!(finalized, 0, "already calibrated — nothing to finalize");
        assert_eq!(b.integrations_done, 1);
        assert_eq!(b.novel.as_ref().map(|n| n.stage), Some(8), "long-term adaptation");
        assert!(b.schema.ownership_confidence > 0.5, "ownership rises with integration");
    }

    #[test]
    fn motor_hooks_are_dormant() {
        let b = organ();
        assert_eq!(b.motor_enabled_count(), 0, "no actuator ever enabled");
        assert!(b.schema.actuators.iter().all(|a| !a.motor_enabled && a.state.is_none()));
    }

    #[test]
    fn body_digest_is_deterministic_and_state_sensitive() {
        let mut a = organ();
        let b = organ();
        assert_eq!(a.digest(), b.digest());
        a.ingest_touch(0.2, 0.1, 0.3, 1500.0, 1.0, 1);
        a.ingest_motion([0.0, 0.0, -1.0], [0.0, 0.5, 0.0], 2);
        a.ingest_interoception(0.5, 0.5, 0.5, 60.0, 0.4, 3);
        assert_ne!(a.digest(), b.digest(), "sensory history changes the digest");
    }
}
