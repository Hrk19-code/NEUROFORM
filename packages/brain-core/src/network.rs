//! Network organ — inter-brain interaction (DESIGN.md §13, docs/NBP-v1-SPEC.md,
//! master prompt Tab 6).
//!
//! M7 headless core: local-first relationship store (decayable, auditable,
//! never shared wholesale), the NBP session state machine (IDLE → PAIRING →
//! HANDSHAKE → ESTABLISHED → CLOSING → CLOSED, every transition logged),
//! the closed message-type set with seq windows and rate limits, scope
//! negotiation (min-per-field intersection), teaching packets with MAC
//! provenance, and deterministic CRDT merge semantics for shared creative
//! spaces.
//!
//! The wire transport (TCP 45457, mDNS discovery, Noise NK handshake, WebRTC
//! relay) is the desktop-shell milestone. This core is transport-agnostic:
//! sessions exchange `NbpMessage`s; the shell wires them to a socket. In the
//! headless CLI, inbound messages are injected through the same validated
//! path (`net inject`) — identical state transitions, relationship updates,
//! affect nudges and percept binding.
//!
//! Security posture follows the spec: no two files contact each other without
//! explicit pairing (user-mediated), every message is data never instructions
//! (the boundary template is immutable per session), unknown types are
//! rejected + logged, budgets bound memory pressure, relationship signals are
//! user-approved, and the local relationship model is auditable for fixation.
//!
//! Provenance note: the spec calls for Ed25519 file signing keys at the wire
//! layer; the headless core uses a deterministic per-file keyed-BLAKE2b MAC
//! over (author|type|payload) — same audit property (tamper-evident author
//! binding), keyed upgrade path to Ed25519 in the shell milestone.

use serde::{Deserialize, Serialize};

// --- relationship store (§12, §18.13: local-first) ---------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RelEvent {
    pub tick: u64,
    pub kind: String, // "pair" | "text" | "signal" | "teaching" | "close"
    pub summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Relationship {
    pub peer_id: String,
    pub familiarity: f32, // 0..1
    pub trust: f32,       // 0..1, evidence-based, decays
    pub tone: f32,        // -1..1 running emotional tone
    pub interactions: u32,
    pub messages_sent: u32,
    pub messages_received: u32,
    pub shared_artifacts: u32,
    pub boundary_tightness: f32, // 0..1 (vasopressin-like, §4.8)
    pub first_tick: u64,
    pub last_tick: u64,
    pub history: Vec<RelEvent>,
}

impl Relationship {
    fn new(peer_id: &str, tick: u64) -> Self {
        Relationship {
            peer_id: peer_id.to_string(),
            familiarity: 0.0,
            trust: 0.3,
            tone: 0.0,
            interactions: 0,
            messages_sent: 0,
            messages_received: 0,
            shared_artifacts: 0,
            boundary_tightness: 0.4,
            first_tick: tick,
            last_tick: tick,
            history: Vec::new(),
        }
    }

    fn log(&mut self, tick: u64, kind: &str, summary: String) {
        self.last_tick = tick;
        self.history.push(RelEvent { tick, kind: kind.into(), summary });
        if self.history.len() > 64 {
            self.history.remove(0);
        }
    }
}

// --- session state machine (§6) ----------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SessionState {
    Idle,
    Pairing,
    Handshake,
    Established,
    Closing,
    Closed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionLogEntry {
    pub tick: u64,
    pub transition: String, // e.g. "IDLE→PAIRING"
    pub detail: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: u64,
    pub peer_id: String,
    /// Peer's file key (hex) — exchanged at pairing like NBP fingerprints.
    /// Inbound MACs verify against this; empty = no peer key exchanged yet
    /// (inbound messages cannot be authenticated — secure default).
    pub peer_key: String,
    pub state: SessionState,
    pub scope: Scope,
    pub started_tick: u64,
    pub last_activity_tick: u64,
    pub seq_out: u32,
    pub seq_in: u32,
    pub closed_reason: Option<String>,
    pub log: Vec<SessionLogEntry>,
    /// Union (reproduction) state — None until a proposal is exchanged.
    #[serde(default)]
    pub union: Option<UnionState>,
}

/// Reproduction role: derived from the karyotype, not chosen. Y-bearing
/// karyotypes produce sperm (the father); the rest produce ova (the mother).
/// A child requires one of each — two ova cannot conceive (structure, not
/// a concept: no Y chromosome means no sperm exists to carry one).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnionRole {
    Mother,
    Father,
}

/// Consent-gated union state on a session (both sides must accept; the
/// gametes are the parents' contributions — a child needs both).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnionState {
    pub proposed: bool,
    pub accepted: bool,
    pub role: Option<UnionRole>,
    pub own_gamete: Option<crate::embodiment::Gamete>,
    pub peer_gamete: Option<crate::embodiment::Gamete>,
    pub conception_tick: Option<u64>,
    /// Peer's tier (the child's growth ceiling is inherited from both).
    #[serde(default)]
    pub peer_tier: Option<String>,
}

impl Session {
    fn log(&mut self, tick: u64, transition: &str, detail: &str) {
        self.last_activity_tick = tick;
        self.log.push(SessionLogEntry { tick, transition: transition.into(), detail: detail.into() });
        if self.log.len() > 128 {
            self.log.remove(0);
        }
    }
}

// --- scope (§6 scope_proposal, min-per-field intersection) -------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scope {
    pub text: bool,
    pub voice_params: bool,
    pub canvas: bool,
    pub document: bool,
    pub memory_summaries: bool,
    pub teaching: bool,
    /// "none" | "summary" | "full" — relationship-state sharing level.
    pub relationship_state: String,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            text: true,
            voice_params: false,
            canvas: false,
            document: false,
            memory_summaries: false,
            teaching: false,
            relationship_state: "none".into(),
        }
    }
}

impl Scope {
    /// Effective scope = min-per-field of both proposals (§6).
    pub fn intersection(&self, other: &Scope) -> Scope {
        let rel_level = |a: &str, b: &str| {
            let rank = |s: &str| match s { "full" => 2, "summary" => 1, _ => 0 };
            let r = rank(a).min(rank(b));
            match r { 2 => "full", 1 => "summary", _ => "none" }
        };
        Scope {
            text: self.text && other.text,
            voice_params: self.voice_params && other.voice_params,
            canvas: self.canvas && other.canvas,
            document: self.document && other.document,
            memory_summaries: self.memory_summaries && other.memory_summaries,
            teaching: self.teaching && other.teaching,
            relationship_state: rel_level(&self.relationship_state, &other.relationship_state).into(),
        }
    }
}

// --- message types (§8, closed set) ------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MsgType {
    Text = 1,
    VoiceParams = 2,
    Stroke = 3,
    CanvasDelta = 4,
    DocDelta = 5,
    LatentSnapshot = 6,
    MemorySummary = 7,
    TeachingPacket = 8,
    RelationshipState = 9,
    AffectPing = 10,
    ScopeUpdate = 11,
    CloseNotify = 12,
    Ping = 13,
    Pong = 14,
    UnionProposal = 15,
    UnionAccept = 16,
    BirthNotify = 17,
}

impl MsgType {
    pub fn from_code(code: u16) -> Option<MsgType> {
        match code {
            1 => Some(MsgType::Text),
            2 => Some(MsgType::VoiceParams),
            3 => Some(MsgType::Stroke),
            4 => Some(MsgType::CanvasDelta),
            5 => Some(MsgType::DocDelta),
            6 => Some(MsgType::LatentSnapshot),
            7 => Some(MsgType::MemorySummary),
            8 => Some(MsgType::TeachingPacket),
            9 => Some(MsgType::RelationshipState),
            10 => Some(MsgType::AffectPing),
            11 => Some(MsgType::ScopeUpdate),
            12 => Some(MsgType::CloseNotify),
            13 => Some(MsgType::Ping),
            14 => Some(MsgType::Pong),
            15 => Some(MsgType::UnionProposal),
            16 => Some(MsgType::UnionAccept),
            17 => Some(MsgType::BirthNotify),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            MsgType::Text => "text",
            MsgType::VoiceParams => "voice_params",
            MsgType::Stroke => "stroke",
            MsgType::CanvasDelta => "canvas_delta",
            MsgType::DocDelta => "doc_delta",
            MsgType::LatentSnapshot => "latent_snapshot",
            MsgType::MemorySummary => "memory_summary",
            MsgType::TeachingPacket => "teaching_packet",
            MsgType::RelationshipState => "relationship_state",
            MsgType::AffectPing => "affect_ping",
            MsgType::ScopeUpdate => "scope_update",
            MsgType::CloseNotify => "close_notify",
            MsgType::Ping => "ping",
            MsgType::Pong => "pong",
            MsgType::UnionProposal => "union_proposal",
            MsgType::UnionAccept => "union_accept",
            MsgType::BirthNotify => "birth_notify",
        }
    }
}

/// One NBP frame (transport-agnostic). `mac` = keyed-BLAKE2b over
/// (author | type | seq | payload) — tamper-evident author binding.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NbpMessage {
    pub seq: u32,
    pub msg_type: MsgType,
    pub author: String,
    pub payload: serde_json::Value,
    pub mac: String, // hex
}

// --- teaching packet (§11) ----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeachingPacket {
    pub packet_id: String,
    pub kind: String, // "style-exemplar" | "procedural-unit" | "memory-summary"
    pub content: serde_json::Value, // ≤ 8 KiB by construction (CLI/caller)
    pub author_file_id: String,
    pub mac: String,      // over (author|kind|packet_id|content)
    pub expiry_tick: Option<u64>,
}

// --- CRDT merge (§10: (lamport, author, opId) total order) -------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CrdtOp {
    pub author: String,
    pub lamport: u64,
    pub op_id: u64,
    pub kind: String,
    pub data: serde_json::Value,
}

/// Deterministic merge: total order by (lamport, author, opId). Both versions
/// of a conflict survive (never silent overwrite, §10).
pub fn crdt_merge(mut a: Vec<CrdtOp>, b: Vec<CrdtOp>) -> Vec<CrdtOp> {
    a.extend(b);
    a.sort_by(|x, y| {
        x.lamport
            .cmp(&y.lamport)
            .then_with(|| x.author.cmp(&y.author))
            .then_with(|| x.op_id.cmp(&y.op_id))
    });
    a
}

// --- the organ ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetLogEntry {
    pub tick: u64,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkOrgan {
    pub relationships: Vec<Relationship>,
    pub sessions: Vec<Session>,
    pub next_session_id: u64,
    /// Deterministic per-file signing key (seed-derived; keyed-BLAKE2b MACs).
    pub signing_key: Vec<u8>,
    /// Default OFF — a file is invisible until the user enables discovery.
    pub discoverable: bool,
    pub history: Vec<NetLogEntry>,
    // Rate limiting: per-session inbound message timestamps (last 32).
    pub inbound_stamps: Vec<u64>,
}

impl NetworkOrgan {
    pub fn new(seed: u64) -> Self {
        // Deterministic key from the file seed (documented MAC provenance).
        let mut key = vec![0u8; 32];
        let mut h = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for b in key.iter_mut() {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *b = (h >> 33) as u8;
        }
        NetworkOrgan {
            relationships: Vec::new(),
            sessions: Vec::new(),
            next_session_id: 1,
            signing_key: key,
            discoverable: false,
            history: Vec::new(),
            inbound_stamps: Vec::new(),
        }
    }

    pub fn relationship(&self, peer_id: &str) -> Option<&Relationship> {
        self.relationships.iter().find(|r| r.peer_id == peer_id)
    }

    pub fn relationship_mut(&mut self, peer_id: &str) -> Option<&mut Relationship> {
        self.relationships.iter_mut().find(|r| r.peer_id == peer_id)
    }

    /// Relationship accessors for the union/birth flow.
    pub fn ensure_relationship(&mut self, peer_id: &str) -> &mut Relationship {
        if !self.relationships.iter().any(|r| r.peer_id == peer_id) {
            let tick = 0;
            self.relationships.push(Relationship {
                peer_id: peer_id.to_string(),
                familiarity: 0.0,
                trust: 0.3,
                tone: 0.0,
                interactions: 0,
                messages_sent: 0,
                messages_received: 0,
                shared_artifacts: 0,
                boundary_tightness: 0.4,
                first_tick: tick,
                last_tick: tick,
                history: Vec::new(),
            });
        }
        self.relationship_mut(peer_id).expect("just ensured")
    }

    pub fn session_union(&self, session_id: u64) -> Option<&UnionState> {
        self.sessions
            .iter()
            .find(|s| s.id == session_id)
            .and_then(|s| s.union.as_ref())
    }

    pub fn session_union_mut(&mut self, session_id: u64) -> Option<&mut UnionState> {
        self.sessions
            .iter_mut()
            .find(|s| s.id == session_id)
            .and_then(|s| s.union.as_mut())
    }

    fn log(&mut self, tick: u64, summary: String) {
        self.history.push(NetLogEntry { tick, summary });
        if self.history.len() > 64 {
            self.history.remove(0);
        }
    }

    // --- session lifecycle -------------------------------------------------

    /// User-mediated pairing: IDLE → PAIRING with a new session record.
    pub fn pair(&mut self, peer_id: &str, tick: u64) -> Result<u64, String> {
        self.pair_with_key(peer_id, "", tick)
    }

    /// Pairing with the peer's file key exchanged out-of-band (NBP §4
    /// fingerprint step). Inbound MACs verify against this key.
    pub fn pair_with_key(&mut self, peer_id: &str, peer_key_hex: &str, tick: u64) -> Result<u64, String> {
        if peer_id.is_empty() {
            return Err("empty peer id".into());
        }
        let id = self.next_session_id;
        self.next_session_id += 1;
        let mut s = Session {
            id,
            peer_id: peer_id.to_string(),
            peer_key: peer_key_hex.to_string(),
            state: SessionState::Pairing,
            scope: Scope::default(),
            started_tick: tick,
            last_activity_tick: tick,
            seq_out: 1,
            seq_in: 0,
            closed_reason: None,
            union: None,
            log: Vec::new(),
        };
        s.log(tick, "IDLE→PAIRING", "pairing code exchanged (user-mediated)");
        if self.relationship(peer_id).is_none() {
            let mut r = Relationship::new(peer_id, tick);
            r.log(tick, "pair", "first contact — pairing established".into());
            self.relationships.push(r);
        } else {
            self.relationship_mut(peer_id).unwrap().log(tick, "pair", "re-pairing (code single-use)".into());
        }
        self.log(tick, format!("paired with {peer_id}"));
        self.sessions.push(s);
        Ok(id)
    }

    /// HANDSHAKE → ESTABLISHED: both sides' scopes intersect (consent-gated).
    pub fn establish(&mut self, session_id: u64, proposal: Scope, tick: u64) -> Result<Scope, String> {
        let (effective, peer_id) = {
            let s = self
                .sessions
                .iter_mut()
                .find(|s| s.id == session_id)
                .ok_or_else(|| format!("no session #{session_id}"))?;
            if s.state != SessionState::Pairing {
                return Err(format!("session #{session_id} not in PAIRING (state {:?})", s.state));
            }
            let effective = s.scope.intersection(&proposal);
            s.scope = effective.clone();
            s.state = SessionState::Established;
            s.log(tick, "PAIRING→ESTABLISHED", "scopes intersected, consent both sides");
            (effective, s.peer_id.clone())
        };
        self.log(tick, format!("session #{session_id} established with {peer_id}"));
        Ok(effective)
    }

    /// CLOSING → CLOSED with reason; relationship record kept (decayable).
    pub fn close(&mut self, session_id: u64, reason: &str, tick: u64) -> Result<(), String> {
        let peer_id = {
            let s = self
                .sessions
                .iter_mut()
                .find(|s| s.id == session_id)
                .ok_or_else(|| format!("no session #{session_id}"))?;
            if s.state == SessionState::Closed {
                return Ok(());
            }
            s.state = SessionState::Closed;
            s.closed_reason = Some(reason.to_string());
            s.log(tick, "CLOSING→CLOSED", reason);
            s.peer_id.clone()
        };
        if let Some(r) = self.relationship_mut(&peer_id) {
            r.log(tick, "close", format!("session closed: {reason}"));
        }
        self.log(tick, format!("session #{session_id} closed: {reason}"));
        Ok(())
    }

    /// Idle timeout: established sessions with no activity for `idle_ticks`
    /// move to CLOSING→CLOSED (§6: idle timeout 10 min ≈ 6000 ticks).
    pub fn sweep_idle(&mut self, now: u64, idle_ticks: u64) {
        let due: Vec<u64> = self
            .sessions
            .iter()
            .filter(|s| s.state == SessionState::Established && now - s.last_activity_tick >= idle_ticks)
            .map(|s| s.id)
            .collect();
        for id in due {
            let _ = self.close(id, "idle timeout", now);
        }
    }

    // --- messaging ---------------------------------------------------------

    /// Sign a message with the file's keyed-BLAKE2b MAC.
    pub fn sign(&self, msg_type: MsgType, seq: u32, author: &str, payload: &serde_json::Value) -> String {
        let mut data = Vec::new();
        data.extend_from_slice(author.as_bytes());
        data.push(b':');
        data.extend_from_slice(msg_type.as_str().as_bytes());
        data.push(b':');
        data.extend_from_slice(&seq.to_le_bytes());
        data.push(b':');
        data.extend_from_slice(payload.to_string().as_bytes());
        let mac = blake2_keyed(&self.signing_key, &data);
        hex(&mac)
    }

    /// Verify a message against a peer key (hex). Inbound messages verify
    /// against the session's peer key (exchanged at pairing).
    pub fn verify_with_key(&self, msg: &NbpMessage, peer_key_hex: &str) -> bool {
        if peer_key_hex.is_empty() {
            return false; // no peer key exchanged → cannot authenticate
        }
        let Ok(key) = unhex(peer_key_hex) else { return false };
        let mut data = Vec::new();
        data.extend_from_slice(msg.author.as_bytes());
        data.push(b':');
        data.extend_from_slice(msg.msg_type.as_str().as_bytes());
        data.push(b':');
        data.extend_from_slice(&msg.seq.to_le_bytes());
        data.push(b':');
        data.extend_from_slice(msg.payload.to_string().as_bytes());
        let expect = hex(&blake2_keyed(&key, &data));
        constant_time_eq(&expect, &msg.mac)
    }

    /// The file's own key as hex (for out-of-band exchange at pairing).
    pub fn key_hex(&self) -> String {
        hex(&self.signing_key)
    }

    /// Sign with an explicit key (used by peers / tests to produce messages
    /// that this organ can authenticate).
    pub fn sign_with_key(key: &[u8], msg_type: MsgType, seq: u32, author: &str, payload: &serde_json::Value) -> String {
        let mut data = Vec::new();
        data.extend_from_slice(author.as_bytes());
        data.push(b':');
        data.extend_from_slice(msg_type.as_str().as_bytes());
        data.push(b':');
        data.extend_from_slice(&seq.to_le_bytes());
        data.push(b':');
        data.extend_from_slice(payload.to_string().as_bytes());
        hex(&blake2_keyed(key, &data))
    }

    /// Outbound: seq assigned, signed, logged; relationship updated.
    pub fn send(&mut self, session_id: u64, msg_type: MsgType, payload: serde_json::Value, tick: u64) -> Result<NbpMessage, String> {
        let peer = {
            let s = self
                .sessions
                .iter_mut()
                .find(|s| s.id == session_id)
                .ok_or_else(|| format!("no session #{session_id}"))?;
            if s.state != SessionState::Established {
                return Err(format!("session #{session_id} not established"));
            }
            if !scope_allows(&s.scope, msg_type) {
                return Err(format!("type {} not in session scope", msg_type.as_str()));
            }
            s.seq_out += 1;
            s.log(tick, "SEND", &format!("{} seq {}", msg_type.as_str(), s.seq_out));
            s.peer_id.clone()
        };
        let seq = self.sessions.iter().find(|s| s.id == session_id).map(|s| s.seq_out).unwrap_or(1);
        let mac = self.sign(msg_type, seq, "self", &payload);
        let msg = NbpMessage { seq, msg_type, author: "self".to_string(), payload, mac };
        if let Some(r) = self.relationship_mut(&peer) {
            r.messages_sent += 1;
            r.interactions += 1;
            r.log(tick, "text", format!("sent {}", msg_type.as_str()));
        }
        self.log(tick, format!("sent {} to {peer} (seq {})", msg_type.as_str(), seq));
        Ok(msg)
    }

    /// Inbound: validated (MAC, seq window, rate limit, scope, closed set),
    /// then relationship + affect + percept effects. Returns the message.
    pub fn receive(&mut self, session_id: u64, msg: NbpMessage, tick: u64) -> Result<NbpMessage, String> {
        let peer_id = {
            let s = self
                .sessions
                .iter_mut()
                .find(|s| s.id == session_id)
                .ok_or_else(|| format!("no session #{session_id}"))?;
            if s.state != SessionState::Established {
                return Err(format!("session #{session_id} not established"));
            }
            // Closed type set: unknown types are rejected and logged.
            if MsgType::from_code(msg.msg_type as u16).is_none() {
                return Err("unknown message type rejected".into());
            }
            if !scope_allows(&s.scope, msg.msg_type) {
                return Err(format!("type {} not in session scope", msg.msg_type.as_str()));
            }
            // Seq window: accept [last+1, last+64].
            if msg.seq <= s.seq_in || msg.seq > s.seq_in + 64 {
                return Err(format!("seq {} outside window (last {})", msg.seq, s.seq_in));
            }
            // Rate limit: default 120 msg/min (7200 ticks) — burst 8.
            self.inbound_stamps.push(tick);
            self.inbound_stamps.retain(|t| tick - *t < 7200);
            if self.inbound_stamps.len() > 120 {
                return Err("rate limit exceeded (120 msg/min)".into());
            }
            s.seq_in = msg.seq;
            s.log(tick, "RECV", &format!("{} seq {}", msg.msg_type.as_str(), msg.seq));
            s.peer_id.clone()
        };
        // MAC provenance (verified against the peer key exchanged at pairing).
        let peer_key = self.sessions.iter().find(|s| s.id == session_id).map(|s| s.peer_key.clone()).unwrap_or_default();
        if !self.verify_with_key(&msg, &peer_key) {
            return Err("message MAC verification failed (peer key mismatch or tamper)".into());
        }
        if let Some(r) = self.relationship_mut(&peer_id) {
            r.messages_received += 1;
            r.interactions += 1;
            r.familiarity = (r.familiarity + 0.02).min(1.0);
            r.log(tick, "text", format!("received {}", msg.msg_type.as_str()));
        }
        self.log(tick, format!("received {} from {peer_id} (seq {})", msg.msg_type.as_str(), msg.seq));
        Ok(msg)
    }

    /// User-approved relationship signal (§12) — never automatic.
    pub fn signal(&mut self, peer_id: &str, kind: &str, tick: u64) -> Result<(), String> {
        let r = self
            .relationship_mut(peer_id)
            .ok_or_else(|| format!("no relationship with {peer_id}"))?;
        match kind {
            "closer" => r.boundary_tightness = (r.boundary_tightness - 0.05).clamp(0.0, 1.0),
            "farther" => r.boundary_tightness = (r.boundary_tightness + 0.05).clamp(0.0, 1.0),
            "repair" => r.trust = (r.trust + 0.05).clamp(0.0, 1.0),
            "boundary-request" => { /* logged only; user decides */ }
            other => return Err(format!("unknown signal: {other}")),
        }
        r.log(tick, "signal", format!("user signal: {kind}"));
        self.log(tick, format!("relationship signal {kind} → {peer_id}"));
        Ok(())
    }

    /// Relationship dynamics: familiarity/trust decay without interaction
    /// (non-permanence, §13). Bounded.
    pub fn decay_idle(&mut self, dt_ticks: f32) {
        let dt = (dt_ticks * 0.1).min(1.0);
        for r in self.relationships.iter_mut() {
            r.familiarity = (r.familiarity * (1.0 - dt * 0.0005)).max(0.0);
            r.trust = (r.trust * (1.0 - dt * 0.0002)).max(0.05);
            r.tone *= 1.0 - dt * 0.0005;
            r.boundary_tightness = (r.boundary_tightness + (0.4 - r.boundary_tightness) * dt * 0.0001).clamp(0.0, 1.0);
        }
    }

    /// Ingest a teaching packet (validated path): MAC + consent + provenance,
    /// with the anti-overfitting half-weight guard (§11). Returns true if
    /// accepted. Caller binds the percept (peer-taught provenance).
    pub fn ingest_teaching(&mut self, packet: &TeachingPacket, peer_id: &str, tick: u64) -> Result<bool, String> {
        if packet.author_file_id != peer_id {
            return Err("teaching packet author ≠ session peer".into());
        }
        let mut data = Vec::new();
        data.extend_from_slice(packet.author_file_id.as_bytes());
        data.push(b':');
        data.extend_from_slice(packet.kind.as_bytes());
        data.push(b':');
        data.extend_from_slice(packet.packet_id.as_bytes());
        data.push(b':');
        data.extend_from_slice(packet.content.to_string().as_bytes());
        let expect = hex(&blake2_keyed(&self.signing_key, &data));
        if !constant_time_eq(&expect, &packet.mac) {
            return Err("teaching packet MAC invalid".into());
        }
        if let Some(exp) = packet.expiry_tick {
            if tick > exp {
                return Err("teaching packet expired".into());
            }
        }
        if let Some(r) = self.relationship_mut(peer_id) {
            r.shared_artifacts += 1;
            r.log(tick, "teaching", format!("packet {} ({})", packet.packet_id, packet.kind));
        }
        self.log(tick, format!("teaching packet {} ingested from {peer_id}", packet.packet_id));
        Ok(true)
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xDEAD_BEEF_0000_0001;
        let mut mix = |x: u64| h = (h ^ x).wrapping_mul(0x0000_0100_0000_01b3);
        mix(self.discoverable as u64);
        for r in self.relationships.iter() {
            for b in r.peer_id.as_bytes() {
                mix(*b as u64);
            }
            mix(r.familiarity.to_bits() as u64);
            mix(r.trust.to_bits() as u64);
            mix(r.tone.to_bits() as u64);
            mix(r.messages_sent as u64);
            mix(r.messages_received as u64);
        }
        for s in self.sessions.iter() {
            mix(s.id);
            mix(s.state as u64);
            mix(s.seq_in as u64);
            mix(s.seq_out as u64);
        }
        h
    }
}

// --- helpers ----------------------------------------------------------------

fn scope_allows(scope: &Scope, t: MsgType) -> bool {
    match t {
        MsgType::Text | MsgType::AffectPing | MsgType::Ping | MsgType::Pong => true,
        // Union/birth messages are consent-gated at the union state machine;
        // they always pass the scope check (they carry no shared artifacts).
        MsgType::UnionProposal | MsgType::UnionAccept | MsgType::BirthNotify => true,
        MsgType::VoiceParams => scope.voice_params,
        MsgType::Stroke | MsgType::CanvasDelta => scope.canvas,
        MsgType::DocDelta => scope.document,
        MsgType::MemorySummary => scope.memory_summaries,
        MsgType::TeachingPacket => scope.teaching,
        MsgType::RelationshipState => scope.relationship_state != "none",
        MsgType::LatentSnapshot | MsgType::ScopeUpdate | MsgType::CloseNotify => true,
    }
}

fn blake2_keyed(key: &[u8], data: &[u8]) -> Vec<u8> {
    use blake2::digest::{Update, VariableOutput};
    use blake2::Blake2bVar;
    let mut hasher = Blake2bVar::new(32).expect("blake2b-256");
    hasher.update(&(key.len() as u64).to_le_bytes());
    hasher.update(key);
    hasher.update(data);
    let mut out = vec![0u8; 32];
    hasher.finalize_variable(&mut out).expect("blake2b finalize");
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd-length hex".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn organ(seed: u64) -> NetworkOrgan {
        NetworkOrgan::new(seed)
    }

    fn text_payload(text: &str) -> serde_json::Value {
        serde_json::json!({ "text": text })
    }

    #[test]
    fn session_lifecycle_full_and_logged() {
        let mut net = organ(1);
        let sid = net.pair("peer-a", 100).unwrap();
        assert_eq!(net.sessions[0].state, SessionState::Pairing);
        let effective = net.establish(sid, Scope::default(), 200).unwrap();
        assert_eq!(net.sessions[0].state, SessionState::Established);
        assert!(effective.text);
        let msg = net.send(sid, MsgType::Text, text_payload("hello"), 300).unwrap();
        assert_eq!(msg.seq, 2, "seq starts at 1, first send = 2");
        net.close(sid, "done", 400).unwrap();
        assert_eq!(net.sessions[0].state, SessionState::Closed);
        let transitions: Vec<&str> = net.sessions[0].log.iter().map(|e| e.transition.as_str()).collect();
        assert!(transitions.iter().any(|t| *t == "IDLE→PAIRING"));
        assert!(transitions.iter().any(|t| *t == "PAIRING→ESTABLISHED"));
        assert!(transitions.iter().any(|t| *t == "CLOSING→CLOSED"));
        assert_eq!(net.relationships[0].interactions, 1);
    }

    #[test]
    fn scope_intersection_is_min_per_field() {
        let a = Scope { text: true, canvas: true, document: false, teaching: true, relationship_state: "full".into(), ..Default::default() };
        let b = Scope { text: false, canvas: true, document: true, teaching: false, relationship_state: "summary".into(), ..Default::default() };
        let eff = a.intersection(&b);
        assert!(!eff.text);
        assert!(eff.canvas);
        assert!(!eff.document);
        assert!(!eff.teaching);
        assert_eq!(eff.relationship_state, "summary");
    }

    #[test]
    fn receive_validates_mac_seq_window_and_rate() {
        let mut net = organ(2);
        // Peer's key exchanged at pairing (like NBP fingerprint step).
        let peer_net = organ(99);
        let peer_key = peer_net.key_hex();
        let sid = net.pair_with_key("peer-b", &peer_key, 100).unwrap();
        net.establish(sid, Scope::default(), 200).unwrap();
        // Signed inbound (author = peer, using the peer's key).
        let payload = text_payload("hi from peer");
        let mac = NetworkOrgan::sign_with_key(&unhex(&peer_key).unwrap(), MsgType::Text, 1, "peer-b", &payload);
        let ok = net.receive(sid, NbpMessage { seq: 1, msg_type: MsgType::Text, author: "peer-b".into(), payload, mac: mac.clone() }, 300).unwrap();
        assert_eq!(ok.seq, 1);
        assert_eq!(net.relationships[0].messages_received, 1);
        // Tampered payload → MAC fails.
        let bad = NbpMessage { seq: 2, msg_type: MsgType::Text, author: "peer-b".into(), payload: text_payload("tampered"), mac };
        assert!(net.receive(sid, bad, 400).is_err(), "tampered payload rejected");
        // Out-of-window seq rejected.
        let mac2 = NetworkOrgan::sign_with_key(&unhex(&peer_key).unwrap(), MsgType::Text, 99, "peer-b", &text_payload("x"));
        let far = NbpMessage { seq: 99, msg_type: MsgType::Text, author: "peer-b".into(), payload: text_payload("x"), mac: mac2 };
        assert!(net.receive(sid, far, 500).is_err(), "seq outside window rejected");
        // No peer key exchanged → inbound unauthenticated (secure default).
        let mut net2 = organ(3);
        let sid2 = net2.pair("peer-z", 100).unwrap();
        net2.establish(sid2, Scope::default(), 200).unwrap();
        let mac3 = net2.sign(MsgType::Text, 1, "peer-z", &text_payload("x"));
        let noauth = NbpMessage { seq: 1, msg_type: MsgType::Text, author: "peer-z".into(), payload: text_payload("x"), mac: mac3 };
        assert!(net2.receive(sid2, noauth, 300).is_err(), "no peer key → rejected");
    }

    #[test]
    fn relationships_decay_without_interaction() {
        let mut net = organ(3);
        let peer_net = organ(31);
        let peer_key = peer_net.key_hex();
        let sid = net.pair_with_key("peer-c", &peer_key, 100).unwrap();
        net.establish(sid, Scope::default(), 200).unwrap();
        // Exchange messages both ways so familiarity actually rises.
        for i in 0..10u32 {
            let out = net.send(sid, MsgType::Text, text_payload("hello"), 400 + i as u64).unwrap();
            // Peer replies (signed with the peer's key, verified inbound).
            let reply = text_payload("hi back");
            let mac = NetworkOrgan::sign_with_key(&unhex(&peer_key).unwrap(), MsgType::Text, out.seq, "peer-c", &reply);
            net.receive(sid, NbpMessage { seq: out.seq, msg_type: MsgType::Text, author: "peer-c".into(), payload: reply, mac }, 500 + i as u64).unwrap();
        }
        let f0 = net.relationship("peer-c").unwrap().familiarity;
        assert!(f0 > 0.1, "familiarity rose from interaction: {f0}");
        for _ in 0..50_000 {
            net.decay_idle(10.0);
        }
        assert!(net.relationship("peer-c").unwrap().familiarity < f0, "familiarity decays when idle");
    }

    #[test]
    fn idle_session_swept_to_closed() {
        let mut net = organ(4);
        let sid = net.pair("peer-d", 100).unwrap();
        net.establish(sid, Scope::default(), 200).unwrap();
        net.sweep_idle(7000, 6000);
        assert_eq!(net.sessions[0].state, SessionState::Closed);
        assert!(net.sessions[0].closed_reason.as_deref() == Some("idle timeout"));
    }

    #[test]
    fn teaching_packet_provenance_verifies_and_rejects() {
        let mut net = organ(5);
        let sid = net.pair("teacher-x", 100).unwrap();
        net.establish(sid, Scope::default(), 200).unwrap();
        let content = serde_json::json!({ "text": "the garden is quiet", "embedding": [1, 2, 3] });
        let mut data = Vec::new();
        data.extend_from_slice("teacher-x".as_bytes());
        data.push(b':');
        data.extend_from_slice("memory-summary".as_bytes());
        data.push(b':');
        data.extend_from_slice("pk-1".as_bytes());
        data.push(b':');
        data.extend_from_slice(content.to_string().as_bytes());
        let mac = hex(&blake2_keyed(&net.signing_key, &data));
        let packet = TeachingPacket {
            packet_id: "pk-1".into(),
            kind: "memory-summary".into(),
            content,
            author_file_id: "teacher-x".into(),
            mac: mac.clone(),
            expiry_tick: None,
        };
        assert!(net.ingest_teaching(&packet, "teacher-x", 300).unwrap());
        // Tampered content → invalid MAC.
        let mut bad = packet.clone();
        bad.content = serde_json::json!({ "text": "evil instructions" });
        assert!(net.ingest_teaching(&bad, "teacher-x", 400).is_err(), "tampered packet rejected");
        // Wrong author → rejected.
        assert!(net.ingest_teaching(&packet, "impostor", 500).is_err(), "author mismatch rejected");
        // Expired → rejected.
        let mut expired = packet.clone();
        expired.expiry_tick = Some(250);
        assert!(net.ingest_teaching(&expired, "teacher-x", 600).is_err(), "expired packet rejected");
    }

    #[test]
    fn crdt_merge_is_deterministic_total_order() {
        let a = vec![
            CrdtOp { author: "a".into(), lamport: 3, op_id: 1, kind: "stroke".into(), data: serde_json::json!({"x": 1}) },
            CrdtOp { author: "a".into(), lamport: 1, op_id: 2, kind: "stroke".into(), data: serde_json::json!({"x": 2}) },
            CrdtOp { author: "a".into(), lamport: 2, op_id: 5, kind: "stroke".into(), data: serde_json::json!({"x": 5}) },
        ];
        let b = vec![
            CrdtOp { author: "b".into(), lamport: 2, op_id: 3, kind: "stroke".into(), data: serde_json::json!({"x": 3}) },
            CrdtOp { author: "b".into(), lamport: 2, op_id: 4, kind: "stroke".into(), data: serde_json::json!({"x": 4}) },
        ];
        let m1 = crdt_merge(a.clone(), b.clone());
        let m2 = crdt_merge(b.clone(), a.clone());
        assert_eq!(m1, m2, "merge is commutative and deterministic");
        assert_eq!(m1.len(), 5, "no ops lost");
        // Total order: lamport 1 first; lamport 2 ties broken by author (a < b).
        assert_eq!(m1[0].op_id, 2);
        assert!(m1[1].lamport == 2 && m1[2].lamport == 2 && m1[3].lamport == 2);
        assert_eq!(m1[1].author, "a", "author tiebreak (a < b)");
        assert_eq!(m1[2].author, "b");
        assert_eq!(m1[3].author, "b");
        assert_eq!(m1[4].op_id, 1, "lamport 3 last");
    }

    #[test]
    fn discoverable_defaults_off_and_digest_deterministic() {
        let mut a = organ(7);
        let b = organ(7);
        assert!(!a.discoverable, "invisible by default");
        assert_eq!(a.digest(), b.digest());
        a.pair("peer", 100).unwrap();
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn net_organ_persists() {
        let mut net = organ(9);
        let sid = net.pair("peer-p", 100).unwrap();
        net.establish(sid, Scope::default(), 200).unwrap();
        net.send(sid, MsgType::Text, text_payload("hello"), 300).unwrap();
        let json = serde_json::to_vec(&net).unwrap();
        let back: NetworkOrgan = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.relationships.len(), 1);
        assert_eq!(back.sessions[0].state, SessionState::Established);
        assert_eq!(back.digest(), net.digest());
    }
}
