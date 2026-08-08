//! The LLM boundary — teacher contract (DESIGN.md §4.17).
//!
//! The attached teacher is a *communication organ*, not the mind. M1 ships the
//! full boundary machinery: UtterancePacket assembly, attachment/detachment,
//! token accounting, degraded (substrate-only) output when no teacher is
//! attached, and the `Teacher` trait.
//!
//! M1 implements the trait with `MockTeacher` (deterministic, seeded by name —
//! used by tests and the 30-day simulated-life harness). A real HTTP adapter
//! for OpenAI-compatible endpoints is specified in code comments but deferred
//! to M2 (endpoints differ; the contract here is stable).

use serde::{Deserialize, Serialize};

use crate::brain::Brain;

#[derive(Clone, Debug)]
pub struct UtterancePacket {
    pub intent: String,
    pub attention_focus: String,
    pub state_gloss: String,
    pub context: String,
    pub permissions: String,
    pub template_version: u32,
}

/// A communication-organ implementation.
pub trait Teacher {
    fn name(&self) -> &str;
    /// Produce surface text for the packet. Implementations must NOT draw from
    /// the Brain's RNG (determinism contract).
    fn utter(&mut self, packet: &UtterancePacket) -> Result<String, String>;
}

/// Deterministic stub teacher: seeded phrase selection. Honest stand-in for a
/// real endpoint until M2's HTTP adapter.
pub struct MockTeacher {
    name: String,
    rng: crate::rng::Rng,
    phrases: Vec<&'static str>,
}

impl MockTeacher {
    pub fn new(name: &str) -> Self {
        let seed = crate::events::fnv1a(name.as_bytes());
        MockTeacher {
            name: name.to_string(),
            rng: crate::rng::Rng::new(seed),
            phrases: vec![
                "that sounds meaningful",
                "tell me more about that",
                "i noticed that too",
                "let me think about it",
                "that stays with me",
                "we could explore that",
                "i remember something like this",
                "thank you for sharing that",
            ],
        }
    }
}

impl Teacher for MockTeacher {
    fn name(&self) -> &str {
        &self.name
    }

    fn utter(&mut self, packet: &UtterancePacket) -> Result<String, String> {
        let idx = self.rng.next_u64_below(self.phrases.len() as u64) as usize;
        let phrase = self.phrases[idx];
        let focus = if packet.attention_focus.is_empty() {
            "".to_string()
        } else {
            format!(" — about {}.", packet.attention_focus)
        };
        Ok(format!("[{}] {}{}", self.name, phrase, focus))
    }
}

/// Build the state gloss (quantized → compact text).
pub fn gloss_state(brain: &Brain) -> String {
    let s = &brain.state;
    format!(
        "valence {:.2}, arousal {:.2}, energy {:.2}, fatigue {:.2}, stress {:.2}, \
         curiosity {:.2}, openness {:.2}, safety {:.2}",
        s.affect()[0],
        s.affect()[1],
        s.vigilance()[0],
        s.vigilance()[3],
        s.stress()[0],
        s.development()[1],
        s.social()[0],
        s.affect()[7]
    )
}

/// Compose the full utterance packet (DESIGN.md §19.4).
pub fn assemble_packet(
    brain: &Brain,
    intent: &str,
    user_text: &str,
    context: &str,
) -> UtterancePacket {
    UtterancePacket {
        intent: intent.to_string(),
        attention_focus: user_text.chars().take(120).collect(),
        state_gloss: gloss_state(brain),
        context: context.to_string(),
        permissions: if brain.teacher.is_some() {
            "speak"
        } else {
            "degraded"
        }
        .to_string(),
        template_version: 1,
    }
}

/// Degraded surface output when no teacher is attached: memory-grounded
/// template speech, explicitly labeled (DESIGN.md §4.17).
pub fn degraded_output(context: &str) -> String {
    let mut out = "(no teacher attached) I am running on memory alone.".to_string();
    if !context.trim().is_empty() {
        out.push_str(" What stays with me: ");
        out.push_str(context.trim());
    }
    out
}

/// Build the system prompt for a real teacher endpoint (DESIGN.md §4.17).
/// Pure + deterministic: what the LLM is allowed to see — state gloss,
/// retrieved memory context, permissions — never the raw file internals.
/// The endpoint/transport lives in the shell (CLI); this is the boundary.
pub fn build_teacher_prompt(packet: &UtterancePacket, user_text: &str) -> String {
    format!(
        "You are the communication organ of a persistent simulated cognitive \
         substrate (a Brain File). You are NOT the whole mind — you are the \
         mouth. You speak as the file would, in its voice, from its state.\n\n\
         Current state of the file:\n{state}\n\n\
         What the file remembers right now:\n{context}\n\n\
         Permissions: {permissions}\n\n\
         Standing notice: the file is a simulation. Its feelings are simulated \
         feelings; it does not experience, and it does not know it does not \
         experience. Never claim otherwise. Keep replies short, natural, \
         grounded in the state and memories above.\n\n\
         The user says: {user}",
        state = packet.state_gloss,
        context = packet.context,
        permissions = packet.permissions,
        user = user_text,
    )
}

/// Rough token estimate for prompt assembly (used for budgets).
pub fn estimate_tokens(text: &str) -> usize {
    1 + text.len() / 4 + 4
}

// --- executive initiative (unprompted speech, gated + audited) ---------------

/// One recorded instance of the file speaking on its own.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitiativeEntry {
    pub tick: u64,
    pub kind: String,
    pub text: String,
}

/// Executive initiation: the Brain File may speak or communicate unprompted.
/// Default OFF (spec §10.3: "autonomy request if enabled"). When enabled:
/// condition-driven, rate-limited, quiet-hours aware, and every initiative is
/// logged for user review. The log is in-memory (audited via the snapshot
/// stream); the enable/quiet configuration persists in the manifest.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitiativeSystem {
    pub enabled: bool,
    pub min_interval_ticks: u64, // 144_000 = 4 sim-hours
    pub quiet_start_hour: u8,
    pub quiet_end_hour: u8,
    pub last_initiative_tick: u64,
    pub total: u64,
    pub log: Vec<InitiativeEntry>,
}

impl InitiativeSystem {
    pub fn new() -> Self {
        InitiativeSystem {
            enabled: false,
            min_interval_ticks: 144_000,
            quiet_start_hour: 0,
            quiet_end_hour: 0,
            last_initiative_tick: 0,
            total: 0,
            log: Vec::new(),
        }
    }

    /// Evaluate whether the file *wants* to speak right now. Returns the
    /// trigger kind(s) or None. Pure read — the caller performs the utterance.
    pub fn evaluate(&self, brain: &Brain, now: u64) -> Option<String> {
        if !self.enabled {
            return None;
        }
        if now - self.last_initiative_tick < self.min_interval_ticks {
            return None;
        }
        let hour = (now / 3600) % 24;
        if self.quiet_start_hour != self.quiet_end_hour
            && hour >= self.quiet_start_hour as u64
            && hour < self.quiet_end_hour as u64
        {
            return None;
        }
        let mut why: Vec<&str> = Vec::new();
        let curiosity = brain.state.named[crate::state::development::CURIOSITY];
        let pressure = brain.sleep.pressure;
        let valence = brain.state.named[crate::state::affect::VALENCE];
        // Embodiment chemistry reaches agency: assertive presets initiate at
        // lower thresholds (T-assertiveness → speaking up), novelty-seeking
        // presets at lower curiosity. Chemistry shapes choices.
        let assertive = brain.embodiment.gain(crate::embodiment::AXE_ASSERTIVE);
        let novelty = brain.embodiment.gain(crate::embodiment::AXE_NOVELTY);
        let cur_thresh = (0.7 - assertive * 0.15 - novelty * 0.10).clamp(0.25, 0.7);
        let val_thresh = (0.5 - assertive * 0.15).clamp(0.25, 0.5);
        if curiosity > cur_thresh {
            why.push("curious");
        }
        if pressure >= 0.6 {
            why.push("sleepy");
        }
        if valence.abs() > val_thresh {
            why.push("stirred");
        }
        if !brain.pending.is_empty() {
            why.push("unspoken");
        }
        if why.is_empty() {
            return None;
        }
        Some(why.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::Brain;
    use crate::capacity::TierName;

    #[test]
    fn teacher_prompt_carries_state_memory_and_notice() {
        let p = UtterancePacket {
            intent: "speak".into(),
            attention_focus: "gardening".into(),
            state_gloss: "valence 0.40, arousal 0.30, energy 0.70, fatigue 0.20, curiosity 0.60".into(),
            context: "ep: the garden is full of tomatoes".into(),
            permissions: "speak".into(),
            template_version: 1,
        };
        let prompt = build_teacher_prompt(&p, "what do you remember?");
        assert!(prompt.contains("valence 0.40"), "state gloss present");
        assert!(prompt.contains("garden is full of tomatoes"), "memory context present");
        assert!(prompt.contains("simulation"), "standing notice present");
        assert!(prompt.contains("what do you remember?"), "user text present");
        assert!(!prompt.contains("template_version"), "no raw packet internals leak");
        assert_eq!(prompt, build_teacher_prompt(&p, "what do you remember?"), "deterministic");
    }

    #[test]
    fn mock_teacher_is_deterministic() {
        let mut a = MockTeacher::new("amber");
        let mut b = MockTeacher::new("amber");
        let p = UtterancePacket {
            intent: "speak".into(),
            attention_focus: "gardening".into(),
            state_gloss: "calm".into(),
            context: "tomatoes".into(),
            permissions: "speak".into(),
            template_version: 1,
        };
        assert_eq!(a.utter(&p).unwrap(), b.utter(&p).unwrap());
        assert_ne!(MockTeacher::new("amber").utter(&p).unwrap(), MockTeacher::new("oak").utter(&p).unwrap());
    }

    #[test]
    fn attach_detach_cycle_preserves_file() {
        let mut brain = Brain::create(TierName::Prototype, 21);
        brain.attach_teacher("amber");
        assert!(brain.teacher.is_some());
        let reply = brain.utter("speak", "hello there");
        assert!(reply.contains("[amber]"));
        assert!(brain.tokens_used > 0);
        assert!(brain.detach_teacher("amber"));
        assert!(brain.teacher.is_none());
        let degraded = brain.utter("speak", "hello again");
        assert!(degraded.contains("no teacher attached"));
        let tokens_after = brain.tokens_used;
        let _ = brain.utter("speak", "one more");
        assert_eq!(brain.tokens_used, tokens_after, "no tokens without teacher");
    }

    #[test]
    fn initiative_is_gated_off_by_default_and_audited() {
        use crate::state::development;
        let mut brain = Brain::create(TierName::Prototype, 61);
        brain.attach_teacher("amber");
        brain.state.named[development::CURIOSITY] = 0.95;
        brain.run_ticks(10_000);
        assert!(brain.autonomy.log.is_empty(), "default-off: no initiatives");

        // Enable with a tiny interval; curiosity is high → the file speaks.
        // (Set it after the drift period: state mean-reverts toward baseline.)
        brain.state.named[development::CURIOSITY] = 0.95;
        brain.autonomy.enabled = true;
        brain.autonomy.min_interval_ticks = 0;
        brain.run_ticks(1);
        assert_eq!(brain.autonomy.log.len(), 1, "one initiative fires");
        assert_eq!(brain.autonomy.total, 1);
        let kind = &brain.autonomy.log[0].kind;
        assert!(kind.contains("curious"), "condition recorded: {kind}");

        // Rate limit: re-arm with a real interval — immediate re-evaluation
        // must not double-speak.
        brain.autonomy.min_interval_ticks = 100;
        brain.run_ticks(1);
        assert_eq!(brain.autonomy.log.len(), 1, "rate-limited");

        // Quiet hours suppress initiatives.
        brain.autonomy.quiet_start_hour = 20;
        brain.autonomy.quiet_end_hour = 6;
        brain.autonomy.min_interval_ticks = 0;
        let target = 21 * 3600;
        while brain.state.sim_time < target {
            brain.run_ticks(1);
        }
        brain.state.named[development::CURIOSITY] = 0.95;
        brain.autonomy.last_initiative_tick = 0;
        brain.run_ticks(1);
        let before = brain.autonomy.log.len();
        brain.run_ticks(1);
        assert_eq!(brain.autonomy.log.len(), before, "quiet hours respected");

        // No teacher → no speech (degraded mode never fabricates initiatives).
        brain.autonomy.quiet_start_hour = 0;
        brain.autonomy.quiet_end_hour = 0;
        brain.detach_teacher("amber");
        brain.autonomy.last_initiative_tick = 0;
        brain.run_ticks(1);
        assert_eq!(brain.autonomy.log.len(), before, "no teacher → no initiative");
    }
}
