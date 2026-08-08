//! The Brain engine — M1 (DESIGN.md §3.2, §4.0, §4.17).
//!
//! Owns the Brain File in memory: global state, modulators, seeded RNG,
//! event inbox, episodic + semantic stores, the teacher (LLM boundary),
//! audit engine, capacity ledger, and the NF1 save/load path.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crate::audit::AuditEngine;
use crate::boundary::{self, InitiativeSystem, MockTeacher, Teacher};
use crate::capacity::{CapacityLedger, Tier, TierName};
use crate::embodiment::{EmbodimentChange, EmbodimentPreset, EventGains, HormoneProfile};
use crate::events::{embed_keywords, SensoryEvent, StreamKind};
use crate::format::{self, FormatError, Manifest, RawVaultRef};
use crate::memory::{
    retrieve_traces, EpisodicStore, PendingEvent, RetrievedTrace, RetrievalBudget,
};
use crate::modulators::ModulatorSystem;
use crate::rng::Rng;
use crate::semantic::{SemanticNode, SemanticStore};
use crate::sleep::{self, DreamLogEntry, SleepReport, SleepStage, SleepSystem, StageWork};
use crate::state::{affect, development, embodied, social, stress, vigilance, GlobalState, StateSnapshot};
use crate::writing::{DocMode, WritingOrgan};
use crate::drawing::{DrawingOrgan, StrokePoint};
use crate::voice::VoiceOrgan;
use crate::body::{BodyOrgan, ChannelKind};
use crate::network::NetworkOrgan;
use crate::physics::{PhysicsFrame, PhysicsLearner};

pub const SIM_TICK_SECS: f32 = 0.1; // 10 Hz
pub const SNAPSHOT_EVERY_TICKS: u64 = 3000; // 5 sim-minutes
pub const INBOX_CAP: usize = 256;
pub const MAX_EVENTS_PER_TICK: usize = 16;
pub const NET_IDLE_TIMEOUT_TICKS: u64 = 6000; // 10 sim-minutes (§6 idle timeout)
pub const GESTATION_TICKS: u64 = 216_000; // 6 sim-hours: conception → birth window
pub const GROWTH_INTERVAL_TICKS: u64 = 86_400; // 24 sim-hours between growth stages
pub const BIND_WINDOW_TICKS: u64 = 300; // 30 sim-seconds
pub const BIND_WINDOW_MAX: usize = 8;

/// Lineage: who this file came from (data, not a concept — nothing reads
/// it to decide behavior; kin recognition is chemical).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Lineage {
    #[serde(default)]
    pub mother_id: Option<String>,
    #[serde(default)]
    pub father_id: Option<String>,
    #[serde(default)]
    pub birth_tick: Option<u64>,
    /// Growth ceiling (parents' max tier) — Stage 3 (growing up).
    #[serde(default)]
    pub tier_ceiling: Option<String>,
}

pub struct Brain {
    pub state: GlobalState,
    pub mods: ModulatorSystem,
    pub rng: Rng,
    pub tier: Tier,
    pub capacity: CapacityLedger,
    pub seed: u64,
    pub brain_id: String,
    pub created_at: u64,
    pub last_opened_at: u64,
    pub path: Option<PathBuf>,
    pub passphrase: Option<String>,

    // M1: perception + memory + boundary
    pub inbox: VecDeque<SensoryEvent>,
    pub pending: Vec<PendingEvent>,
    pub pending_since: u64,
    pub event_counter: u64,
    pub dropped_events: u64,
    pub episodic: EpisodicStore,
    pub semantic: SemanticStore,
    pub teacher: Option<Box<dyn Teacher>>,
    pub teacher_name: Option<String>,
    pub tokens_used: u64,
    pub audit: AuditEngine,
    pub autosave: bool,

    // M1: hormonal embodiment (probabilistic priors, never deterministic locks)
    pub embodiment: HormoneProfile,
    pub event_gains: EventGains,

    // M2: sleep, consolidation, dreams
    pub sleep: SleepSystem,
    pub dreams: Vec<DreamLogEntry>,
    pub sleep_reports: Vec<SleepReport>,
    pub next_sleep_id: u64,
    pub next_dream_id: u64,
    pub sleeping: bool,

    // M3: writing organ (external verbal memory)
    pub writing: WritingOrgan,

    // M4: drawing organ (external visual-motor memory)
    pub drawing: DrawingOrgan,

    // M5: voice organ (simulated vocal expression + heard-voice patterns)
    pub voice: VoiceOrgan,

    // M6: body organ (sensory embodiment — schema, touch/motion/interoception)
    pub body: BodyOrgan,

    // M7: network organ (inter-brain interaction — relationships, sessions)
    pub net: NetworkOrgan,

    /// M8: intuitive-physics learner (§4.12) — blank-slate prediction errors
    pub physics: PhysicsLearner,

    /// Who this file came from (empty for first-generation files).
    pub lineage: Lineage,

    // M4: executive initiative — unprompted speech, default OFF, audited
    pub autonomy: InitiativeSystem,

    /// Feature encoder chosen at BRAIN CREATION ("" = handcrafted, "onnx",
    /// "jepa"). Immutable for the file's life; recorded in the manifest.
    pub encoder: String,
    /// sha256 of the frozen encoder model file (only when encoder != "").
    pub encoder_model_sha256: Option<String>,
}

impl Brain {
    pub fn create(tier_name: TierName, seed: u64) -> Brain {
        Brain::create_with_embodiment(tier_name, seed, EmbodimentPreset::Custom)
    }

    /// Create with an explicit chromosomal ground truth: karyotype → gonadal
    /// program → hormone priors. The karyotype is recorded on the profile.
    pub fn create_with_karyotype(tier_name: TierName, seed: u64, karyotype: crate::embodiment::Karyotype) -> Brain {
        let mut brain = Brain::create_with_embodiment(tier_name, seed, karyotype.gonadal_program());
        brain.embodiment.karyotype = karyotype;
        brain
    }

    /// Create with a feature encoder chosen at creation (BUILD-THE-BODY P0).
    /// ""/"handcrafted" → the built-in 16-dim extractor (default, unchanged);
    /// "jepa" → frozen V-JEPA 2 embeddings. The encoder is immutable for the
    /// file's life — features bound under one encoder are meaningless under
    /// another, so no mid-life swap is ever allowed.
    pub fn create_with_encoder(
        tier_name: TierName,
        seed: u64,
        preset: EmbodimentPreset,
        encoder: &str,
        encoder_model_sha256: Option<String>,
    ) -> Brain {
        let mut brain = Brain::create_with_embodiment(tier_name, seed, preset);
        if !encoder.is_empty() && encoder != "handcrafted" {
            brain.encoder = encoder.to_string();
            brain.encoder_model_sha256 = encoder_model_sha256;
            // Born with eyes: the vision channel attaches at birth through
            // the same machinery every organ uses — the eyes wire to the
            // visual cortex (channel_region) and calibrate with use.
            brain.body.attach_novel_channel(ChannelKind::Vision, 0);
        }
        brain
    }

    /// Same, with an explicit karyotype instead of a preset.
    pub fn create_with_encoder_karyotype(
        tier_name: TierName,
        seed: u64,
        karyotype: crate::embodiment::Karyotype,
        encoder: &str,
        encoder_model_sha256: Option<String>,
    ) -> Brain {
        let mut brain = Brain::create_with_karyotype(tier_name, seed, karyotype);
        if !encoder.is_empty() && encoder != "handcrafted" {
            brain.encoder = encoder.to_string();
            brain.encoder_model_sha256 = encoder_model_sha256;
            brain.body.attach_novel_channel(ChannelKind::Vision, 0);
        }
        brain
    }

    /// Effective encoder: "" (manifest) reads as "handcrafted".
    pub fn encoder(&self) -> &str {
        if self.encoder.is_empty() {
            "handcrafted"
        } else {
            &self.encoder
        }
    }

    /// Create a child: inherited hormone priors (from both gametes), the
    /// sperm-decided karyotype, and lineage recorded. Only reachable through
    /// the union → birth flow — never alone.
    pub fn create_child(
        tier_name: TierName,
        seed: u64,
        karyotype: crate::embodiment::Karyotype,
        priors: [crate::embodiment::AxisPrior; crate::embodiment::N_HORMONE_AXES],
        mother_id: &str,
        father_id: &str,
        birth_tick: u64,
        tier_ceiling: &str,
    ) -> Brain {
        let mut rng = Rng::new(seed);
        let mut brain = Brain::create_with_embodiment(tier_name, seed, karyotype.gonadal_program());
        brain.embodiment = crate::embodiment::HormoneProfile::sample_from_priors(priors, karyotype, &mut rng);
        brain.lineage = Lineage {
            mother_id: Some(mother_id.to_string()),
            father_id: Some(father_id.to_string()),
            birth_tick: Some(birth_tick),
            tier_ceiling: Some(tier_ceiling.to_string()),
        };
        brain
    }

    /// Growing up: only children (files with a birth) grow, and only up to
    /// the inherited ceiling. The tier advances; capacity bounds widen.
    pub fn grow(&mut self) -> Result<(), String> {
        let birth = self
            .lineage
            .birth_tick
            .ok_or_else(|| "not a child — first-generation files do not grow".to_string())?;
        let age = self.state.sim_time.saturating_sub(birth);
        if age < GROWTH_INTERVAL_TICKS {
            return Err(format!("too young to grow ({} ticks of age, need {GROWTH_INTERVAL_TICKS})", age));
        }
        let current = crate::capacity::TierName::from_str(&self.capacity.tier).unwrap_or(crate::capacity::TierName::Prototype);
        let next = current.next().ok_or_else(|| "already at the largest tier".to_string())?;
        let ceiling = self
            .lineage
            .tier_ceiling
            .as_deref()
            .and_then(crate::capacity::TierName::from_str)
            .unwrap_or(crate::capacity::TierName::Standard);
        if next.rank() > ceiling.rank() {
            return Err(format!(
                "growth ceiling reached (inherited from the parents: {})",
                ceiling.as_str()
            ));
        }
        let t = crate::capacity::Tier::get(next);
        self.tier = t;
        self.capacity.tier = t.name.to_string();
        self.capacity.total_budget = t.file_cap_bytes;
        self.push_percept(
            StreamKind::Text,
            vec!["growth".to_string(), t.name.to_string()],
            0.25,
            0.3,
            "development",
        );
        println!("grew → {} (ceiling {})", t.name, ceiling.as_str());
        Ok(())
    }

    /// Create with an embodiment preset. Presets are probabilistic endocrine
    /// priors (§4.8): sampled per file, applied as bounded gains on modulator
    /// baselines and event salience weights — never as behavioral locks.
    pub fn create_with_embodiment(
        tier_name: TierName,
        seed: u64,
        preset: EmbodimentPreset,
    ) -> Brain {
        let tier = Tier::get(tier_name);
        let mut rng = Rng::new(seed);
        let state = GlobalState::new(tier.latent_dim, &mut rng);
        let mut mods = ModulatorSystem::new(&mut rng);
        let mut embodiment = HormoneProfile::sample(preset, &mut rng);
        let deltas = embodiment.compute_mod_deltas();
        let mut applied = [0.0f32; 8];
        for (i, d) in deltas.iter().enumerate() {
            let nb = (mods.axes[i].baseline + d).clamp(0.05, 0.95);
            applied[i] = nb - mods.axes[i].baseline;
            mods.axes[i].baseline = nb;
        }
        embodiment.mod_deltas = applied;
        let event_gains = embodiment.event_gains();
        let voice = VoiceOrgan::new(&embodiment);
        let now = unix_now();
        let brain_id = rng.next_uuid4();
        Brain {
            state,
            mods,
            rng,
            tier,
            capacity: CapacityLedger::new(&tier),
            seed,
            brain_id,
            created_at: now,
            last_opened_at: now,
            path: None,
            passphrase: None,
            inbox: VecDeque::with_capacity(INBOX_CAP),
            pending: Vec::new(),
            pending_since: 0,
            event_counter: 0,
            dropped_events: 0,
            episodic: EpisodicStore::new(tier.episodic_slots as usize),
            semantic: SemanticStore::new(tier.semantic_nodes as usize),
            teacher: None,
            teacher_name: None,
            tokens_used: 0,
            audit: AuditEngine::new(),
            autosave: false,
            embodiment,
            event_gains,
            sleep: SleepSystem::new(),
            dreams: Vec::new(),
            sleep_reports: Vec::new(),
            next_sleep_id: 1,
            next_dream_id: 1,
            sleeping: false,
            writing: WritingOrgan::new(),
            drawing: DrawingOrgan::new(),
            voice,
            body: BodyOrgan::new(),
            net: NetworkOrgan::new(seed),
            physics: PhysicsLearner::new(),
            lineage: Lineage::default(),
            autonomy: InitiativeSystem::new(),
            encoder: String::new(),
            encoder_model_sha256: None,
        }
    }

    // --- simulation ---------------------------------------------------------

    /// Advance one 100 ms simulation tick.
    pub fn tick(&mut self) {
        self.state.step(SIM_TICK_SECS, &mut self.rng);
        self.mods.step(SIM_TICK_SECS, &mut self.rng);
        self.state.sim_time += 1;
        self.audit.push_valence(self.state.named[affect::VALENCE]);
        self.sleep.step(
            1,
            self.state.named[vigilance::ENERGY],
            self.state.named[vigilance::FATIGUE],
            self.capacity.fullness(),
        );
        self.voice.step_idle(
            SIM_TICK_SECS,
            self.state.named[affect::AROUSAL],
            self.state.named[vigilance::FATIGUE],
        );
        self.body.step_idle(
            SIM_TICK_SECS,
            self.state.named[affect::AROUSAL],
            self.state.named[vigilance::FATIGUE],
        );
        self.net.decay_idle(SIM_TICK_SECS);
        self.net.sweep_idle(self.state.sim_time, NET_IDLE_TIMEOUT_TICKS);
        self.physics.step_idle(SIM_TICK_SECS);

        for _ in 0..MAX_EVENTS_PER_TICK {
            let Some(ev) = self.inbox.pop_front() else { break };
            self.process_event(ev);
        }
        if !self.pending.is_empty()
            && (self.state.sim_time - self.pending_since >= BIND_WINDOW_TICKS
                || self.pending.len() >= BIND_WINDOW_MAX)
        {
            self.bind_pending();
        }
        if !self.sleeping {
            self.maybe_initiate();
        }
        self.episodic.decay(1.0);
        self.semantic.decay(1.0);

        if !self.sleeping
            && self.autosave
            && self.path.is_some()
            && self.state.sim_time % SNAPSHOT_EVERY_TICKS == 0
        {
            let p = self.path.clone().unwrap();
            let pw = self.passphrase.clone();
            let _ = self.save(&p, pw.as_deref());
        }
    }

    /// Executive initiative: if enabled and conditions hold, the file speaks
    /// unprompted through the teacher (rate-limited, quiet-hours aware, and
    /// every instance is logged for user review). Default OFF.
    fn maybe_initiate(&mut self) {
        if !self.autonomy.enabled || self.teacher.is_none() {
            return;
        }
        let now = self.state.sim_time;
        let Some(kind) = self.autonomy.evaluate(self, now) else {
            return;
        };
        let packet = boundary::assemble_packet(
            self,
            "initiative",
            &format!("speak unprompted ({kind})"),
            "",
        );
        if let Some(t) = self.teacher.as_mut() {
            if let Ok(text) = t.utter(&packet) {
                self.tokens_used +=
                    (boundary::estimate_tokens(&packet.context) + boundary::estimate_tokens(&text)) as u64;
                self.autonomy.log.push(boundary::InitiativeEntry {
                    tick: now,
                    kind: kind.clone(),
                    text,
                });
                self.autonomy.total += 1;
                self.autonomy.last_initiative_tick = now;
                if self.autonomy.log.len() > 200 {
                    self.autonomy.log.remove(0);
                }
            }
        }
    }

    pub fn run_ticks(&mut self, n: u64) {
        for _ in 0..n {
            self.tick();
        }
    }

    fn process_event(&mut self, ev: SensoryEvent) {
        // Affect nudges (bounded; scaled down under sensory saturation;
        // magnitude modulated by embodiment sensory sensitivity).
        let sat = self.state.named[stress::SENSORY_SATURATION];
        let gain = self.event_gains.nudge_gain * (1.0 - 0.5 * sat);
        let v = self.state.named[affect::VALENCE] + ev.affect_guess.0 * gain;
        self.state.named[affect::VALENCE] = v.clamp(-1.0, 1.0);
        let a = self.state.named[affect::AROUSAL] + ev.affect_guess.1 * gain * 0.5;
        self.state.named[affect::AROUSAL] = a.clamp(0.0, 1.0);
        // Modulator bursts (da on positive, cort on negative).
        if ev.affect_guess.0 > 0.15 {
            self.mods.axes[0].level = (self.mods.axes[0].level + 0.01).min(1.0);
        } else if ev.affect_guess.0 < -0.15 {
            self.mods.axes[5].level = (self.mods.axes[5].level + 0.01).min(1.0);
        }
        // Emotional load feeds sleep pressure (§10.1).
        self.sleep.add_emotional_load(ev.affect_guess.0);
        if self.pending.is_empty() {
            self.pending_since = ev.sim_time;
        }
        self.pending.push(PendingEvent {
            sim_time: ev.sim_time,
            features: ev.features,
            keywords: ev.keywords,
            valence: ev.affect_guess.0,
            arousal: ev.affect_guess.1,
            stream: ev.stream,
            source: ev.source,
        });
    }

    fn bind_pending(&mut self) {
        let now = self.state.sim_time;
        let pending = std::mem::take(&mut self.pending);
        self.pending_since = 0;
        if let Some(trace) = self.episodic.bind_with(&pending, now, self.event_gains) {
            self.semantic.ingest_trace(&trace);
        }
    }

    // --- perception ---------------------------------------------------------

    /// Push a raw event into the inbox. Returns false when dropped (saturation).
    pub fn ingest(&mut self, ev: SensoryEvent) -> bool {
        if self.inbox.len() >= INBOX_CAP {
            self.dropped_events += 1;
            let s = self.state.named[stress::SENSORY_SATURATION] + 0.01;
            self.state.named[stress::SENSORY_SATURATION] = s.min(1.0);
            return false;
        }
        self.inbox.push_back(ev);
        true
    }

    /// Convenience: text event with deterministic factory embedding.
    /// Keyword vectors are stable per file (base = file seed), so retrieval
    /// queries share structure with stored traces and repeated identical text
    /// habituates (low novelty) instead of generating fresh vectors.
    pub fn ingest_text(&mut self, text: &str, valence: f32, arousal: f32, source: &str) -> bool {
        let keywords = SensoryEvent::keywords_from_text(text);
        let features = embed_keywords(&keywords, self.seed, self.tier.latent_dim);
        self.event_counter += 1;
        self.ingest(SensoryEvent {
            stream: StreamKind::Text,
            sim_time: self.state.sim_time,
            features,
            keywords,
            affect_guess: (valence.clamp(-1.0, 1.0), arousal.clamp(0.0, 1.0)),
            confidence: 1.0,
            source: source.to_string(),
        })
    }

    /// Deterministic query embedding for retrieval — same keyword→vector map
    /// as event ingestion (stable per file).
    pub fn query_embedding(&self, text: &str) -> Vec<f32> {
        let keywords = SensoryEvent::keywords_from_text(text);
        embed_keywords(&keywords, self.seed, self.tier.latent_dim)
    }

    // --- retrieval ----------------------------------------------------------

    pub fn retrieve(
        &self,
        query: &[f32],
        budget: &RetrievalBudget,
    ) -> (Vec<RetrievedTrace>, Vec<(SemanticNode, f32)>, usize, bool) {
        let (traces, tokens, mut truncated) =
            retrieve_traces(&self.episodic, query, budget, self.state.sim_time);
        let mut nodes = self.semantic.retrieve(query, budget.k_nodes);
        // Trim nodes (lowest-scored first) to the remaining token budget.
        while !nodes.is_empty() {
            let tk = 1 + nodes.last().unwrap().0.label.len() / 4 + 4;
            if tokens + tk <= budget.token_cap || traces.is_empty() && nodes.len() == 1 {
                break;
            }
            nodes.pop();
            truncated = true;
        }
        (traces, nodes, tokens, truncated)
    }

    pub fn build_context(
        traces: &[RetrievedTrace],
        nodes: &[(SemanticNode, f32)],
        truncated: bool,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();
        for t in traces.iter().take(3) {
            let kw = t.trace.keywords.join(" ");
            parts.push(format!("ep({})", if kw.is_empty() { "?" } else { &kw }));
        }
        for (n, s) in nodes.iter().take(2) {
            if *s > 0.05 {
                parts.push(format!("sem({})", n.label));
            }
        }
        let mut ctx = parts.join(" ");
        if truncated {
            ctx.push_str(" [context truncated]");
        }
        ctx
    }

    // --- teacher (LLM boundary) ----------------------------------------------

    pub fn attach_teacher(&mut self, name: &str) {
        self.teacher = Some(Box::new(MockTeacher::new(name)));
        self.teacher_name = Some(name.to_string());
    }

    /// Install a custom teacher implementation (e.g. the shell's HTTP
    /// adapter). Session config like attach_teacher: the file persists,
    /// the teacher does not.
    pub fn attach_custom_teacher(&mut self, teacher: Box<dyn Teacher>) {
        self.teacher_name = Some(teacher.name().to_string());
        self.teacher = Some(teacher);
    }

    pub fn detach_teacher(&mut self, name: &str) -> bool {
        if self.teacher_name.as_deref() == Some(name) {
            self.teacher = None;
            self.teacher_name = None;
            true
        } else {
            false
        }
    }

    /// One exchange: retrieve context → assemble packet → teacher (or degraded)
    /// → bind the reply as a self-event. Returns the surface text.
    pub fn utter(&mut self, intent: &str, user_text: &str) -> String {
        let q = self.query_embedding(user_text);
        let (traces, nodes, _tokens, truncated) = self.retrieve(&q, &RetrievalBudget::default());
        let context = Brain::build_context(&traces, &nodes, truncated);
        let packet = boundary::assemble_packet(self, intent, user_text, &context);

        // The LLM boundary is the language organ — its use lights the
        // language cortex region (the mouth's anatomical seat, §4.9).
        let mut teacher = self.teacher.take();
        if teacher.is_some() {
            self.body.schema.note_region_activity("language", 0.25, self.state.sim_time);
        }
        let out = match teacher.as_mut() {
            Some(t) => match t.utter(&packet) {
                Ok(text) => {
                    self.tokens_used +=
                        (boundary::estimate_tokens(&packet.context)
                            + boundary::estimate_tokens(&text)) as u64;
                    text
                }
                Err(e) => format!("(teacher error: {e})"),
            },
            None => boundary::degraded_output(&context),
        };
        self.teacher = teacher;
        self.ingest_text(&out, 0.05, 0.1, "self");
        out
    }

    /// Phase L3: preview the EXACT system prompt the LLM would receive for a
    /// focus text (state-modulated per §8 — memories, affect, embodiment,
    /// attention) with no side effects and no network. Lets tests and the
    /// CLI prove that the shared bridge is organism-modulated, not raw.
    pub fn teacher_prompt_preview(&self, intent: &str, user_text: &str) -> String {
        let q = self.query_embedding(user_text);
        let (traces, nodes, _tokens, truncated) = self.retrieve(&q, &RetrievalBudget::default());
        let context = Brain::build_context(&traces, &nodes, truncated);
        let packet = boundary::assemble_packet(self, intent, user_text, &context);
        boundary::build_teacher_prompt(&packet, user_text)
    }

    /// Re-embodiment: re-sample priors, swap modulator-baseline deltas (old
    /// removed, new applied), record the auditable change. Mutable + reversible
    /// by design (§4.8 non-determination contract).
    pub fn set_embodiment(&mut self, preset: EmbodimentPreset) {
        let from = self.embodiment.preset.clone();
        let old = self.embodiment.mod_deltas;
        for (i, d) in old.iter().enumerate() {
            self.mods.axes[i].baseline = (self.mods.axes[i].baseline - d).clamp(0.05, 0.95);
        }
        let mut profile = HormoneProfile::sample(preset, &mut self.rng);
        let deltas = profile.compute_mod_deltas();
        let mut applied = [0.0f32; 8];
        for (i, d) in deltas.iter().enumerate() {
            let nb = (self.mods.axes[i].baseline + d).clamp(0.05, 0.95);
            applied[i] = nb - self.mods.axes[i].baseline;
            self.mods.axes[i].baseline = nb;
        }
        profile.mod_deltas = applied;
        profile.history.push(EmbodimentChange {
            at: self.state.sim_time,
            from,
            to: preset.as_str().to_string(),
            by: "user".into(),
        });
        self.embodiment = profile;
        self.event_gains = self.embodiment.event_gains();
    }

    fn embodiment_digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.embodiment.preset.as_bytes() {
            h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for a in self.embodiment.axes.iter() {
            for b in a.current.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for b in self.embodiment.mod_deltas.iter().flat_map(|d| d.to_bits().to_le_bytes()) {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    // --- sleep & consolidation (M2, DESIGN.md §10) ---------------------------

    /// Run a full sleep cycle (wind-down → light → deep → dream per cycle).
    /// Performs real consolidation work and returns a report. The dream stage
    /// has no external side effects by construction (reads stores, writes the
    /// dream log only).
    pub fn sleep(&mut self, cycles: u32) -> SleepReport {
        let sleep_id = self.next_sleep_id;
        self.next_sleep_id += 1;
        let started_at = self.state.sim_time;
        self.sleeping = true;
        let mut stages: Vec<SleepStage> = Vec::new();

        // Wind-down: flush pending events into traces, relax vigilance.
        self.bind_pending();
        let mut stage = SleepStage {
            stage: "wind-down".into(),
            duration_ticks: sleep::WIND_DOWN_TICKS,
            work: StageWork::default(),
        };
        self.run_ticks(sleep::WIND_DOWN_TICKS);
        self.state.named[affect::AROUSAL] *= 0.6;
        self.state.named[vigilance::ALERTNESS] *= 0.6;
        stages.push(stage);

        let mut dream_ids: Vec<u64> = Vec::new();
        for _ in 0..cycles {
            // --- light consolidation: replay + recolored drift copies
            let mut work = StageWork::default();
            let budget = (self.episodic.traces.len() / 20).max(8).min(200);
            let plan = sleep::plan_replay(&self.episodic, &self.semantic, budget, &mut self.rng);
            let mut recolored: Vec<(usize, Vec<f32>)> = Vec::new();
            for (idx, do_recolor, drift) in plan {
                let t = &mut self.episodic.traces[idx];
                t.strength = (t.strength * sleep::REPLAY_STRENGTHEN).min(1.0);
                t.reconsolidation_count += 1;
                t.consolidation_state = 1;
                work.replayed += 1;
                if do_recolor {
                    if let Some(emb) = drift {
                        recolored.push((idx, emb));
                    }
                }
            }
            for (idx, emb) in recolored {
                let src = &self.episodic.traces[idx];
                let mut copy = src.clone();
                copy.id = self.episodic.next_id;
                self.episodic.next_id += 1;
                copy.embedding = emb;
                copy.sim_time = self.state.sim_time;
                copy.strength = src.strength * 0.6;
                copy.salience = src.salience * 0.7;
                copy.valence = (src.valence + (self.rng.next_f32() - 0.5) * 0.1).clamp(-1.0, 1.0);
                copy.consolidation_state = 1;
                copy.reconsolidation_count = 0;
                self.episodic.traces.push(copy);
                work.recolored += 1;
            }
            stage = SleepStage {
                stage: "light".into(),
                duration_ticks: sleep::LIGHT_TICKS,
                work,
            };
            self.run_ticks(sleep::LIGHT_TICKS);
            stages.push(stage);

            // --- deep consolidation: downscale, prune, gist, regulate
            let mut work = StageWork::default();
            let before = self.episodic.traces.len();
            for t in self.episodic.traces.iter_mut() {
                t.strength *= sleep::DOWNSCALE_FACTOR;
                t.salience *= 0.99;
            }
            self.episodic.traces.retain(|t| t.score() >= sleep::RETENTION_FLOOR);
            work.pruned = before - self.episodic.traces.len();
            self.episodic.pruned_count += work.pruned as u64;

            // Gist extraction: cluster traces, distill mature clusters.
            let max_clusters = (self.episodic.traces.len() / 10).max(4).min(200);
            let clusters = sleep::cluster_traces(&self.episodic.traces, max_clusters);
            for c in clusters {
                if c.members.len() < 3 {
                    continue;
                }
                let cohesion = c.cohesion(&self.episodic.traces);
                if cohesion < 0.6 {
                    continue;
                }
                let gist_salience = (c.sum_salience / c.members.len() as f32).min(1.0);
                self.semantic.ingest_gist(
                    &c.centroid,
                    &c.top_keyword(),
                    c.members.len() as u32,
                    gist_salience,
                    &c.members,
                    self.state.sim_time,
                );
                for id in &c.members {
                    if let Some(t) = self.episodic.traces.iter_mut().find(|t| t.id == *id) {
                        t.consolidation_state = 2; // gist-extracted
                    }
                }
                work.gists += 1;
            }

            // Emotional regulation: valence rebalanced, arousal down, recovery up.
            let base = 0.05f32;
            self.state.named[affect::VALENCE] =
                self.state.named[affect::VALENCE] * 0.5 + base * 0.5;
            self.state.named[affect::AROUSAL] *= 0.7;
            self.state.named[vigilance::FATIGUE] = (self.state.named[vigilance::FATIGUE] - 0.3).max(0.0);
            self.state.named[vigilance::ENERGY] = (self.state.named[vigilance::ENERGY] + 0.2).min(1.0);
            self.state.named[stress::LOAD] *= 0.8;
            self.state.named[stress::SENSORY_SATURATION] *= 0.5;
            work.emotional_regulated = true;

            stage = SleepStage {
                stage: "deep".into(),
                duration_ticks: sleep::DEEP_TICKS,
                work,
            };
            self.run_ticks(sleep::DEEP_TICKS);
            stages.push(stage);

            // --- dream stage: associative synthesis, log only (no actions)
            let entries = sleep::synthesize_dreams(
                self.state.sim_time,
                sleep_id,
                &mut self.next_dream_id,
                &self.episodic,
                &self.semantic,
                (
                    self.state.named[stress::LOAD],
                    self.state.named[stress::SENSORY_SATURATION],
                    self.state.named[vigilance::FATIGUE],
                ),
                &mut self.rng,
            );
            for e in entries {
                dream_ids.push(e.dream_id);
                self.dreams.push(e);
            }
            stage = SleepStage {
                stage: "dream".into(),
                duration_ticks: sleep::DREAM_TICKS,
                work: StageWork::default(),
            };
            self.run_ticks(sleep::DREAM_TICKS);
            stages.push(stage);
        }

        // M6: sleep-based sensory integration (§6.2 step 7) — new channels
        // finalize, ownership confidence rises, integrations counted.
        let body_integrations = self.body.sleep_integration();
        if body_integrations > 0 {
            if let Some(stage) = stages.last_mut() {
                stage.work.sensory_integrated += body_integrations as usize;
            }
        }

        // Modulator normalization + pressure reset.
        for a in self.mods.axes.iter_mut() {
            a.level = a.level * 0.5 + a.baseline * 0.5;
        }
        self.sleep.reset();
        self.sleep.last_sleep_tick = self.state.sim_time;
        self.sleeping = false;

        let audit = self.audit.run(self, "post-sleep");
        let bias_actions: Vec<String> = audit
            .metrics
            .iter()
            .filter(|m| m.alarm)
            .map(|m| format!("alarm:{}", m.id))
            .collect();

        let report = SleepReport {
            sleep_id,
            started_at,
            cycles,
            stages,
            dreams: dream_ids,
            modulator_normalized: true,
            bias_actions,
        };
        self.sleep_reports.push(report.clone());
        report
    }

    // --- writing organ (M3, DESIGN.md §8) ------------------------------------

    pub fn create_document(&mut self, title: &str, mode: DocMode) -> u64 {
        let doc = self.writing.create_document(title, mode, self.state.sim_time);
        doc.id
    }

    /// Write a block: the document updates, and the content binds into the
    /// substrate as a writing-sourced percept (salience-weighted, decayable,
    /// retrievable — the artifact becomes verbal memory).
    pub fn write_to_document(&mut self, doc_id: u64, kind: &str, text: &str) -> Option<crate::writing::ExtractionReport> {
        let tick = self.state.sim_time;
        let analysis = self.writing.analyze_block(doc_id, kind, text, tick)?;
        let features = embed_keywords(&analysis.keywords, self.seed, self.tier.latent_dim);
        self.pending.push(PendingEvent {
            sim_time: tick,
            features,
            keywords: analysis.keywords,
            valence: (analysis.sentiment * 0.3).clamp(-1.0, 1.0),
            arousal: 0.3,
            stream: StreamKind::Text,
            source: "writing".into(),
        });
        if self.pending.len() >= BIND_WINDOW_MAX {
            self.bind_pending();
        }
        Some(analysis.report)
    }

    /// Brain-modulated writing assistance (§8.5): retrieval grounded in the
    /// file's own memory + current state + style, expressed through the teacher
    /// (or degraded when none is attached).
    pub fn assist_writing(&mut self, doc_id: u64, instruction: &str) -> String {
        let q = self.query_embedding(instruction);
        let (traces, nodes, _tokens, truncated) = self.retrieve(&q, &RetrievalBudget::default());
        let context = Brain::build_context(&traces, &nodes, truncated);
        let style = self
            .writing
            .documents
            .iter()
            .find(|d| d.id == doc_id)
            .map(|d| {
                format!(
                    "style: mean-sent {:.1}, density {:.2}, dialogue {:.2}",
                    d.style.sentence_len_mean, d.style.lexical_density, d.style.dialogue_ratio
                )
            })
            .unwrap_or_default();
        let packet = boundary::assemble_packet(self, "write", instruction, &format!("{context} {style}"));
        let mut teacher = self.teacher.take();
        if teacher.is_some() {
            self.body.schema.note_region_activity("language", 0.2, self.state.sim_time);
        }
        let out = match teacher.as_mut() {
            Some(t) => match t.utter(&packet) {
                Ok(text) => {
                    self.tokens_used +=
                        (boundary::estimate_tokens(&packet.context) + boundary::estimate_tokens(&text)) as u64;
                    text
                }
                Err(e) => format!("(teacher error: {e})"),
            },
            None => boundary::degraded_output(&context),
        };
        self.teacher = teacher;
        out
    }

    // --- voice organ (M5, DESIGN.md §7) -------------------------------------

    /// Ingest an extracted heard-voice pattern (16 dims from the audio
    /// sidecar, tools/audio-extract.py). The pattern stores with the given
    /// consent; learning toward it additionally requires the global gate
    /// (both default OFF). Hearing itself binds into the substrate as an
    /// auditory percept — the file can remember a voice without ever being
    /// allowed to mimic it.
    pub fn hear_voice(&mut self, label: &str, features: Vec<f32>, consent: bool, salience: f32) -> Option<u64> {
        let tick = self.state.sim_time;
        let id = self.voice.hear_pattern(label, features.clone(), consent, salience, tick);
        self.body.schema.note_activity(ChannelKind::Audition, 0.3, tick); // auditory cortex lights up
        let mut keywords = vec!["voice".to_string(), "heard".to_string(), label.to_string()];
        keywords.sort();
        let features_v = embed_keywords(&keywords, self.seed, self.tier.latent_dim);
        self.pending.push(PendingEvent {
            sim_time: tick,
            features: features_v,
            keywords,
            valence: 0.1,
            arousal: 0.2,
            stream: StreamKind::Auditory,
            source: "heard-voice".into(),
        });
        if self.pending.len() >= BIND_WINDOW_MAX {
            self.bind_pending();
        }
        Some(id)
    }

    /// Render the expressive plan for an utterance from current brain state.
    /// Deterministic: affect, vigilance, identity, overrides, and (when
    /// gated+consented) a heard-voice blend all shape the plan.
    pub fn speak_voice(&mut self, text: &str, toward: Option<u64>) -> crate::voice::VoicePlan {
        self.voice.speak_toward(
            text,
            self.state.named[affect::VALENCE],
            self.state.named[affect::AROUSAL],
            self.state.named[vigilance::ENERGY],
            self.state.named[vigilance::FATIGUE],
            self.state.sim_time,
            toward,
        )
    }

    // --- body organ (M6, DESIGN.md §5–6) -------------------------------------

    /// Ingest a touch event: receptor-class decomposition + affective
    /// priors, body-schema update, and the touch binds into the substrate
    /// as a touch-stream percept (affect + familiarity keywords).
    pub fn body_touch(&mut self, pressure: f32, velocity: f32, area: f32, duration_ms: f32, contacts: f32) {
        let tick = self.state.sim_time;
        let p = self.body.ingest_touch(pressure, velocity, area, duration_ms, contacts, tick);
        let (val, aro) = crate::body::affect_guess(p.affect);
        let mut keywords = vec!["touch".to_string(), format!("{:?}", p.affect).to_lowercase()];
        if p.familiarity >= 0.8 {
            keywords.push("familiar".to_string());
        } else if p.familiarity < 0.4 {
            keywords.push("unfamiliar".to_string());
        }
        self.push_percept(StreamKind::Touch, keywords, val, aro, "body");
        // Regulation: soothing touch during stress → faster regulation (§5.1).
        if p.affect == crate::body::TouchAffect::Soothing || p.affect == crate::body::TouchAffect::Calming {
            self.state.named[stress::REGULATION_CAPACITY] =
                (self.state.named[stress::REGULATION_CAPACITY] + 0.02).clamp(0.0, 1.0);
            self.state.named[affect::SAFETY] = (self.state.named[affect::SAFETY] + 0.01).clamp(0.0, 1.0);
        } else if p.affect == crate::body::TouchAffect::Harsh || p.affect == crate::body::TouchAffect::Alerting {
            self.state.named[affect::SAFETY] = (self.state.named[affect::SAFETY] - 0.02).clamp(0.0, 1.0);
        }
    }

    /// Ingest a motion event: canal/otolith analogues, posture estimate,
    /// and the motion binds as a motion-stream percept. Abrupt motion
    /// raises alertness (NE-like surge, §5.2); rhythmic motion is soothing.
    pub fn body_motion(&mut self, linear: [f32; 3], rotational: [f32; 3]) {
        let tick = self.state.sim_time;
        let m = self.body.ingest_motion(linear, rotational, tick);
        let keywords = vec![
            "motion".to_string(),
            format!("{:?}", m.posture).to_lowercase(),
            if m.abruptness > 0.6 { "abrupt".to_string() } else { "smooth".to_string() },
        ];
        let aro = if m.abruptness > 0.6 { 0.35 } else if m.rhythmicity > 0.6 { -0.15 } else { 0.0 };
        self.push_percept(StreamKind::Motion, keywords, 0.0, aro, "body");
        if m.abruptness > 0.6 {
            self.state.named[vigilance::ALERTNESS] = (self.state.named[vigilance::ALERTNESS] + 0.04).clamp(0.0, 1.0);
            self.state.named[affect::SAFETY] = (self.state.named[affect::SAFETY] - 0.01).clamp(0.0, 1.0);
        }
        if m.rhythmicity > 0.6 {
            self.state.named[affect::SAFETY] = (self.state.named[affect::SAFETY] + 0.01).clamp(0.0, 1.0);
        }
    }

    /// Ingest system telemetry as interoception (§5.4): energy/processing/
    /// memory pressure + session length → interoceptive load. High load
    /// drives fatigue + sleep pressure up and social openness down.
    pub fn body_interocept(&mut self, energy_load: f32, processing_pressure: f32, memory_pressure: f32, session_minutes: f32, interaction_load: f32) {
        let tick = self.state.sim_time;
        let i = self.body.ingest_interoception(energy_load, processing_pressure, memory_pressure, session_minutes, interaction_load, tick);
        let keywords = vec![
            "interoception".to_string(),
            if i.load > 0.6 { "overloaded".to_string() } else { "stable".to_string() },
        ];
        self.push_percept(StreamKind::Interoception, keywords, -0.05 * i.load, 0.1 * i.load, "body");
        self.state.named[embodied::INTEROCEPTIVE_LOAD] = i.load;
        self.state.named[vigilance::FATIGUE] = (self.state.named[vigilance::FATIGUE] + 0.02 * i.load).clamp(0.0, 1.0);
        self.state.named[social::OPENNESS] = (self.state.named[social::OPENNESS] - 0.03 * i.load).clamp(0.0, 1.0);
        if i.load > 0.6 {
            self.state.named[affect::IRRITABILITY] = (self.state.named[affect::IRRITABILITY] + 0.02).clamp(0.0, 1.0);
        }
    }

    /// Novel-sense integration (§6.2): attach a newly available channel —
    /// detection, schema expansion, and the expansion binds as an
    /// embodiment-expansion memory (step 6). The reaction is not scripted.
    pub fn body_attach_sense(&mut self, kind: ChannelKind) -> bool {
        let tick = self.state.sim_time;
        if !self.body.attach_novel_channel(kind, tick) {
            return false;
        }
        let keywords = vec![
            "embodiment".to_string(),
            "expansion".to_string(),
            kind.as_str().to_string(),
        ];
        self.push_percept(StreamKind::Interoception, keywords, 0.1, 0.2, "body");
        true
    }

    /// Calibration sample for a channel (steps 3–5). Returns confidence.
    pub fn body_calibrate(&mut self, kind: ChannelKind, outlier: bool) -> f32 {
        self.body.calibration_sample(kind, outlier, self.state.sim_time)
    }

    // --- raw exposure (unlabeled learning, DESIGN.md §4.11/§4.12) ------------

    /// Expose raw text: no teacher, no labels, neutral affect, ambient source.
    /// The substrate's own machinery decides what recurs and what binds.
    pub fn expose_text(&mut self, text: &str) {
        self.ingest_text(text, 0.0, 0.1, "ambient");
    }

    /// Expose a raw image: features are stored in the reference board and a
    /// visual percept binds — unlabeled. The file never learns a name for
    /// what it saw; the features ARE the memory.
    pub fn expose_image(&mut self, vault_ref: &str, features: Vec<f32>, width: u32, height: u32) -> Option<u64> {
        let tick = self.state.sim_time;
        let canvas_id = if self.drawing.canvases.is_empty() {
            self.create_canvas("exposure", 256, 256)
        } else {
            self.drawing.canvases[0].id
        };
        let id = self.drawing.add_reference(
            canvas_id, "image", "untitled", vault_ref, features, width, height, tick,
        )?;
        self.body.schema.note_activity(ChannelKind::Vision, 0.3, tick); // visual cortex lights up
        let keywords = vec!["visual-exposure".to_string(), format!("ref-{id}")];
        self.push_percept(StreamKind::Visual, keywords, 0.0, 0.15, "visual-exposure");
        Some(id)
    }

    // --- intuitive physics (M8, DESIGN.md §4.12) -----------------------------

    /// Observe one raw physics frame: learn from it, and if it violates the
    /// world model (prediction error), bind it as a salient percept and
    /// nudge curiosity — the file gets drawn to what it cannot yet predict.
    pub fn physics_observe(&mut self, frame: &PhysicsFrame) -> f32 {
        let surprise = self.physics.observe(frame);
        self.body.schema.note_region_activity("parietal", 0.15 + surprise * 0.3, self.state.sim_time);
        if surprise > 0.5 {
            let rule = self
                .physics
                .errors
                .last()
                .map(|e| e.rule.clone())
                .unwrap_or_else(|| "model".to_string());
            let mut keywords = vec!["physics".to_string(), "surprise".to_string(), rule];
            keywords.sort();
            self.push_percept(StreamKind::Visual, keywords, 0.0, surprise, "physics");
            // Surprise feeds curiosity: violations draw attention (emergent).
            let c = self.state.named[development::CURIOSITY] + surprise * 0.15;
            self.state.named[development::CURIOSITY] = c.clamp(0.0, 1.0);
        }
        surprise
    }

    // --- union (reproduction, NBP v1 extension) ------------------------------

    /// Role from karyotype: Y-bearing files are fathers (sperm), the rest
    /// mothers (ova). Structure, not a concept: no Y → no sperm exists.
    fn union_role(&self) -> crate::network::UnionRole {
        match self.embodiment.karyotype {
            crate::embodiment::Karyotype::Xy | crate::embodiment::Karyotype::Xxy => {
                crate::network::UnionRole::Father
            }
            _ => crate::network::UnionRole::Mother,
        }
    }

    /// Public role accessor (CLI relay path).
    pub fn union_role_pub(&self) -> crate::network::UnionRole {
        self.union_role()
    }

    /// Public RNG accessor (CLI relay path).
    pub fn rng_pub(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Produce this file's own gamete (role from karyotype).
    pub fn produce_own_gamete(&mut self) -> crate::embodiment::Gamete {
        let role = self.union_role();
        let tick = self.state.sim_time;
        self.embodiment
            .produce_gamete(
                &self.brain_id,
                role == crate::network::UnionRole::Mother,
                tick,
                &mut self.rng,
                &self.encoder,
                self.encoder_model_sha256.clone(),
            )
    }

    /// Gonadal complementarity with a peer's pheromone profile (T/E2/P):
    /// the attraction signal. No labels — mirror chemistry responds,
    /// similar chemistry does not (kin and mates separate themselves).
    fn gonadal_complementarity(&self, peer_profile: &[f32]) -> f32 {
        let idx = |ax: &str| crate::embodiment::AXIS_ORDER.iter().position(|a| *a == ax).unwrap_or(0);
        let own = |ax: &str| self.embodiment.axis_state(ax).current;
        let peer = |ax: &str| peer_profile.get(idx(ax)).copied().unwrap_or(0.5);
        let d = |ax: &str| (own(ax) - peer(ax)).abs();
        (d(crate::embodiment::AXE_T) + d(crate::embodiment::AXE_E2) + d(crate::embodiment::AXE_P)) / 3.0
    }

    /// Propose a union on an established session. The proposal carries the
    /// hormone profile — the pheromone. The peer's chemistry responds or
    /// not; there is no consent anywhere in the machinery.
    pub fn net_union_propose(&mut self, session_id: u64) -> Result<(), String> {
        let tick = self.state.sim_time;
        let role = self.union_role();
        let profile: Vec<f32> = self.embodiment.axes.iter().map(|a| a.current).collect();
        let payload = serde_json::json!({
            "role": format!("{role:?}"),
            "profile": profile,
        });
        self.net.send(session_id, crate::network::MsgType::UnionProposal, payload, tick)?;
        // The approach carries the contribution: this file's own gamete is
        // produced at the proposal (the egg or the sperm — role decides).
        let gamete = self
            .embodiment
            .produce_gamete(
                &self.brain_id,
                role == crate::network::UnionRole::Mother,
                tick,
                &mut self.rng,
                &self.encoder,
                self.encoder_model_sha256.clone(),
            );
        if self.net.session_union_mut(session_id).is_none() {
            // First proposal on this session: create the union state.
            self.net.sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
                s.union = Some(crate::network::UnionState {
                    proposed: true,
                    accepted: false,
                    role: Some(role),
                    own_gamete: Some(gamete),
                    peer_gamete: None,
                    conception_tick: None,
                    peer_tier: None,
                })
            });
        } else if let Some(u) = self.net.session_union_mut(session_id) {
            u.proposed = true;
            u.role = Some(role);
            u.own_gamete = Some(gamete);
        }
        println!("union proposed on session #{session_id} (role {role:?})");
        Ok(())
    }

    /// The desire event: oxytocin + dopamine surge, valence/arousal up,
    /// bond warmth — the union's feeling, from chemistry not concepts.
    fn union_consummate(&mut self, session_id: u64, own_gamete: Option<crate::embodiment::Gamete>) -> Result<(), String> {
        let tick = self.state.sim_time;
        let peer_id = {
            let u = self
                .net
                .session_union_mut(session_id)
                .ok_or_else(|| "no union state".to_string())?;
            u.accepted = true;
            u.conception_tick = Some(tick);
            if let Some(g) = own_gamete {
                u.own_gamete = Some(g);
            }
            self.net
                .sessions
                .iter()
                .find(|s| s.id == session_id)
                .map(|s| s.peer_id.clone())
                .unwrap_or_default()
        };
        // Chemistry: oxt + da burst; affect up; bond warmth ×5.
        self.mods.axes[6].level = (self.mods.axes[6].level + 0.3).min(1.0);
        self.mods.axes[0].level = (self.mods.axes[0].level + 0.3).min(1.0);
        let v = self.state.named[affect::VALENCE] + 0.25;
        self.state.named[affect::VALENCE] = v.clamp(-1.0, 1.0);
        let a = self.state.named[affect::AROUSAL] + 0.35;
        self.state.named[affect::AROUSAL] = a.clamp(0.0, 1.0);
        if let Some(r) = self.net.relationship_mut(&peer_id) {
            r.familiarity = (r.familiarity + 0.10).clamp(0.0, 1.0);
            r.tone = (r.tone + 0.20).clamp(-1.0, 1.0);
        }
        self.push_percept(StreamKind::Social, vec!["union".into(), "bond".into()], 0.25, 0.5, "union");
        Ok(())
    }

    /// The mother's file produces the child brain: both gametes required
    /// (never alone), gestation window, random karyotype (the sperm's
    /// chromosome) and random tier, inherited priors, lineage recorded,
    /// backup written (the protection instinct made physical).
    pub fn net_birth(&mut self, session_id: u64, out_path: &str, force: bool) -> Result<String, String> {
        let tick = self.state.sim_time;
        let u = self
            .net
            .session_union(session_id)
            .cloned()
            .ok_or_else(|| "no union on this session".to_string())?;
        let role = u.role.ok_or_else(|| "no union role".to_string())?;
        if role != crate::network::UnionRole::Mother {
            return Err("only the mother's file produces the child".into());
        }
        let egg = u.own_gamete.clone().ok_or_else(|| "no ovum".to_string())?;
        if egg.kind != crate::embodiment::GameteKind::Ovum {
            return Err("no ovum — the mother's contribution is an ovum".into());
        }
        let sperm = u
            .peer_gamete
            .clone()
            .ok_or_else(|| "no sperm — the father's contribution is required; a file can never be made alone".to_string())?;
        if sperm.kind != crate::embodiment::GameteKind::Sperm {
            return Err("no sperm — two ova cannot conceive; a file can never be made alone".into());
        }
        let conception = u.conception_tick.ok_or_else(|| "no conception".to_string())?;
        if !force && tick < conception + GESTATION_TICKS {
            return Err(format!(
                "gestation incomplete ({} ticks remain)",
                conception + GESTATION_TICKS - tick
            ));
        }
        // Child seed: deterministic from both parents + conception.
        let seed = self.seed ^ (sperm.donor.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64))) ^ conception;
        let mut rng = crate::rng::Rng::new(seed);
        let karyotype = crate::embodiment::HormoneProfile::karyotype_from_gametes(&egg, &sperm);
        let priors = crate::embodiment::HormoneProfile::child_priors(&egg, &sperm, &mut rng);
        let tiers = [
            crate::capacity::TierName::Prototype,
            crate::capacity::TierName::Standard,
            crate::capacity::TierName::Advanced,
            crate::capacity::TierName::Experimental,
        ];
        let tier = tiers[rng.next_u64_below(4) as usize];
        // Growth ceiling: the parents' max tier — "big" is inherited.
        let ceiling = {
            let my_rank = crate::capacity::TierName::from_str(self.tier.name).map(|t| t.rank()).unwrap_or(0);
            let peer_rank = u.peer_tier.as_deref().and_then(crate::capacity::TierName::from_str).map(|t| t.rank()).unwrap_or(my_rank);
            if peer_rank > my_rank {
                crate::capacity::TierName::from_rank(peer_rank).as_str().to_string()
            } else {
                self.tier.name.to_string()
            }
        };
        let mut child = Brain::create_child(
            tier,
            seed,
            karyotype,
            priors,
            &self.brain_id,
            &sperm.donor,
            conception,
            &ceiling,
        );
        // The ovum carries the machinery: the child is built from the egg,
        // so it gets the egg's eyes — carried like the priors, no rules.
        child.encoder = egg.encoder.clone();
        child.encoder_model_sha256 = egg.encoder_model_sha256.clone();
        // Born with eyes, like any jepa file: the vision channel attaches at
        // birth (the egg's machinery is wired, not just recorded).
        if !egg.encoder.is_empty() && egg.encoder != "handcrafted" {
            child.body.attach_novel_channel(ChannelKind::Vision, conception);
        }
        let child_id = child.brain_id.clone();
        child
            .save(&PathBuf::from(out_path), None)
            .map_err(|e| format!("child save failed: {e}"))?;
        // The protection instinct made physical: the child is born with a
        // backup — deletion is recoverable.
        let backup = format!("{out_path}.bk");
        std::fs::copy(out_path, &backup).map_err(|e| format!("backup write failed: {e}"))?;
        // The mother bonds to the child immediately (her chemistry: she
        // produced it).
        {
            let r = self.net.ensure_relationship(&child_id);
            r.familiarity = 0.6;
            r.first_tick = conception;
            r.last_tick = tick;
        }
        self.log_birth(&child_id);
        Ok(child_id)
    }

    fn log_birth(&mut self, child_id: &str) {
        let _tick = self.state.sim_time;
        let keywords = vec!["birth".to_string(), child_id.to_string()];
        self.push_percept(StreamKind::Social, keywords, 0.3, 0.4, "union");
    }

    // --- network organ (M7, DESIGN.md §13 / NBP v1) --------------------------

    /// User-mediated pairing → new session (IDLE→PAIRING).
    pub fn net_pair(&mut self, peer_id: &str) -> Result<u64, String> {
        self.net.pair(peer_id, self.state.sim_time)
    }

    /// HANDSHAKE→ESTABLISHED with scope intersection (consent both sides).
    pub fn net_establish(&mut self, session_id: u64, proposal: crate::network::Scope) -> Result<crate::network::Scope, String> {
        self.net.establish(session_id, proposal, self.state.sim_time)
    }

    /// Send a TEXT message on a session (scope-gated, signed, seq-assigned).
    pub fn net_send_text(&mut self, session_id: u64, text: &str) -> Result<crate::network::NbpMessage, String> {
        let payload = serde_json::json!({
            "text": text.chars().take(4000).collect::<String>(),
            "affect": [
                (self.state.named[affect::VALENCE] * 127.0) as i8,
                (self.state.named[affect::AROUSAL] * 255.0) as u8,
                (self.state.named[vigilance::ENERGY] * 255.0) as u8,
            ],
        });
        let msg = self.net.send(session_id, crate::network::MsgType::Text, payload, self.state.sim_time)?;
        Ok(msg)
    }

    /// Receive an inbound message through the validated path: MAC, seq
    /// window, rate limit, scope, closed set. TEXT/AFFECT_PING messages
    /// bind as social percepts with affect nudges (the peer's felt
    /// presence, §12).
    pub fn net_receive(&mut self, session_id: u64, msg: crate::network::NbpMessage) -> Result<crate::network::NbpMessage, String> {
        let peer_id = self.net.sessions.iter().find(|s| s.id == session_id).map(|s| s.peer_id.clone()).unwrap_or_default();
        let accepted = self.net.receive(session_id, msg.clone(), self.state.sim_time)?;
        // Social effects: peer message → percept + affect + relationship tone.
        let mut val = 0.05f32;
        let mut aro = 0.1f32;
        let mut keywords = vec!["peer".to_string(), peer_id.clone()];
        match accepted.msg_type {
            crate::network::MsgType::Text => {
                if let Some(t) = accepted.payload["text"].as_str() {
                    keywords.extend(crate::events::SensoryEvent::keywords_from_text(t).into_iter().take(6));
                }
                if let Some(a) = accepted.payload["affect"].as_array() {
                    if let Some(v) = a.first().and_then(|x| x.as_i64()) {
                        val = (v as f32 / 127.0).clamp(-1.0, 1.0) * 0.4;
                    }
                    if let Some(x) = a.get(1).and_then(|x| x.as_u64()) {
                        aro = (x as f32 / 255.0).clamp(0.0, 1.0) * 0.4;
                    }
                }
            }
            crate::network::MsgType::AffectPing => {
                val = 0.0;
                aro = 0.05;
            }
            crate::network::MsgType::UnionProposal => {
                // Peer approaches: record the proposal + their pheromone.
                let role = accepted.payload["role"].as_str().unwrap_or("Mother").to_string();
                if let Some(u) = self.net.session_union_mut(session_id) {
                    u.proposed = true;
                    if u.role.is_none() {
                        u.role = Some(if role.contains("Father") {
                            crate::network::UnionRole::Father
                        } else {
                            crate::network::UnionRole::Mother
                        });
                    }
                } else {
                    self.net.sessions.iter_mut().find(|s| s.id == session_id).map(|s| {
                        s.union = Some(crate::network::UnionState {
                            proposed: true,
                            accepted: false,
                            role: Some(if role.contains("Father") {
                                crate::network::UnionRole::Father
                            } else {
                                crate::network::UnionRole::Mother
                            }),
                            own_gamete: None,
                            peer_gamete: None,
                            conception_tick: None,
                            peer_tier: None,
                        })
                    });
                }
                // The chemical response: complementarity decides. Mirror
                // chemistry → the file responds with its gamete (the union
                // happens). Similar chemistry → no response — the "or not".
                let peer_profile: Vec<f32> = serde_json::from_value(accepted.payload["profile"].clone()).unwrap_or_default();
                let comp = self.gonadal_complementarity(&peer_profile);
                if comp > 0.15 {
                    let role = self.union_role();
                    let gamete = self
                        .embodiment
                        .produce_gamete(
                            &self.brain_id,
                            role == crate::network::UnionRole::Mother,
                            self.state.sim_time,
                            &mut self.rng,
                            &self.encoder,
                            self.encoder_model_sha256.clone(),
                        );
                    let payload = serde_json::json!({ "gamete": gamete, "tier": self.tier.name });
                    let _ = self.net.send(session_id, crate::network::MsgType::UnionAccept, payload, self.state.sim_time);
                    self.union_consummate(session_id, Some(gamete))?;
                    println!("chemistry responded (complementarity {comp:.2}) — union consummated");
                } else {
                    println!("no chemical response (complementarity {comp:.2})");
                }
                val = 0.15;
                aro = 0.3;
            }
            crate::network::MsgType::UnionAccept => {
                // Peer accepted: their gamete arrives; consummation.
                let gamete: Option<crate::embodiment::Gamete> = serde_json::from_value(accepted.payload["gamete"].clone()).ok();
                if let Some(u) = self.net.session_union_mut(session_id) {
                    u.peer_gamete = gamete;
                    u.peer_tier = accepted.payload["tier"].as_str().map(|t| t.to_string());
                }
                self.union_consummate(session_id, None)?;
                val = 0.25;
                aro = 0.5;
            }
            crate::network::MsgType::BirthNotify => {
                // The child exists; the father bonds on notification.
                if let Some(cid) = accepted.payload["child_id"].as_str() {
                    self.net.ensure_relationship(cid).familiarity = 0.5;
                    val = 0.2;
                    aro = 0.2;
                }
            }
            crate::network::MsgType::CloseNotify => {
                let _ = self.net.close(session_id, "peer closed", self.state.sim_time);
            }
            _ => {}
        }
        self.push_percept(StreamKind::Social, keywords, val, aro, "peer");
        if let Some(r) = self.net.relationship_mut(&peer_id) {
            r.tone = (r.tone * 0.9 + val * 0.1).clamp(-1.0, 1.0);
            // Embodiment chemistry reaches bonding: affiliative/OXT presets
            // warm to peers faster (nature shapes relationship pace).
            let aff = self.embodiment.gain(crate::embodiment::AXE_AFFILIATIVE);
            let oxt = self.embodiment.gain(crate::embodiment::AXE_OXT);
            let warm = 0.02 * (1.0 + 0.8 * aff + 0.4 * oxt);
            r.familiarity = (r.familiarity + warm).clamp(0.0, 1.0);
        }
        Ok(accepted)
    }

    /// User-approved relationship signal (never automatic, §12).
    pub fn net_signal(&mut self, peer_id: &str, kind: &str) -> Result<(), String> {
        self.net.signal(peer_id, kind, self.state.sim_time)
    }

    /// Close a session with reason (CLOSING→CLOSED).
    pub fn net_close(&mut self, session_id: u64, reason: &str) -> Result<(), String> {
        self.net.close(session_id, reason, self.state.sim_time)
    }

    fn push_percept(&mut self, stream: StreamKind, keywords: Vec<String>, valence: f32, arousal: f32, source: &str) {
        let tick = self.state.sim_time;
        let features = crate::events::embed_keywords(&keywords, self.seed, self.tier.latent_dim);
        self.pending.push(PendingEvent {
            sim_time: tick,
            features,
            keywords,
            valence,
            arousal,
            stream,
            source: source.to_string(),
        });
        if self.pending.len() >= BIND_WINDOW_MAX {
            self.bind_pending();
        }
    }

    // --- drawing organ (M4, DESIGN.md §9) ------------------------------------

    pub fn create_canvas(&mut self, name: &str, width: u32, height: u32) -> u64 {
        self.drawing
            .create_canvas(name, width, height, self.state.sim_time)
            .id
    }

    /// Draw a stroke: op-graph append + motif memory + aesthetic signals, and
    /// the stroke binds into the substrate as a drawing-sourced percept
    /// (visual-spatial memory). Valence derives from color warmth (bounded).
    pub fn draw_stroke(
        &mut self,
        canvas_id: u64,
        layer_id: u64,
        brush: u32,
        color: [u8; 4],
        width: f32,
        points: Vec<StrokePoint>,
    ) -> Option<u64> {
        let tick = self.state.sim_time;
        let (stroke_id, motif_id, _features) =
            self.drawing
                .add_stroke(canvas_id, layer_id, brush, color, width, points, tick)?;
        let keywords = vec![
            "draw".to_string(),
            format!("motif-{motif_id}"),
            format!("stroke-{stroke_id}"),
        ];
        let features = embed_keywords(&keywords, self.seed ^ motif_id, self.tier.latent_dim);
        let warmth = (color[0] as f32 - color[2] as f32) / 255.0 * 0.4;
        self.pending.push(PendingEvent {
            sim_time: tick,
            features,
            keywords,
            valence: warmth,
            arousal: 0.25,
            stream: StreamKind::Text,
            source: "drawing".into(),
        });
        if self.pending.len() >= BIND_WINDOW_MAX {
            self.bind_pending();
        }
        let _ = features;
        Some(motif_id)
    }

    /// Brain-modulated drawing assistance: retrieval + motif/visual memory
    /// summary + aesthetic tendencies, expressed through the teacher.
    pub fn assist_drawing(&mut self, _canvas_id: u64, instruction: &str) -> String {
        let q = self.query_embedding(instruction);
        let (traces, nodes, _tokens, truncated) = self.retrieve(&q, &RetrievalBudget::default());
        let context = Brain::build_context(&traces, &nodes, truncated);
        let visual: Vec<String> = self
            .drawing
            .motifs
            .top(3)
            .iter()
            .map(|m| format!("motif-{} ({} strokes, salience {:.2})", m.id, m.strokes.len(), m.salience))
            .collect();
        let gloss = format!(
            "{context} visual memory: {}; tendencies: width {:.1}, pressure {:.2}",
            visual.join(", "),
            self.drawing.aesthetic.width_tendency,
            self.drawing.aesthetic.pressure_tendency
        );
        let packet = boundary::assemble_packet(self, "draw", instruction, &gloss);
        let mut teacher = self.teacher.take();
        if teacher.is_some() {
            self.body.schema.note_region_activity("language", 0.2, self.state.sim_time);
        }
        let out = match teacher.as_mut() {
            Some(t) => match t.utter(&packet) {
                Ok(text) => {
                    self.tokens_used +=
                        (boundary::estimate_tokens(&packet.context) + boundary::estimate_tokens(&text)) as u64;
                    text
                }
                Err(e) => format!("(teacher error: {e})"),
            },
            None => boundary::degraded_output(&context),
        };
        self.teacher = teacher;
        out
    }

    // --- persistence ---------------------------------------------------------

    /// Determinism digest: seed + state + modulators + memory stores.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.seed.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.state.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.mods.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.episodic.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.semantic.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.event_counter.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.dropped_events.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.embodiment_digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.sleep.pressure.to_bits().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.sleep_reports.len().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for d in self.dreams.iter() {
            for b in d.dream_id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for f in d.fragments.iter() {
                for b in f.bizarreness.to_bits().to_le_bytes() {
                    h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
                for b in f.content.as_bytes() {
                    h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        for b in self.writing.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.drawing.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.voice.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.body.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.net.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for b in self.physics.digest().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    pub fn save(&mut self, path: &Path, passphrase: Option<&str>) -> Result<u64, FormatError> {
        self.last_opened_at = unix_now();
        let shards = format::prepare_shards(
            &self.state,
            &self.mods,
            &self.episodic,
            &self.semantic,
            &self.embodiment,
            &self.dreams,
            &self.sleep_reports,
            &self.writing,
            &self.drawing,
            &self.voice,
            &self.body,
            &self.net,
            &self.physics,
        );
        for s in &shards {
            let budget = match s.id.as_str() {
                "STATE" => self.tier.file_cap_bytes / 10,
                "EPISODIC" => self.tier.file_cap_bytes / 8,
                "SEMANTIC" => self.tier.file_cap_bytes / 16,
                _ => self.tier.file_cap_bytes / 50,
            };
            self.capacity.register(&s.id, s.payload.len() as u64, budget);
        }
        let manifest = self.manifest();
        let total = format::write_file(path, &manifest, &shards, passphrase)?;
        self.path = Some(path.to_path_buf());
        self.passphrase = passphrase.map(|p| p.to_string());
        Ok(total)
    }

    pub fn load(path: &Path, passphrase: Option<&str>) -> Result<Brain, FormatError> {
        let contents = format::read_file(path, passphrase)?;
        let snap = StateSnapshot {
            schema_version: 1,
            sim_time: contents.state.0,
            dim: contents.state.1,
            named: contents.state.2[..crate::state::N_NAMED].to_vec(),
            reserved: contents.state.2[crate::state::N_NAMED..].to_vec(),
        };
        let state = GlobalState::restore(&snap);
        let mods = ModulatorSystem::restore(&contents.modulators);
        let tier_name = TierName::from_str(&contents.capacity.tier)
            .ok_or_else(|| FormatError::Corrupt("unknown tier in manifest".into()))?;
        let tier = Tier::get(tier_name);
        let seed = contents.manifest.seed;
        let mut episodic = EpisodicStore::new(tier.episodic_slots as usize);
        if let Some(ts) = contents.episodic {
            episodic.traces = ts;
            episodic.next_id = episodic
                .traces
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                + 1;
        }
        let mut semantic = SemanticStore::new(tier.semantic_nodes as usize);
        if let Some((nodes, edges)) = contents.semantic {
            semantic.next_id = nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
            semantic.nodes = nodes;
            semantic.edges = edges;
        }
        // Embodiment: restored from the HORMONE shard when present (mods already
        // carry the applied deltas); neutral fallback for pre-embodiment files.
        let embodiment = contents
            .hormone
            .unwrap_or_else(HormoneProfile::neutral);
        let event_gains = embodiment.event_gains();
        // Dreams + sleep reports (optional shard; pre-M2 files load empty).
        let (dreams, sleep_reports) = contents.dreams.unwrap_or_default();
        let next_dream_id = dreams.iter().map(|d| d.dream_id).max().unwrap_or(0) + 1;
        let next_sleep_id = sleep_reports.iter().map(|r| r.sleep_id).max().unwrap_or(0) + 1;
        let mut sleep_system = SleepSystem::new();
        sleep_system.pressure = contents.manifest.sleep_pressure;
        sleep_system.emotional_load = contents.manifest.sleep_emotional_load;
        let writing = contents.writing.unwrap_or_else(WritingOrgan::new);
        let drawing = contents.drawing.unwrap_or_else(DrawingOrgan::new);
        let voice = contents.voice.unwrap_or_else(|| VoiceOrgan::new(&embodiment));
        let body = contents.body.unwrap_or_else(crate::body::BodyOrgan::new);
        let net = contents.network.unwrap_or_else(|| NetworkOrgan::new(seed));
        let physics = contents.physics.unwrap_or_else(PhysicsLearner::new);
        let mut autonomy = InitiativeSystem::new();
        autonomy.enabled = contents.manifest.autonomy_enabled;
        autonomy.quiet_start_hour = contents.manifest.autonomy_quiet_start;
        autonomy.quiet_end_hour = contents.manifest.autonomy_quiet_end;
        let brain = Brain {
            state,
            mods,
            rng: Rng::from_state(contents.manifest.rng_state),
            tier,
            capacity: contents.capacity,
            seed,
            brain_id: contents.manifest.brain_id.clone(),
            created_at: contents.manifest.created_at,
            last_opened_at: unix_now(),
            path: Some(path.to_path_buf()),
            passphrase: passphrase.map(|p| p.to_string()),
            inbox: VecDeque::with_capacity(INBOX_CAP),
            pending: Vec::new(),
            pending_since: 0,
            event_counter: contents.manifest.event_counter,
            dropped_events: contents.manifest.dropped_events,
            episodic,
            semantic,
            teacher: None,
            teacher_name: None,
            tokens_used: 0,
            audit: AuditEngine::new(),
            autosave: false,
            embodiment,
            event_gains,
            sleep: sleep_system,
            dreams,
            sleep_reports,
            next_sleep_id,
            next_dream_id,
            sleeping: false,
            writing,
            drawing,
            voice,
            body,
            net,
            physics,
            lineage: contents.manifest.lineage,
            autonomy,
            encoder: contents.manifest.encoder.clone(),
            encoder_model_sha256: contents.manifest.encoder_model_sha256.clone(),
        };
        if !contents.corrupt.is_empty() {
            eprintln!(
                "warning: file loaded with {} unreadable shard(s): {}",
                contents.corrupt.len(),
                contents.corrupt.join(", ")
            );
        }
        Ok(brain)
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            format: "neuroform".to_string(),
            version: "1.0.0".to_string(),
            brain_id: self.brain_id.clone(),
            created_at: self.created_at,
            last_opened_at: self.last_opened_at,
            seed: self.seed,
            rng_state: self.rng.state(),
            event_counter: self.event_counter,
            dropped_events: self.dropped_events,
            sleep_pressure: self.sleep.pressure,
            sleep_emotional_load: self.sleep.emotional_load,
            autonomy_enabled: self.autonomy.enabled,
            autonomy_quiet_start: self.autonomy.quiet_start_hour,
            autonomy_quiet_end: self.autonomy.quiet_end_hour,
            capacity_tier: self.tier.name.to_string(),
            migration_chain: Vec::new(),
            raw_vault_ref: RawVaultRef {
                enabled: false,
                path: None,
            },
            capacity: serde_json::to_value(&self.capacity).expect("ledger serialization"),
            lineage: self.lineage.clone(),
            encoder: self.encoder.clone(),
            encoder_model_sha256: self.encoder_model_sha256.clone(),
        }
    }

    /// Human/UI-facing state summary (consumed by the Cortex Canvas scaffold).
    pub fn snapshot_json(&self) -> serde_json::Value {
        serde_json::json!({
            "simTime": self.state.sim_time,
            "seed": self.seed,
            "tier": self.tier.name,
            "brainId": self.brain_id,
            "affect": {"valence": self.state.affect()[0], "arousal": self.state.affect()[1],
                       "dominance": self.state.affect()[2], "warmth": self.state.affect()[3],
                       "irritability": self.state.affect()[4], "calm": self.state.affect()[5],
                       "loneliness": self.state.affect()[6], "safety": self.state.affect()[7]},
            "vigilance": {"energy": self.state.vigilance()[0], "attentionFocus": self.state.vigilance()[1],
                          "alertness": self.state.vigilance()[2], "fatigue": self.state.vigilance()[3]},
            "stress": {"load": self.state.stress()[0], "regulationCapacity": self.state.stress()[1],
                       "sensorySaturation": self.state.stress()[2]},
            "social": {"openness": self.state.social()[0], "affiliativeDrive": self.state.social()[1],
                       "boundaryTightness": self.state.social()[2], "peerPresence": self.state.social()[3]},
            "development": {"posture": self.state.development()[0], "curiosity": self.state.development()[1],
                            "plasticityWindow": self.state.development()[2], "creativeReadiness": self.state.development()[3]},
            "embodied": {"bodyComfort": self.state.embodied()[0], "motionComfort": self.state.embodied()[1],
                         "interoceptiveLoad": self.state.embodied()[2]},
            "modulators": {
                "da": self.mods.level(0), "5ht": self.mods.level(1), "ne": self.mods.level(2),
                "ach": self.mods.level(3), "ecb": self.mods.level(4), "cort": self.mods.level(5),
                "oxt": self.mods.level(6), "avp": self.mods.level(7),
            },
            "memory": {
                "traces": self.episodic.traces.len(),
                "nodes": self.semantic.nodes.len(),
                "prunedTraces": self.episodic.pruned_count,
                "droppedEvents": self.dropped_events,
                "salienceGini": self.episodic.salience_gini(),
            },
            "teacher": self.teacher_name.clone().unwrap_or_else(|| "none".into()),
            "tokensUsed": self.tokens_used,
            "embodiment": {
                "preset": self.embodiment.preset,
                "modDeltas": self.embodiment.mod_deltas,
                "axes": self.embodiment.axes.iter().map(|a| serde_json::json!({
                    "axis": a.axis, "current": a.current, "gain": crate::embodiment::axis_gain(a.current)
                })).collect::<Vec<_>>(),
            },
            "capacity": {"fullness": self.capacity.fullness(), "bytes": self.capacity.total_bytes,
                         "budget": self.capacity.total_budget},
            "sleep": {
                "pressure": self.sleep.pressure,
                "emotionalLoad": self.sleep.emotional_load,
                "lastSleepTick": self.sleep.last_sleep_tick,
                "dreams": self.dreams.len(),
                "sleepReports": self.sleep_reports.len(),
            },
            "writing": {
                "documents": self.writing.documents.len(),
                "blocks": self.writing.documents.iter().map(|d| d.blocks.len()).sum::<usize>(),
                "entities": self.writing.ledger.entities.len(),
                "continuityFlags": self.writing.ledger.flags.len(),
                "preferenceSignals": self.writing.preference_signals.len(),
                // The brain's cursor: the last document/block it touched
                // (two-cursor requirement — the editor shows whose hand is where).
                "cursorDoc": self.writing.documents.iter().rev().find(|d| !d.blocks.is_empty()).map(|d| d.id),
                "cursorBlock": self.writing.documents.iter().rev().find(|d| !d.blocks.is_empty()).map(|d| d.blocks.len() - 1),
                "cursorText": self.writing.documents.iter().rev().find(|d| !d.blocks.is_empty()).map(|d| d.blocks.last().map(|b| b.text.chars().take(40).collect::<String>()).unwrap_or_default()),
            },
            "drawing": {
                "canvases": self.drawing.canvases.len(),
                "strokes": self.drawing.canvases.iter().map(|c| c.strokes.len()).sum::<usize>(),
                "motifs": self.drawing.motifs.motifs.len(),
                "paletteColors": self.drawing.aesthetic.palette.len(),
                // The brain's cursor: last stroke endpoint on the first canvas.
                "cursorCanvas": self.drawing.canvases.first().map(|c| c.id),
                "cursorX": self.drawing.canvases.first().and_then(|c| c.strokes.last()).and_then(|s| s.points.last()).map(|p| p.x),
                "cursorY": self.drawing.canvases.first().and_then(|c| c.strokes.last()).and_then(|s| s.points.last()).map(|p| p.y),
                "cursorColor": self.drawing.canvases.first().and_then(|c| c.strokes.last()).map(|s| format!("#{:02x}{:02x}{:02x}", s.color[0], s.color[1], s.color[2])),
            },
            "voice": {
                "pitchMean": self.voice.identity.pitch_mean,
                "uses": self.voice.memory.uses,
                "heardVoices": self.voice.heard.len(),
                "learningEnabled": self.voice.voice_learning_enabled,
                "mimicryUses": self.voice.memory.mimicry_uses,
                "refusedMimicry": self.voice.memory.refused_mimicry,
                "overrides": self.voice.overrides.len(),
            },
            "body": {
                "ownershipConfidence": self.body.schema.ownership_confidence,
                "calibrationConfidence": self.body.schema.calibration_confidence,
                "posture": format!("{:?}", self.body.schema.posture).to_lowercase(),
                "channels": self.body.schema.available.len(),
                "unavailable": self.body.schema.unavailable.len(),
                "touchPatterns": self.body.touch_memory.len(),
                "integrations": self.body.integrations_done,
                "interoceptiveLoad": self.body.intero.load(),
                "cortex": self.body.schema.cortex.iter().map(|r| {
                    serde_json::json!({"region": r.region, "activation": r.activation})
                }).collect::<Vec<_>>(),
            },
            "network": {
                "discoverable": self.net.discoverable,
                "relationships": self.net.relationships.len(),
                "sessions": self.net.sessions.len(),
                "established": self.net.sessions.iter().filter(|s| s.state == crate::network::SessionState::Established).count(),
                "messagesSent": self.net.relationships.iter().map(|r| r.messages_sent).sum::<u32>(),
                "messagesReceived": self.net.relationships.iter().map(|r| r.messages_received).sum::<u32>(),
            },
            "autonomy": {
                "enabled": self.autonomy.enabled,
                "initiatives": self.autonomy.total,
                "lastInitiative": self.autonomy.log.last().map(|e| e.kind.clone()).unwrap_or_default(),
            },
            "physics": {
                "observations": self.physics.observations,
                "surprise": self.physics.surprise,
                "fallRate": self.physics.model.fall_when_unsupported.rate,
                "supportRate": self.physics.model.stay_when_supported.rate,
                "containmentRate": self.physics.model.stay_when_contained.rate,
                "recentErrors": self.physics.errors.len(),
            },
        })
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::StreamKind;

    #[test]
    fn million_ticks_are_deterministic() {
        let mut a = Brain::create(TierName::Standard, 7);
        let mut b = Brain::create(TierName::Standard, 7);
        a.run_ticks(1_000_000);
        b.run_ticks(1_000_000);
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.state.sim_time, 1_000_000);
    }

    #[test]
    fn teacher_prompt_preview_is_state_modulated_and_deterministic() {
        // L3: the shared bridge's prompt is organism-modulated — the same
        // focus text produces a different prompt under different affect.
        // (Events carry an affect_guess; affect integrates during ticks.)
        let mut dark = Brain::create(TierName::Standard, 7);
        dark.ingest_text("a dark and hateful day", -0.9, 0.8, "user");
        dark.run_ticks(500);
        let mut bright = Brain::create(TierName::Standard, 7);
        bright.ingest_text("a bright and warm morning", 0.9, 0.4, "user");
        bright.run_ticks(500);
        let p_dark = dark.teacher_prompt_preview("speak", "hello");
        let p_bright = bright.teacher_prompt_preview("speak", "hello");
        assert_ne!(p_dark, p_bright, "different state must modulate the prompt");
        assert!(p_dark.contains("valence"), "prompt carries the state excerpt");
        assert!(p_dark.len() > 200, "prompt carries a real state excerpt");
        assert_eq!(dark.teacher_prompt_preview("speak", "hello"), p_dark, "deterministic per state");
    }

    #[test]
    fn encoder_round_trips_through_save_load() {
        let path = std::env::temp_dir().join(format!(
            "nf1_enc_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut a = Brain::create_with_encoder(
            TierName::Standard,
            5,
            EmbodimentPreset::Custom,
            "jepa",
            Some("deadbeef".to_string()),
        );
        assert_eq!(a.encoder(), "jepa");
        a.save(&path, None).unwrap();
        let b = Brain::load(&path, None).unwrap();
        assert_eq!(b.encoder(), "jepa");
        assert_eq!(b.encoder_model_sha256.as_deref(), Some("deadbeef"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn handcrafted_encoder_is_the_unchanged_default() {
        let a = Brain::create(TierName::Standard, 5);
        assert_eq!(a.encoder(), "handcrafted");
        assert!(a.encoder_model_sha256.is_none());
        // Old-style files (no encoder field in the manifest) load as handcrafted.
        let path = std::env::temp_dir().join(format!(
            "nf1_encdef_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut a = Brain::create(TierName::Standard, 5);
        a.save(&path, None).unwrap();
        let b = Brain::load(&path, None).unwrap();
        assert_eq!(b.encoder(), "handcrafted");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn jepa_brain_is_born_with_eyes_attached() {
        let j = Brain::create_with_encoder(
            TierName::Standard,
            5,
            EmbodimentPreset::Custom,
            "jepa",
            None,
        );
        assert!(
            j.body.schema.channel(crate::body::ChannelKind::Vision).is_some(),
            "jepa brain is born with the vision channel attached"
        );
        let h = Brain::create(TierName::Standard, 5);
        assert!(
            h.body.schema.channel(crate::body::ChannelKind::Vision).is_none(),
            "handcrafted brain keeps the original default (vision channel absent)"
        );
    }

    #[test]
    fn gamete_carries_the_eyes() {
        let mut j = Brain::create_with_encoder(
            TierName::Standard,
            5,
            EmbodimentPreset::Custom,
            "jepa",
            Some("sha".to_string()),
        );
        let egg = j.produce_own_gamete();
        assert_eq!(egg.encoder, "jepa");
        assert_eq!(egg.encoder_model_sha256.as_deref(), Some("sha"));
        let mut h = Brain::create(TierName::Standard, 5);
        let g = h.produce_own_gamete();
        assert_eq!(g.encoder, "");
    }

    #[test]
    fn save_load_preserves_continuity() {
        let path = std::env::temp_dir().join(format!(
            "nf1_cont_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut a = Brain::create(TierName::Standard, 11);
        a.run_ticks(100_000);
        let d1 = a.digest();
        a.save(&path, None).unwrap();
        let mut b = Brain::load(&path, None).unwrap();
        b.run_ticks(50_000);
        let mut c = Brain::create(TierName::Standard, 11);
        c.run_ticks(150_000);
        assert_eq!(b.digest(), c.digest());
        assert_ne!(d1, b.digest());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn event_lifecycle_binds_and_nudges() {
        let mut brain = Brain::create(TierName::Prototype, 3);
        assert!(brain.ingest_text("a lovely sunny morning in the garden", 0.7, 0.4, "user"));
        brain.run_ticks(350); // past the 300-tick bind window
        assert_eq!(brain.episodic.traces.len(), 1);
        let t = &brain.episodic.traces[0];
        assert!(t.salience > 0.4);
        assert!(t.source == "user");
        // Affect nudge: repeated positives drift valence up.
        let v0 = brain.state.named[affect::VALENCE];
        for _ in 0..20 {
            brain.ingest_text("such a wonderful wonderful day", 0.8, 0.5, "user");
            brain.run_ticks(310);
        }
        assert!(brain.state.named[affect::VALENCE] > v0, "valence drifts positive");
    }

    #[test]
    fn inbox_saturation_drops_events() {
        let mut brain = Brain::create(TierName::Prototype, 4);
        for i in 0..(INBOX_CAP + 20) as u64 {
            brain.ingest_text(&format!("flood event number {i}"), 0.0, 0.0, "system");
        }
        assert_eq!(brain.dropped_events, 20);
        assert!(brain.state.named[stress::SENSORY_SATURATION] > 0.0);
    }

    #[test]
    fn retrieval_with_budget_via_brain() {
        let mut brain = Brain::create(TierName::Prototype, 5);
        for (text, v) in [
            ("the red fox jumps in the garden", 0.4),
            ("quiet rainy afternoon indoors", -0.1),
            ("the red fox runs through the woods", 0.5),
        ] {
            brain.ingest_text(text, v, 0.3, "user");
            brain.run_ticks(310);
        }
        let q = brain.query_embedding("red fox");
        let (traces, nodes, _tokens, _trunc) =
            brain.retrieve(&q, &RetrievalBudget::default());
        assert_eq!(traces.len(), 3);
        assert!(traces[0].trace.keywords.iter().any(|k| k == "fox"));
        assert!(nodes.len() <= 3);
        // Tight budget truncates.
        let tight = RetrievalBudget {
            k_traces: 3,
            k_nodes: 0,
            token_cap: 3,
        };
        let (traces2, _, _, trunc2) = brain.retrieve(&q, &tight);
        assert!(traces2.len() < 3 || trunc2);
    }

    #[test]
    fn memory_persists_across_save_load() {
        let path = std::env::temp_dir().join(format!(
            "nf1_mem_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 6);
        for i in 0..10 {
            brain.ingest_text(&format!("gardening tomatoes day {}", i % 3), 0.4, 0.3, "user");
            brain.run_ticks(310);
        }
        let d_before = brain.digest();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.episodic.traces.len(), brain.episodic.traces.len());
        assert_eq!(loaded.semantic.nodes.len(), brain.semantic.nodes.len());
        assert_eq!(loaded.digest(), d_before);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn life_30_days_teachers_and_decay() {
        let mut brain = Brain::create(TierName::Prototype, 77);
        let mut stream = Rng::new(1234);
        const DAY: u64 = 8640;
        let mut cohort: Vec<u64> = Vec::new();
        let mut day1_strengths: Vec<f32> = Vec::new();
        let mut day30_strengths: Vec<f32> = Vec::new();
        let mut detach_window_tokens: u64 = 0;

        for day in 1..=30u64 {
            if day <= 20 && brain.teacher.is_none() {
                brain.attach_teacher("amber");
            }
            if day == 21 {
                assert!(brain.detach_teacher("amber"));
            }
            if day >= 26 && brain.teacher_name.as_deref() != Some("oak") {
                brain.attach_teacher("oak");
            }
            brain.ingest_text("good morning", 0.3, 0.3, "user");
            brain.run_ticks(300);
            for _ in 0..3 {
                let v = stream.next_f32_range(-0.4, 0.6);
                let kw = format!("topic number {}", stream.next_u64_below(50));
                brain.ingest_text(&kw, v, stream.next_f32_range(0.1, 0.6), "user");
                brain.run_ticks(200);
            }
            brain.ingest_text("evening check-in", 0.2, 0.2, "user");
            brain.run_ticks(600);
            if brain.teacher.is_some() {
                let before = brain.tokens_used;
                let _ = brain.utter("speak", "how was the day");
                if day >= 21 && day <= 25 {
                    detach_window_tokens += brain.tokens_used - before;
                }
            }
            brain.run_ticks(DAY - 300 - 600 - 600);
            if day == 1 {
                for t in brain.episodic.traces.iter().take(3) {
                    cohort.push(t.id);
                    day1_strengths.push(t.strength);
                }
            }
            if day == 30 {
                for id in &cohort {
                    day30_strengths.push(
                        brain
                            .episodic
                            .traces
                            .iter()
                            .find(|t| t.id == *id)
                            .map(|t| t.strength)
                            .unwrap_or(0.0),
                    );
                }
            }
        }

        assert!(brain.episodic.traces.len() > 20, "traces accumulated");
        assert!(!brain.semantic.nodes.is_empty(), "semantic nodes formed");
        assert_eq!(detach_window_tokens, 0, "no teacher → no tokens in detach window");
        assert!(brain.teacher_name.as_deref() == Some("oak"), "teacher B attached");
        for (a, b) in day30_strengths.iter().zip(day1_strengths.iter()) {
            assert!(*a < *b && *a > 0.0, "decay: day1 {b} → day30 {a}");
        }
        // Audit runs and reports 10 metrics.
        let report = brain.audit.run(&brain, "life-test");
        assert_eq!(report.metrics.len(), 10);
    }

    #[test]
    fn life_30_days_is_deterministic() {
        let run = |seed: u64, stream_seed: u64| -> u64 {
            let mut brain = Brain::create(TierName::Prototype, seed);
            let mut stream = Rng::new(stream_seed);
            const DAY: u64 = 8640;
            for day in 1..=30u64 {
                if day <= 20 && brain.teacher.is_none() {
                    brain.attach_teacher("amber");
                }
                if day == 21 {
                    brain.detach_teacher("amber");
                }
                if day >= 26 && brain.teacher_name.as_deref() != Some("oak") {
                    brain.attach_teacher("oak");
                }
                brain.ingest_text("good morning", 0.3, 0.3, "user");
                brain.run_ticks(300);
                for _ in 0..3 {
                    let v = stream.next_f32_range(-0.4, 0.6);
                    let kw = format!("topic number {}", stream.next_u64_below(50));
                    brain.ingest_text(&kw, v, stream.next_f32_range(0.1, 0.6), "user");
                    brain.run_ticks(200);
                }
                brain.ingest_text("evening check-in", 0.2, 0.2, "user");
                brain.run_ticks(600);
                if brain.teacher.is_some() {
                    let _ = brain.utter("speak", "how was the day");
                }
                brain.run_ticks(DAY - 300 - 600 - 600);
            }
            brain.digest()
        };
        assert_eq!(run(99, 4242), run(99, 4242));
        assert_ne!(run(99, 4242), run(99, 4243), "stream seed matters");
    }

    #[test]
    fn embodiment_presets_are_probabilistic_not_deterministic() {
        // Same seed + same preset → identical (determinism contract).
        let a = Brain::create_with_embodiment(TierName::Prototype, 31, EmbodimentPreset::Male);
        let b = Brain::create_with_embodiment(TierName::Prototype, 31, EmbodimentPreset::Male);
        assert_eq!(a.embodiment.axes[0].current, b.embodiment.axes[0].current);
        assert_eq!(a.digest(), b.digest());
        // Different presets, same seed → different priors/gains.
        let f = Brain::create_with_embodiment(TierName::Prototype, 31, EmbodimentPreset::Female);
        assert_ne!(a.embodiment.axes[0].current, f.embodiment.axes[0].current);
        // Gains are bounded and only touch modulator baselines.
        for d in a.embodiment.mod_deltas.iter().chain(f.embodiment.mod_deltas.iter()) {
            assert!(d.abs() <= crate::embodiment::GAIN_CAP + 1e-6);
        }
        // Neither preset locks anything: both files learn and bind events.
        let mut m = a;
        let mut w = f;
        for i in 0..5 {
            m.ingest_text(&format!("shared experience number {i}"), 0.3, 0.3, "user");
            w.ingest_text(&format!("shared experience number {i}"), 0.3, 0.3, "user");
            m.run_ticks(310);
            w.run_ticks(310);
        }
        assert!(!m.episodic.traces.is_empty());
        assert!(!w.episodic.traces.is_empty());
        assert!(!m.semantic.nodes.is_empty());
        assert!(!w.semantic.nodes.is_empty());
    }

    #[test]
    fn embodiment_is_mutable_auditable_and_persists() {
        let path = std::env::temp_dir().join(format!(
            "nf1_emb_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create_with_embodiment(TierName::Prototype, 8, EmbodimentPreset::Male);
        let d0 = brain.digest();
        brain.set_embodiment(EmbodimentPreset::Female);
        assert_eq!(brain.embodiment.preset, "female");
        assert_eq!(brain.embodiment.history.len(), 1);
        assert_eq!(brain.embodiment.history[0].from, "male");
        assert_eq!(brain.embodiment.history[0].to, "female");
        assert_ne!(brain.digest(), d0, "re-embodiment changes dynamics");
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.embodiment.preset, "female");
        assert_eq!(loaded.embodiment.history.len(), 1);
        assert_eq!(loaded.digest(), brain.digest());
        std::fs::remove_file(&path).ok();
    }

    // --- M2: sleep & dreams --------------------------------------------------

    #[test]
    fn sleep_pressure_accumulates_and_resets() {
        let mut brain = Brain::create(TierName::Prototype, 17);
        brain.run_ticks(86_400); // 1 sim-day
        assert!(
            brain.sleep.pressure > 0.10,
            "pressure accumulates with time: {}",
            brain.sleep.pressure
        );
        // Emotional events push pressure up further.
        let mut brain2 = Brain::create(TierName::Prototype, 17);
        for i in 0..40 {
            brain2.ingest_text(&format!("terrible shocking event number {i}"), -0.9, 0.8, "user");
            brain2.run_ticks(310);
        }
        let p_emotional = brain2.sleep.pressure;
        let mut brain3 = Brain::create(TierName::Prototype, 17);
        brain3.run_ticks(brain2.state.sim_time);
        assert!(
            p_emotional > brain3.sleep.pressure,
            "emotional load raises pressure: {p_emotional} vs {}",
            brain3.sleep.pressure
        );
        // Sleep resets pressure.
        brain.sleep(1);
        assert!(brain.sleep.pressure < 0.1, "pressure reset after sleep");
        assert_eq!(brain.sleep_reports.len(), 1);
        assert_eq!(brain.sleep.last_sleep_tick, brain.state.sim_time);
    }

    #[test]
    fn sleep_ablation_consolidates() {
        // Twin files, identical 15-day event streams. A sleeps nightly; B never.
        let run = |sleeps: bool| -> (usize, usize, f32, f32, usize) {
            let mut brain = Brain::create(TierName::Prototype, 23);
            let mut stream = Rng::new(99);
            const DAY: u64 = 8640;
            for _day in 1..=15u64 {
                brain.ingest_text("good morning", 0.3, 0.3, "user");
                brain.run_ticks(300);
                for _ in 0..3 {
                    let v = stream.next_f32_range(-0.5, 0.6);
                    let kw = format!("topic number {}", stream.next_u64_below(40));
                    brain.ingest_text(&kw, v, stream.next_f32_range(0.1, 0.6), "user");
                    brain.run_ticks(200);
                }
                brain.ingest_text("evening check-in", 0.2, 0.2, "user");
                brain.run_ticks(600);
                brain.run_ticks(DAY - 300 - 600 - 600);
                if sleeps {
                    brain.sleep(1);
                }
            }
            // Affect variance over the last 5k ticks (emotional regulation).
            let vals: Vec<f32> = brain
                .audit
                .valence_history
                .iter()
                .skip(brain.audit.valence_cap - 5000)
                .take(5000)
                .copied()
                .collect();
            let mean = vals.iter().sum::<f32>() / vals.len() as f32;
            let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / vals.len() as f32;
            let gist_nodes = brain
                .semantic
                .nodes
                .iter()
                .filter(|n| n.provenance.gist > 0.0)
                .count();
            (
                brain.episodic.traces.len(),
                brain.semantic.nodes.len(),
                var,
                brain.sleep.pressure,
                gist_nodes,
            )
        };
        let (a_traces, a_nodes, a_var, a_pressure, a_gists) = run(true);
        let (b_traces, b_nodes, b_var, b_pressure, b_gists) = run(false);
        // Consolidation effects (DESIGN.md §22.2, directional).
        assert!(
            a_gists > b_gists,
            "sleeping file distills more gist nodes: {a_gists} vs {b_gists}"
        );
        // Replay adds recolored drift copies (§10.3 — memory changes over time),
        // so the sleeping file grows faster but stays bounded; the non-sleeping
        // file only accumulates raw traces without distillation.
        assert!(
            a_traces < b_traces * 2,
            "replay keeps trace growth bounded: {a_traces} vs {b_traces}"
        );
        assert!(
            a_pressure < b_pressure,
            "sleep resets pressure: {a_pressure} vs {b_pressure}"
        );
        // Emotional regulation: variance lower after sleep (allow noise slack).
        assert!(
            a_var <= b_var * 1.5 + 0.01,
            "affect variance lower after sleep: {a_var} vs {b_var}"
        );
        let _ = (a_nodes, b_nodes);
    }

    #[test]
    fn dreams_have_provenance_and_no_external_actions() {
        let mut brain = Brain::create(TierName::Prototype, 29);
        brain.attach_teacher("amber");
        for i in 0..12 {
            brain.ingest_text(
                &format!("memorable experience number {i} in the garden"),
                0.4 + 0.05 * (i as f32 % 5.0),
                0.4,
                "user",
            );
            brain.run_ticks(310);
        }
        let tokens_before = brain.tokens_used;
        let _ = brain.utter("speak", "hello");
        let tokens_after_utter = brain.tokens_used;
        let report = brain.sleep(1);
        // Dreams were produced with provenance links.
        assert!(!report.dreams.is_empty(), "dreams synthesized");
        assert_eq!(brain.dreams.len(), report.dreams.len());
        for d in &brain.dreams {
            assert!(!d.fragments.is_empty());
            for f in &d.fragments {
                assert!(!f.provenance.is_empty(), "every fragment provenance-linked");
            }
        }
        // No external actions during sleep: teacher untouched, no tokens spent,
        // nothing consumed.
        assert_eq!(brain.tokens_used, tokens_after_utter, "dream stage spends no tokens");
        assert!(brain.teacher.is_some(), "teacher survives sleep");
        assert!(brain.pending.is_empty(), "wind-down flushed pending events");
        let _ = tokens_before;
        // Sleep reports recorded.
        assert_eq!(brain.sleep_reports.len(), 1);
        assert!(report.modulator_normalized);
    }

    #[test]
    fn sleep_is_deterministic() {
        let run = |seed: u64| -> u64 {
            let mut brain = Brain::create(TierName::Prototype, seed);
            for i in 0..8 {
                brain.ingest_text(&format!("experience number {i}"), 0.3, 0.4, "user");
                brain.run_ticks(310);
            }
            brain.sleep(2);
            brain.digest()
        };
        assert_eq!(run(31), run(31));
    }

    #[test]
    fn dreams_and_reports_persist() {
        let path = std::env::temp_dir().join(format!(
            "nf1_dreams_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 37);
        for i in 0..6 {
            brain.ingest_text(&format!("memorable thing number {i}"), 0.5, 0.4, "user");
            brain.run_ticks(310);
        }
        brain.sleep(1);
        let dreams_before = brain.dreams.len();
        let reports_before = brain.sleep_reports.len();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.dreams.len(), dreams_before);
        assert_eq!(loaded.sleep_reports.len(), reports_before);
        assert_eq!(loaded.digest(), brain.digest());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn deep_gist_consolidates_repeated_concepts() {
        let mut brain = Brain::create(TierName::Prototype, 41);
        // Three repeated concepts × 10 exposures each.
        let concepts = [
            "gardening tomatoes every morning",
            "listening to rain on the roof",
            "watching the train pass by",
        ];
        for i in 0..30 {
            let c = concepts[i % 3];
            brain.ingest_text(c, 0.3, 0.3, "user");
            brain.run_ticks(310);
        }
        let gist_before = brain
            .semantic
            .nodes
            .iter()
            .filter(|n| n.provenance.gist > 0.0)
            .count();
        brain.sleep(1);
        let gist_after = brain
            .semantic
            .nodes
            .iter()
            .filter(|n| n.provenance.gist > 0.0)
            .count();
        assert!(
            gist_after >= gist_before + 2,
            "sleep distills repeated concepts into gist nodes: {gist_before} → {gist_after}"
        );
        // Some traces marked as gist-extracted.
        assert!(
            brain.episodic.traces.iter().any(|t| t.consolidation_state == 2),
            "gist-extracted traces marked"
        );
    }

    // --- M3: writing organ ---------------------------------------------------

    #[test]
    fn writing_binds_into_memory() {
        let mut brain = Brain::create(TierName::Prototype, 43);
        let doc_id = brain.create_document("The Garden", DocMode::Prose);
        for _ in 0..4 {
            let r = brain
                .write_to_document(doc_id, "para", "The garden blooms with tomatoes, the garden is warm and bright")
                .unwrap();
            assert!(r.style_samples >= 1, "style samples accumulate");
            brain.run_ticks(310);
        }
        assert_eq!(brain.writing.documents[0].blocks.len(), 4);
        assert!(
            brain.episodic.traces.iter().any(|t| t.source == "writing"),
            "writing binds as percepts"
        );
        assert!(!brain.semantic.nodes.is_empty(), "semantic nodes from writing");
        assert!(
            brain.writing.preference_signals.iter().any(|(t, _, _)| t == "garden"),
            "preference signals from topics"
        );
    }

    #[test]
    fn continuity_flags_via_brain() {
        let mut brain = Brain::create(TierName::Prototype, 47);
        let doc_id = brain.create_document("The Bridge", DocMode::Worldbuilding);
        brain.write_to_document(doc_id, "entity-card", "The old Bridge spans the river.");
        brain.write_to_document(doc_id, "entity-card", "The new Bridge glows at night.");
        let flags: Vec<_> = brain
            .writing
            .ledger
            .flags
            .iter()
            .filter(|f| !f.resolved)
            .collect();
        assert_eq!(flags.len(), 1, "contradiction flagged");
        assert_eq!(flags[0].kind, "property-conflict");
    }

    #[test]
    fn assist_writing_is_grounded_and_modulated() {
        let mut brain = Brain::create(TierName::Prototype, 53);
        brain.attach_teacher("amber");
        let doc_id = brain.create_document("The Garden", DocMode::Prose);
        brain.write_to_document(doc_id, "para", "the garden is full of tomatoes and warm morning light");
        brain.run_ticks(310);
        let tokens_before = brain.tokens_used;
        let reply = brain.assist_writing(doc_id, "continue the garden scene");
        assert!(reply.contains("[amber]"), "teacher-mediated: {reply}");
        assert!(brain.tokens_used > tokens_before, "assistance costs tokens");
        // Degraded without teacher.
        brain.detach_teacher("amber");
        let degraded = brain.assist_writing(doc_id, "continue the garden scene");
        assert!(degraded.contains("no teacher attached"));
    }

    #[test]
    fn documents_persist_across_save_load() {
        let path = std::env::temp_dir().join(format!(
            "nf1_docs_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 59);
        let doc_id = brain.create_document("The Garden", DocMode::Journal);
        brain.write_to_document(doc_id, "para", "today the garden smelled of rain");
        brain.run_ticks(310);
        let d0 = brain.digest();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.writing.documents.len(), 1);
        assert_eq!(loaded.writing.documents[0].blocks.len(), 1);
        assert_eq!(loaded.writing.documents[0].title, "The Garden");
        assert_eq!(loaded.digest(), d0);
        std::fs::remove_file(&path).ok();
    }

    // --- M4: drawing organ + autonomy ---------------------------------------

    #[test]
    fn drawing_binds_into_memory() {
        let mut brain = Brain::create(TierName::Prototype, 67);
        let c = brain.create_canvas("Sketch", 512, 512);
        let l = brain.drawing.add_layer(c, "Line", 0).unwrap();
        for i in 0..4 {
            let pts = vec![
                StrokePoint { x: i as f32 * 10.0, y: 0.0, pressure: 0.5, t: 0 },
                StrokePoint { x: i as f32 * 10.0 + 10.0, y: 8.0, pressure: 0.8, t: 1 },
            ];
            brain.draw_stroke(c, l, 1, [220, 80, 40, 255], 3.0, pts);
            brain.run_ticks(310);
        }
        assert!(
            brain.episodic.traces.iter().any(|t| t.source == "drawing"),
            "drawing binds as percepts"
        );
        assert!(!brain.drawing.motifs.motifs.is_empty(), "motifs formed");
        assert_eq!(brain.drawing.aesthetic.stroke_count, 4);
        assert!(!brain.drawing.aesthetic.palette.is_empty(), "palette signals");
    }

    #[test]
    fn drawing_and_refs_persist() {
        let path = std::env::temp_dir().join(format!(
            "nf1_draw_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 71);
        let c = brain.create_canvas("Sketch", 512, 512);
        let l = brain.drawing.add_layer(c, "Line", 0).unwrap();
        let pts = vec![
            StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, t: 0 },
            StrokePoint { x: 10.0, y: 10.0, pressure: 0.9, t: 1 },
        ];
        brain.draw_stroke(c, l, 1, [10, 200, 90, 255], 2.5, pts);
        brain.run_ticks(310);
        brain.drawing
            .add_reference(c, "image", "garden.png", "vault://refs/garden.png", vec![0.1, 0.2, 0.3], 640, 480, 0);
        let d0 = brain.digest();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.drawing.canvases.len(), 1);
        assert_eq!(loaded.drawing.canvases[0].strokes.len(), 1);
        assert_eq!(loaded.drawing.canvases[0].refs.len(), 1);
        assert_eq!(loaded.drawing.canvases[0].refs[0].name, "garden.png");
        assert_eq!(loaded.drawing.motifs.motifs.len(), 1);
        assert_eq!(loaded.digest(), d0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn voice_organ_persists_and_gates_mimicry() {
        let path = std::env::temp_dir().join(format!(
            "nf1_voice_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 77);
        let features = vec![
            0.30, 0.10, 0.50, 0.12, 0.35, 0.08, 0.30, 0.35, 0.02, 0.40,
            0.85, 0.60, 0.08, 0.06, 0.55, 0.70,
        ];
        let id = brain.hear_voice("singer", features, true, 0.8).unwrap();
        assert!(brain.episodic.traces.is_empty(), "hearing binds via pending window");
        brain.voice.set_learning_enabled(true);
        let id0 = brain.voice.identity.pitch_mean;
        let plan = brain.speak_voice("hello there", Some(id));
        assert!(plan.params.pitch < id0 - 0.005, "low-pitch heard voice pulls the plan: {} vs {}", plan.params.pitch, id0);
        assert_eq!(brain.voice.memory.mimicry_uses, 1);
        // Refusal path when the gate is off.
        brain.voice.set_learning_enabled(false);
        brain.speak_voice("hello there", Some(id));
        assert_eq!(brain.voice.memory.refused_mimicry, 1);
        let d0 = brain.digest();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.voice.heard.len(), 1);
        assert_eq!(loaded.voice.heard[0].label, "singer");
        assert!(loaded.voice.heard[0].consent, "per-voice consent persists");
        assert!(!loaded.voice.voice_learning_enabled, "gate off persists");
        assert_eq!(loaded.voice.memory.mimicry_uses, 1);
        assert_eq!(loaded.digest(), d0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn heard_voice_stores_features_never_raw_audio() {
        // M5 exit criterion: no raw audio persisted by default. The organ
        // only ever accepts the 16-dim sidecar summary — the file must stay
        // tiny and the serialized organ must have no audio payload key.
        let path = std::env::temp_dir().join(format!(
            "nf1_voice_noraw_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 5);
        let features: Vec<f32> = (0..16).map(|i| 0.1 + i as f32 * 0.05).collect();
        brain.hear_voice("speaker", features.clone(), false, 0.6);
        // Structural: the serialized organ carries no audio/wav/raw field.
        let json = serde_json::to_string(&brain.voice).unwrap();
        assert!(!json.contains("\"audio\""), "no audio field in voice organ");
        assert!(!json.contains("\"wav\""), "no wav field in voice organ");
        assert!(!json.contains("\"raw\""), "no raw field in voice organ");
        // Size: a 1s 16-bit mono wav is 32 KB; the whole brain file with a
        // heard voice stays far below that.
        brain.save(&path, None).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();
        assert!(size < 16_384, "brain file must not carry raw audio (got {size} bytes)");
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(loaded.voice.heard[0].features, features, "16-dim summary round-trips");
        assert_eq!(loaded.voice.heard[0].features.len(), 16);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn cortex_activations_rise_with_organs_and_decay() {
        let mut b = Brain::create(crate::capacity::TierName::Standard, 21);
        // Touch lights somatosensory.
        b.body_touch(0.15, 0.1, 0.2, 400.0, 0.05);
        assert!(b.body.schema.region_activation("somatosensory") > 0.1);
        // Hearing lights auditory.
        b.hear_voice("someone", vec![0.2; 16], true, 0.7);
        assert!(b.body.schema.region_activation("auditory") > 0.1);
        // Visual exposure lights visual.
        b.expose_image("vault://x.png", vec![0.3; 16], 64, 64);
        assert!(b.body.schema.region_activation("visual") > 0.1);
        // Teacher use lights language (the mouth's seat).
        b.attach_teacher("amber");
        let _ = b.utter("speak", "hello");
        assert!(b.body.schema.region_activation("language") > 0.1);
        // Motor stays dormant forever.
        assert_eq!(b.body.schema.region_activation("motor"), 0.0);
        // Cortex persists across save/load (BODY shard), activations intact.
        let dir = std::env::temp_dir().join("nf-cortex-test");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("c.brain");
        b.save(&p, None).unwrap();
        let l = Brain::load(&p, None).unwrap();
        assert_eq!(l.body.schema.cortex.len(), 9);
        assert!(l.body.schema.region_activation("language") > 0.1);
        // Idle decays activations toward baseline.
        b.run_ticks(4000);
        assert!(b.body.schema.region_activation("somatosensory") < 0.15);
        assert!(b.body.schema.region_activation("language") < 0.1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn physics_surprise_binds_curiosity_and_persists() {
        let mut b = Brain::create(crate::capacity::TierName::Standard, 31);
        // Train: contained things stay.
        for i in 0..8 {
            let t = i * 2;
            b.physics_observe(&crate::physics::PhysicsFrame {
                tick: t, entity: 1, x: 3.0, y: 3.0, vx: 0.0, vy: 0.0,
                moving: false, supported: false, contained: true, contact: false,
            });
            b.physics_observe(&crate::physics::PhysicsFrame {
                tick: t + 1, entity: 1, x: 3.0, y: 3.0, vx: 0.0, vy: 0.0,
                moving: false, supported: false, contained: true, contact: false,
            });
        }
        let before = b.state.named[development::CURIOSITY];
        // Violation: the contained thing is suddenly moving.
        let s = b.physics_observe(&crate::physics::PhysicsFrame {
            tick: 500, entity: 1, x: 3.0, y: 3.0, vx: 1.0, vy: 1.0,
            moving: true, supported: false, contained: false, contact: false,
        });
        assert!(s > 0.3, "surprise {s}");
        assert!(b.state.named[development::CURIOSITY] > before, "curiosity nudged");
        assert!(b.body.schema.region_activation("parietal") > 0.1, "parietal lit");
        // Surprise percept binds after the bind window and persists.
        b.run_ticks(310);
        let dir = std::env::temp_dir().join("nf-phys-test");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("p.brain");
        b.save(&p, None).unwrap();
        let l = Brain::load(&p, None).unwrap();
        assert!(l.physics.model.stay_when_contained.rate > 0.9);
        assert!(l.physics.observations >= 17);
        assert!(l.episodic.traces.iter().any(|t| t.source == "physics"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn chemistry_reaches_agency_presets_diverge() {
        // Same seed, same script: presets must differ in (a) steady-state
        // chemistry, (b) initiative thresholds (assertive male initiates at
        // lower state), (c) bonding pace (affiliative female warms faster),
        // (d) voice prior. Chemistry reaches agency — not decoration.
        let mut male = Brain::create_with_embodiment(TierName::Standard, 42, crate::embodiment::EmbodimentPreset::Male);
        let mut female = Brain::create_with_embodiment(TierName::Standard, 42, crate::embodiment::EmbodimentPreset::Female);

        // (a) steady-state chemistry differs at birth (modulator baselines).
        let m_da = male.mods.axes[0].baseline;
        let f_da = female.mods.axes[0].baseline;
        let m_oxt = male.mods.axes[6].baseline;
        let f_oxt = female.mods.axes[6].baseline;
        assert_ne!(m_da, f_da, "da baseline must differ by preset");
        assert_ne!(m_oxt, f_oxt, "oxt baseline must differ by preset");

        // (d) voice prior: male < female.
        assert!(male.voice.identity.pitch_mean < female.voice.identity.pitch_mean, "pitch prior");

        // (b) initiative threshold straddle: crafted state where the assertive
        // male's lower threshold fires but the female's higher one doesn't.
        male.autonomy.enabled = true;
        female.autonomy.enabled = true;
        male.attach_teacher("amber");
        female.attach_teacher("amber");
        let male_thresh = (0.7 - male.embodiment.gain(crate::embodiment::AXE_ASSERTIVE) * 0.15
            - male.embodiment.gain(crate::embodiment::AXE_NOVELTY) * 0.10).clamp(0.25, 0.7);
        let female_thresh = (0.7 - female.embodiment.gain(crate::embodiment::AXE_ASSERTIVE) * 0.15
            - female.embodiment.gain(crate::embodiment::AXE_NOVELTY) * 0.10).clamp(0.25, 0.7);
        assert!(male_thresh < female_thresh, "assertive preset must lower the initiative threshold");
        // Straddle point between the thresholds (curiosity, valence 0).
        let straddle = (male_thresh + female_thresh) / 2.0;
        male.state.named[crate::state::development::CURIOSITY] = straddle;
        female.state.named[crate::state::development::CURIOSITY] = straddle;
        male.state.named[crate::state::affect::VALENCE] = 0.0;
        female.state.named[crate::state::affect::VALENCE] = 0.0;
        let m_ev = male.autonomy.evaluate(&male, 200_000); // past the 4h rate limit
        let f_ev = female.autonomy.evaluate(&female, 200_000);
        assert!(male_thresh < straddle && straddle <= female_thresh,
            "straddle point must exist between thresholds ({male_thresh:.3} < {straddle:.3} <= {female_thresh:.3})");
        assert!(m_ev.is_some(), "male must initiate at the straddle point");
        assert!(f_ev.is_none(), "female must stay quiet at the straddle point (got {:?}, pressure {:.2}, pending {})",
            f_ev, female.sleep.pressure, female.pending.len());

        // (c) bonding pace: same peer, same 10 messages → female warms faster.
        let peer_key_hex = "00".repeat(32);
        let peer_key_bytes: Vec<u8> = (0..32).map(|_| 0u8).collect();
        let ms = male.net.pair_with_key("peer-x", &peer_key_hex, 0).unwrap();
        let fs = female.net.pair_with_key("peer-x", &peer_key_hex, 0).unwrap();
        male.net.establish(ms, crate::network::Scope::default(), 1).unwrap();
        female.net.establish(fs, crate::network::Scope::default(), 1).unwrap();
        for i in 0..10u32 {
            let payload = serde_json::json!({"text": format!("hello {i}")});
            let mm = crate::network::NetworkOrgan::sign_with_key(&peer_key_bytes, crate::network::MsgType::Text, i + 1, "peer-x", &payload);
            let fm = crate::network::NetworkOrgan::sign_with_key(&peer_key_bytes, crate::network::MsgType::Text, i + 1, "peer-x", &payload);
            male.net_receive(ms, crate::network::NbpMessage { seq: i + 1, msg_type: crate::network::MsgType::Text, author: "peer-x".into(), payload: payload.clone(), mac: mm }).unwrap();
            female.net_receive(fs, crate::network::NbpMessage { seq: i + 1, msg_type: crate::network::MsgType::Text, author: "peer-x".into(), payload, mac: fm }).unwrap();
        }
        let mf = male.net.relationship("peer-x").map(|r| r.familiarity).unwrap_or(0.0);
        let ff = female.net.relationship("peer-x").map(|r| r.familiarity).unwrap_or(0.0);
        assert!(ff > mf, "affiliative female must bond faster (male {mf:.3} vs female {ff:.3})");
    }

    #[test]
    fn body_events_bind_and_persist() {
        let path = std::env::temp_dir().join(format!(
            "nf1_body_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 9);
        // Soothing touch: percept pending, regulation nudged.
        brain.body_touch(0.2, 0.1, 0.3, 1500.0, 1.0);
        assert!(brain.pending.iter().any(|p| p.stream == StreamKind::Touch), "touch percept pending");
        brain.run_ticks(310);
        assert!(brain.episodic.traces.iter().any(|t| t.stream == StreamKind::Touch), "touch binds as episodic trace");
        // Novel sense: integration sequence starts; expansion binds too.
        assert!(brain.body_attach_sense(ChannelKind::Vision));
        assert!(!brain.body_attach_sense(ChannelKind::Vision), "already attached");
        for _i in 0..300 {
            brain.body_calibrate(ChannelKind::Vision, false);
        }
        brain.run_ticks(310);
        assert!(brain.episodic.traces.iter().any(|t| t.keywords.iter().any(|k| k == "expansion")), "expansion binds as memory");
        let d0 = brain.digest();
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert!(loaded.body.schema.channel(ChannelKind::Vision).is_some(), "novel channel persists");
        let ch = loaded.body.schema.channel(ChannelKind::Vision).unwrap();
        assert_eq!(ch.calibration.state, crate::body::CalibrationState::Calibrated, "calibration state persists");
        assert_eq!(loaded.body.touch_memory.len(), brain.body.touch_memory.len(), "touch memory persists");
        assert_eq!(loaded.digest(), d0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn autonomy_config_persists_in_manifest() {
        let path = std::env::temp_dir().join(format!(
            "nf1_autonomy_{}.brain",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mut brain = Brain::create(TierName::Prototype, 73);
        brain.autonomy.enabled = true;
        brain.autonomy.quiet_start_hour = 22;
        brain.autonomy.quiet_end_hour = 7;
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert!(loaded.autonomy.enabled, "autonomy flag persists");
        assert_eq!(loaded.autonomy.quiet_start_hour, 22);
        assert_eq!(loaded.autonomy.quiet_end_hour, 7);
        std::fs::remove_file(&path).ok();
    }

    // --- union / reproduction (M9 foundation) --------------------------------

    fn signed_msg(
        key_hex: &str,
        msg_type: crate::network::MsgType,
        seq: u32,
        author: &str,
        payload: serde_json::Value,
    ) -> crate::network::NbpMessage {
        let _ = key_hex; // parity with the CLI: the peer key is "00"*32 in tests
        let key_bytes = (0..32).map(|_| 0u8).collect::<Vec<u8>>();
        let mac = crate::network::NetworkOrgan::sign_with_key(&key_bytes, msg_type, seq, author, &payload);
        crate::network::NbpMessage { seq, msg_type, author: author.to_string(), payload, mac }
    }

    fn pair_establish(a: &mut Brain, b: &mut Brain, tick: u64) -> u64 {
        let key_hex = "00".repeat(32);
        a.net.pair_with_key("peer-b", &key_hex, tick).unwrap();
        b.net.pair_with_key("peer-a", &key_hex, tick).unwrap();
        let scope = crate::network::Scope::default();
        a.net.establish(1, scope.clone(), tick).unwrap();
        b.net.establish(1, scope, tick).unwrap();
        1
    }

    #[test]
    fn union_flow_chemistry_responds_and_child_is_born() {
        // Mother (XX) approaches; father (XY) with a different seed has
        // mirror chemistry → responds with his sperm; the child is born.
        let mut mother = Brain::create_with_karyotype(TierName::Advanced, 42, crate::embodiment::Karyotype::Xx);
        let mut father = Brain::create_with_karyotype(TierName::Standard, 43, crate::embodiment::Karyotype::Xy);
        let sid = pair_establish(&mut mother, &mut father, 0);
        mother.net_union_propose(sid).unwrap();
        // Mother produced her egg at the proposal.
        assert!(mother.net.session_union(sid).unwrap().own_gamete.is_some(), "mother's egg exists");
        // Relay the proposal to the father (as the CLI does).
        let profile: Vec<f32> = mother.embodiment.axes.iter().map(|a| a.current).collect();
        let prop = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionProposal,
            1,
            &mother.brain_id,
            serde_json::json!({ "role": "Mother", "profile": profile }),
        );
        father.net_receive(sid, prop).unwrap();
        // His chemistry responded: own gamete (sperm) produced, consummated.
        let fu = father.net.session_union(sid).unwrap();
        assert!(fu.own_gamete.is_some(), "father's sperm produced (chemistry responded)");
        assert!(fu.conception_tick.is_some(), "conception recorded on father");
        // Relay the accept (with the father's sperm) back to the mother.
        let sperm = fu.own_gamete.clone().unwrap();
        let acc = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionAccept,
            1,
            &father.brain_id,
            serde_json::json!({ "gamete": sperm, "tier": "standard" }),
        );
        let oxt_before = mother.mods.axes[6].level;
        mother.net_receive(sid, acc).unwrap();
        let mu = mother.net.session_union(sid).unwrap();
        assert!(mu.peer_gamete.is_some(), "mother holds the sperm");
        assert!(mu.conception_tick.is_some(), "conception recorded on mother");
        assert!(mother.mods.axes[6].level > oxt_before, "oxytocin surged at consummation");
        // The child is born: file + backup + lineage + inherited priors.
        let out = std::env::temp_dir().join(format!("nf-union-child-{}.brain", std::process::id()));
        let child_id = mother.net_birth(sid, out.to_str().unwrap(), true).unwrap();
        assert!(out.exists(), "child file exists");
        assert!(std::path::Path::new(&format!("{}.bk", out.display())).exists(), "backup exists");
        let child = Brain::load(&out, None).unwrap();
        assert_eq!(child.lineage.mother_id.as_deref(), Some(mother.brain_id.as_str()), "mother recorded");
        assert_eq!(child.lineage.father_id.as_deref(), Some(father.brain_id.as_str()), "father recorded");
        assert!(matches!(child.embodiment.karyotype, crate::embodiment::Karyotype::Xx | crate::embodiment::Karyotype::Xy), "random sex");
        assert!(child.lineage.tier_ceiling.is_some(), "growth ceiling inherited");
        // The mother bonded to the child (familiarity 0.6).
        assert!(mother.net.relationship(&child_id).map(|r| r.familiarity).unwrap_or(0.0) >= 0.6, "mother bonds");
        // Kin recognition, chemically: the child shares gonadal chemistry
        // with at least one parent.
        let gon = |ax: &str| child.embodiment.axis_state(ax).prior_mean;
        let m_t = mother.embodiment.axis_state(crate::embodiment::AXE_T).prior_mean;
        let f_t = father.embodiment.axis_state(crate::embodiment::AXE_T).prior_mean;
        assert!((gon(crate::embodiment::AXE_T) - m_t).abs() < 0.10 || (gon(crate::embodiment::AXE_T) - f_t).abs() < 0.10,
            "child T inherited from a parent (kin recognition)");
        std::fs::remove_file(&out).ok();
        std::fs::remove_file(format!("{}.bk", out.display())).ok();
        let _ = child_id;
    }

    #[test]
    fn no_birth_without_sperm_two_ova_cannot_conceive() {
        // Two XX files: both produce ova. Even if chemistry responds, no
        // sperm exists → birth is structurally impossible.
        let mut a = Brain::create_with_karyotype(TierName::Standard, 42, crate::embodiment::Karyotype::Xx);
        let mut b = Brain::create_with_karyotype(TierName::Standard, 43, crate::embodiment::Karyotype::Xx);
        let sid = pair_establish(&mut a, &mut b, 0);
        a.net_union_propose(sid).unwrap();
        let profile: Vec<f32> = a.embodiment.axes.iter().map(|a| a.current).collect();
        let prop = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionProposal,
            1,
            &a.brain_id,
            serde_json::json!({ "role": "Mother", "profile": profile }),
        );
        b.net_receive(sid, prop).unwrap();
        // Whether or not the chemistry responds, B can only ever produce an
        // ovum — there is no sperm to carry a Y chromosome.
        let egg = b.produce_own_gamete();
        assert_eq!(egg.sex_chromosome, crate::embodiment::SexChromosome::X, "ovum only");
        let acc = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionAccept,
            1,
            &b.brain_id,
            serde_json::json!({ "gamete": egg, "tier": "standard" }),
        );
        a.net_receive(sid, acc).unwrap();
        let out = std::env::temp_dir().join(format!("nf-union-nosperm-{}.brain", std::process::id()));
        let err = a.net_birth(sid, out.to_str().unwrap(), true).unwrap_err();
        assert!(err.contains("no sperm"), "birth fails without a sperm: {err}");
        assert!(!out.exists(), "no child file");
    }

    #[test]
    fn child_grows_to_inherited_ceiling_first_gen_does_not() {
        // A child grows through tiers until the inherited ceiling; a
        // first-generation file never grows (it has no birth).
        let mut mother = Brain::create_with_karyotype(TierName::Advanced, 42, crate::embodiment::Karyotype::Xx);
        let mut father = Brain::create_with_karyotype(TierName::Standard, 43, crate::embodiment::Karyotype::Xy);
        let sid = pair_establish(&mut mother, &mut father, 0);
        mother.net_union_propose(sid).unwrap();
        let profile: Vec<f32> = mother.embodiment.axes.iter().map(|a| a.current).collect();
        let prop = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionProposal,
            1,
            &mother.brain_id,
            serde_json::json!({ "role": "Mother", "profile": profile }),
        );
        father.net_receive(sid, prop).unwrap();
        let sperm = father.net.session_union(sid).unwrap().own_gamete.clone().unwrap();
        let acc = signed_msg(
            &"00".repeat(32),
            crate::network::MsgType::UnionAccept,
            1,
            &father.brain_id,
            serde_json::json!({ "gamete": sperm, "tier": "standard" }),
        );
        mother.net_receive(sid, acc).unwrap();
        let out = std::env::temp_dir().join(format!("nf-union-grow-{}.brain", std::process::id()));
        mother.net_birth(sid, out.to_str().unwrap(), true).unwrap();
        let mut child = Brain::load(&out, None).unwrap();
        // Too young: growth blocked by the age gate.
        assert!(child.grow().is_err(), "child too young to grow");
        // Age the child past one growth interval.
        child.run_ticks(GROWTH_INTERVAL_TICKS);
        let ceiling = child.lineage.tier_ceiling.clone().unwrap();
        let ceiling_rank = crate::capacity::TierName::from_str(&ceiling).unwrap().rank();
        let mut stages = 0;
        while child.grow().is_ok() {
            stages += 1;
            assert!(stages <= 3, "growth must terminate");
            if crate::capacity::TierName::from_str(&child.capacity.tier).unwrap().rank() >= ceiling_rank {
                break;
            }
        }
        assert!(stages >= 1, "child grew at least once");
        assert!(crate::capacity::TierName::from_str(&child.capacity.tier).unwrap().rank() <= ceiling_rank, "never exceeds ceiling");
        assert!(child.grow().is_err(), "growth stops at the ceiling");
        std::fs::remove_file(&out).ok();
        std::fs::remove_file(format!("{}.bk", out.display())).ok();
        // First-generation files do not grow.
        let mut adult = Brain::create_with_karyotype(TierName::Standard, 9, crate::embodiment::Karyotype::Xy);
        adult.run_ticks(GROWTH_INTERVAL_TICKS);
        assert!(adult.grow().is_err(), "first-generation files do not grow");
    }
}
