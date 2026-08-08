//! System 3/4/5 — Episodic binder, semantic store (gist-lite), and budgeted
//! retrieval (DESIGN.md §4.3, §4.4, §19.3).
//!
//! M1 scope: online binding with salience, strength decay, capacity-bounded
//! pruning, streaming gist distillation into semantic nodes, and retrieval with
//! trace/node/token budgets. Sleep-based consolidation, replay and clustering
//! arrive in M2; edges in the semantic graph are created in M2 as well (the
//! schema is already in place).

use serde::{Deserialize, Serialize};

use crate::events::{cosine, StreamKind};

// --- decay parameters (sim ticks; 10 ticks = 1 sim-second) -------------------
/// Base trace half-life: 7 sim-days (604,800 ticks).
pub const BASE_HALF_LIFE_TICKS: f64 = 604_800.0;
/// High-salience traces (≥ 0.7) decay 4× slower; low (< 0.3) 4× faster.
pub const SALIENCE_DECAY_MULT: f64 = 4.0;

fn half_life_for(salience: f32) -> f64 {
    if salience >= 0.7 {
        BASE_HALF_LIFE_TICKS * SALIENCE_DECAY_MULT
    } else if salience < 0.3 {
        BASE_HALF_LIFE_TICKS / SALIENCE_DECAY_MULT
    } else {
        BASE_HALF_LIFE_TICKS
    }
}

fn decay_factor_per_tick(salience: f32) -> f32 {
    let hl = half_life_for(salience);
    // strength *= (1 - r) per tick with half-life hl: r = 1 - 2^(-1/hl)
    (1.0 - 2f64.powf(-1.0 / hl)) as f32
}

// --- episodic traces ---------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EpisodicTrace {
    pub id: u64,
    pub sim_time: u64,
    pub embedding: Vec<f32>,
    pub salience: f32,
    pub valence: f32,
    pub arousal: f32,
    pub strength: f32,
    pub decay_rate: f32,
    pub stream: StreamKind,
    pub keywords: Vec<String>,
    pub source: String,
    pub consolidation_state: u8, // 0 fresh, 1 replayed, 2 gist, 3 pruned-candidate
    pub reconsolidation_count: u32,
}

impl EpisodicTrace {
    pub fn score(&self) -> f32 {
        self.salience * self.strength
    }
}

#[derive(Clone, Debug)]
pub struct PendingEvent {
    pub sim_time: u64,
    pub features: Vec<f32>,
    pub keywords: Vec<String>,
    pub valence: f32,
    pub arousal: f32,
    pub stream: StreamKind,
    pub source: String,
}

/// Salience = novelty_w·novelty + emotion_w·emotion + 0.20·source + 0.10·baseline.
/// Novelty = distance to nearest existing trace (1 − max cosine).
pub fn compute_salience_with(
    embedding: &[f32],
    valence: f32,
    arousal: f32,
    source: &str,
    store: &EpisodicStore,
    novelty_w: f32,
    emotion_w: f32,
) -> f32 {
    let novelty = match store.nearest_cosine(embedding) {
        Some(c) => (1.0 - c).clamp(0.0, 1.0),
        None => 1.0,
    };
    let emotion = (valence.abs() + arousal * 0.5).clamp(0.0, 1.0);
    let source_w = match source {
        "user" => 1.0,
        "peer" => 0.8,
        "teacher" => 0.6,
        "self" => 0.5,
        _ => 0.4,
    };
    (novelty_w * novelty + emotion_w * emotion + 0.20 * source_w + 0.10 * 0.5).clamp(0.0, 1.0)
}

/// Default-weight salience (novelty 0.35, emotion 0.35).
pub fn compute_salience(
    embedding: &[f32],
    valence: f32,
    arousal: f32,
    source: &str,
    store: &EpisodicStore,
) -> f32 {
    compute_salience_with(embedding, valence, arousal, source, store, 0.35, 0.35)
}

pub struct EpisodicStore {
    pub traces: Vec<EpisodicTrace>,
    pub capacity: usize,
    pub next_id: u64,
    pub pruned_count: u64,
}

impl EpisodicStore {
    pub fn new(capacity: usize) -> Self {
        EpisodicStore {
            traces: Vec::new(),
            capacity,
            next_id: 1,
            pruned_count: 0,
        }
    }

    pub fn nearest_cosine(&self, embedding: &[f32]) -> Option<f32> {
        self.traces
            .iter()
            .map(|t| cosine(embedding, &t.embedding))
            .fold(None, |best: Option<f32>, c| {
                Some(best.map_or(c, |b| b.max(c)))
            })
    }

    /// Bind a window of pending events into one trace (salience-weighted mean
    /// embedding; keyword union; salience-weighted affect). Prunes the weakest
    /// trace when at capacity.
    pub fn bind(&mut self, pending: &[PendingEvent], now: u64) -> Option<EpisodicTrace> {
        self.bind_with(pending, now, crate::embodiment::EventGains::default())
    }

    /// Bind with embodiment-modulated salience weights (novelty/emotion).
    /// `now` is reserved for sleep-time replay re-binding (M2).
    pub fn bind_with(
        &mut self,
        pending: &[PendingEvent],
        _now: u64,
        gains: crate::embodiment::EventGains,
    ) -> Option<EpisodicTrace> {
        if pending.is_empty() {
            return None;
        }
        let dim = pending[0].features.len();
        let mut emb = vec![0.0f32; dim];
        let mut w_sum = 0.0f32;
        let mut valence = 0.0f32;
        let mut arousal = 0.0f32;
        let mut keywords: Vec<String> = Vec::new();
        let mut stream = pending[0].stream;
        let mut source = pending[0].source.clone();
        let mut sim_time = pending[0].sim_time;
        // First pass: pre-salience estimate for weighting (novelty vs. current
        // store so the window doesn't bind against itself).
        let mut weights = Vec::with_capacity(pending.len());
        for p in pending {
            let s = compute_salience_with(
                &p.features,
                p.valence,
                p.arousal,
                &p.source,
                self,
                gains.novelty_w,
                gains.emotion_w,
            );
            weights.push(s.max(0.05));
        }
        for (p, w) in pending.iter().zip(weights.iter()) {
            for (e, f) in emb.iter_mut().zip(p.features.iter()) {
                *e += f * w;
            }
            w_sum += w;
            valence += p.valence * w;
            arousal += p.arousal * w;
            for kw in &p.keywords {
                if !keywords.contains(kw) {
                    keywords.push(kw.clone());
                }
            }
            if p.sim_time > sim_time {
                sim_time = p.sim_time;
                stream = p.stream;
                source = p.source.clone();
            }
        }
        if w_sum > 1e-9 {
            for e in emb.iter_mut() {
                *e /= w_sum;
            }
        }
        let norm = (emb.iter().map(|v| v * v).sum::<f32>()).sqrt();
        if norm > 1e-9 {
            for e in emb.iter_mut() {
                *e /= norm;
            }
        }
        valence /= w_sum.max(1e-9);
        arousal /= w_sum.max(1e-9);

        let salience = compute_salience_with(
            &emb,
            valence,
            arousal,
            &source,
            self,
            gains.novelty_w,
            gains.emotion_w,
        );
        let rate = decay_factor_per_tick(salience);
        let trace = EpisodicTrace {
            id: self.next_id,
            sim_time,
            embedding: emb,
            salience,
            valence,
            arousal,
            strength: 1.0,
            decay_rate: rate,
            stream,
            keywords,
            source,
            consolidation_state: 0,
            reconsolidation_count: 0,
        };
        self.next_id += 1;

        // Admission control (§4.0): at capacity, drop the weakest trace.
        if self.traces.len() >= self.capacity {
            let floor = 0.01;
            if trace.score() < floor {
                self.pruned_count += 1;
                return None; // new trace below floor — drop it, log the prune
            }
            if let Some(idx) = self.weakest_index() {
                self.pruned_count += 1;
                self.traces.remove(idx);
            }
        }
        self.traces.push(trace.clone());
        Some(trace)
    }

    fn weakest_index(&self) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, t) in self.traces.iter().enumerate() {
            let s = t.score();
            if best.map_or(true, |(_, bs)| s < bs) {
                best = Some((i, s));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Online decay: strength and salience relax toward their floors.
    pub fn decay(&mut self, dt_ticks: f32) {
        for t in self.traces.iter_mut() {
            // strength *= (1 - per_tick_rate)^dt_ticks  → halved after one half-life
            let f = (1.0 - t.decay_rate).powf(dt_ticks);
            t.strength *= f;
            t.salience *= 1.0 - 1e-6 * dt_ticks; // very slow salience relaxation
        }
    }

    pub fn forget(&mut self, id: u64) -> bool {
        let before = self.traces.len();
        self.traces.retain(|t| t.id != id);
        self.traces.len() != before
    }

    /// Gini coefficient of the salience distribution (audit metric #7).
    pub fn salience_gini(&self) -> f32 {
        let mut s: Vec<f32> = self.traces.iter().map(|t| t.salience).collect();
        let n = s.len();
        if n == 0 {
            return 0.0;
        }
        s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f32 = s.iter().sum();
        if sum <= 0.0 {
            return 0.0;
        }
        let mut acc = 0.0f32;
        for (i, v) in s.iter().enumerate() {
            acc += (n - i) as f32 * v;
        }
        (1.0 - 2.0 * acc / (n as f32 * sum)).clamp(0.0, 1.0)
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.traces.len().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for t in self.traces.iter() {
            for b in t.id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in t.salience.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in t.strength.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in t.embedding.iter().take(8).flat_map(|v| v.to_bits().to_le_bytes()) {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
}

// --- retrieval ---------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RetrievalBudget {
    pub k_traces: usize,
    pub k_nodes: usize,
    pub token_cap: usize,
}

impl Default for RetrievalBudget {
    fn default() -> Self {
        RetrievalBudget {
            k_traces: 5,
            k_nodes: 3,
            token_cap: 2000,
        }
    }
}

pub struct RetrievedTrace {
    pub trace: EpisodicTrace,
    pub score: f32,
}

/// Budgeted retrieval (DESIGN.md §19.3): score = cosine × strength × recency;
/// trim to token cap, nodes dropped before traces.
pub fn retrieve_traces(
    store: &EpisodicStore,
    query: &[f32],
    budget: &RetrievalBudget,
    now: u64,
) -> (Vec<RetrievedTrace>, usize, bool) {
    const RECENCY_SCALE: f32 = 2_592_000.0; // 30 sim-days
    let mut scored: Vec<(f32, usize)> = store
        .traces
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let age = now.saturating_sub(t.sim_time) as f32;
            let recency = (-age / RECENCY_SCALE).exp();
            let score = cosine(query, &t.embedding) * t.strength * recency;
            (score, i)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = Vec::new();
    let mut tokens = 0usize;
    let mut truncated = false;
    for (score, i) in scored.into_iter().take(budget.k_traces) {
        let t = &store.traces[i];
        let tk = 1 + t.keywords.iter().map(|k| k.len() / 4).sum::<usize>() + 4;
        if tokens + tk > budget.token_cap && !out.is_empty() {
            truncated = true;
            break;
        }
        tokens += tk;
        out.push(RetrievedTrace {
            trace: t.clone(),
            score,
        });
    }
    if out.len() >= budget.k_traces {
        truncated = true;
    }
    (out, tokens, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::embed_keywords;

    fn kw(text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    #[test]
    fn binding_creates_salient_traces() {
        let mut store = EpisodicStore::new(100);
        let emb = embed_keywords(&kw("a very happy sunny day"), 1, 64);
        let pending = vec![PendingEvent {
            sim_time: 10,
            features: emb.clone(),
            keywords: kw("a very happy sunny day"),
            valence: 0.8,
            arousal: 0.6,
            stream: StreamKind::Text,
            source: "user".into(),
        }];
        let t = store.bind(&pending, 10).unwrap();
        assert!(t.salience > 0.4, "salience {}", t.salience);
        assert_eq!(store.traces.len(), 1);
        assert_eq!(t.id, 1);
    }

    #[test]
    fn capacity_pruning_drops_weakest() {
        let mut store = EpisodicStore::new(3);
        // Three bland, low-emotion traces (low salience).
        for i in 0..3 {
            let emb = embed_keywords(&kw(&format!("bland filler number {i}")), 1, 64);
            let pending = vec![PendingEvent {
                sim_time: i,
                features: emb,
                keywords: kw(&format!("bland filler number {i}")),
                valence: 0.0,
                arousal: 0.0,
                stream: StreamKind::Text,
                source: "system".into(),
            }];
            store.bind(&pending, i);
        }
        assert_eq!(store.traces.len(), 3);
        // A strong emotional trace must displace the weakest.
        let emb = embed_keywords(&kw("terrible awful horrible day"), 2, 64);
        let pending = vec![PendingEvent {
            sim_time: 10,
            features: emb,
            keywords: kw("terrible awful horrible day"),
            valence: -0.9,
            arousal: 0.8,
            stream: StreamKind::Text,
            source: "user".into(),
        }];
        store.bind(&pending, 10);
        assert_eq!(store.traces.len(), 3, "capacity respected");
        assert!(
            store.traces.iter().any(|t| t.source == "user"),
            "strong trace displaced the weakest bland trace"
        );
    }

    #[test]
    fn decay_follows_half_life() {
        let mut store = EpisodicStore::new(10);
        let emb = embed_keywords(&kw("memorable event"), 3, 64);
        let pending = vec![PendingEvent {
            sim_time: 0,
            features: emb,
            keywords: kw("memorable event"),
            valence: 0.5,
            arousal: 0.5,
            stream: StreamKind::Text,
            source: "user".into(),
        }];
        store.bind(&pending, 0);
        let t0 = store.traces[0].clone();
        // Advance one half-life for its salience class.
        let ticks = half_life_for(t0.salience) as f32;
        store.decay(ticks);
        let t1 = &store.traces[0];
        let ratio = t1.strength / t0.strength;
        assert!(
            (ratio - 0.5).abs() < 0.02,
            "expected ~0.5 after one half-life, got {ratio}"
        );
    }

    #[test]
    fn retrieval_scores_and_budgets() {
        let mut store = EpisodicStore::new(100);
        for (text, v) in [
            ("the red fox jumps quickly", 0.4),
            ("quiet rainy afternoon inside", -0.1),
            ("the red fox runs through woods", 0.5),
        ] {
            let emb = embed_keywords(&kw(text), 5, 64);
            let pending = vec![PendingEvent {
                sim_time: 0,
                features: emb,
                keywords: kw(text),
                valence: v,
                arousal: 0.3,
                stream: StreamKind::Text,
                source: "user".into(),
            }];
            store.bind(&pending, 0);
        }
        let q = embed_keywords(&kw("red fox"), 5, 64);
        let (hits, tokens, _trunc) = retrieve_traces(&store, &q, &RetrievalBudget::default(), 1000);
        assert_eq!(hits.len(), 3);
        assert!(tokens > 0);
        assert!(
            hits[0].score > hits[1].score,
            "fox query ranks fox traces first: {} vs {}",
            hits[0].score,
            hits[1].score
        );
        // Token cap that only fits one trace → truncated flag.
        let tight = RetrievalBudget {
            k_traces: 3,
            k_nodes: 0,
            token_cap: 4,
        };
        let (hits2, _, trunc2) = retrieve_traces(&store, &q, &tight, 1000);
        assert!(hits2.len() < 3);
        assert!(trunc2);
    }

    #[test]
    fn gini_uniform_vs_concentrated() {
        let mut store = EpisodicStore::new(100);
        for i in 0..10 {
            let emb = embed_keywords(&kw(&format!("uniform event {i}")), 7, 64);
            let pending = vec![PendingEvent {
                sim_time: i,
                features: emb,
                keywords: kw(&format!("uniform event {i}")),
                valence: 0.1,
                arousal: 0.1,
                stream: StreamKind::Text,
                source: "system".into(),
            }];
            store.bind(&pending, i);
        }
        // All low-emotion system events → similar salience → low gini.
        let g = store.salience_gini();
        assert!(g < 0.5, "uniform-ish salience gini {g}");
        // Manual concentration check on raw numbers.
        let mut store2 = EpisodicStore::new(100);
        store2.traces = (0..10)
            .map(|i| EpisodicTrace {
                id: i as u64,
                sim_time: 0,
                embedding: vec![],
                salience: if i == 0 { 0.99 } else { 0.01 },
                valence: 0.0,
                arousal: 0.0,
                strength: 1.0,
                decay_rate: 0.0,
                stream: StreamKind::Text,
                keywords: vec![],
                source: "system".into(),
                consolidation_state: 0,
                reconsolidation_count: 0,
            })
            .collect();
        assert!(store2.salience_gini() > 0.6, "concentrated salience → high gini");
    }
}
