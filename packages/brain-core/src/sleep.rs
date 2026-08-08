//! Systems 14/15 — Sleep pressure, consolidation stages, dream synthesis
//! (DESIGN.md §10).
//!
//! Sleep is functional: wind-down (flush + relax), light consolidation
//! (salience-weighted replay + recolored drift copies), deep consolidation
//! (downscaling, pruning, gist clustering into semantic nodes, emotional
//! regulation), and dream synthesis (associative walk over the semantic graph
//! with provenance-linked fragments). All stochasticity comes from the Brain's
//! seeded RNG, so sleep is fully deterministic per file.
//!
//! Structural guarantee: the dream stage performs no external actions — it only
//! reads stores and writes the dream log (enforced by construction: this module
//! has no access to tools, teachers, or files).

use serde::{Deserialize, Serialize};

use crate::events::cosine;
use crate::memory::EpisodicStore;
use crate::rng::Rng;
use crate::semantic::SemanticStore;

// Stage durations in sim-ticks (10 Hz).
pub const WIND_DOWN_TICKS: u64 = 3000; // 5 min
pub const LIGHT_TICKS: u64 = 12_000; // 20 min
pub const DEEP_TICKS: u64 = 24_000; // 40 min
pub const DREAM_TICKS: u64 = 18_000; // 30 min
pub const FULL_CYCLE_TICKS: u64 = WIND_DOWN_TICKS + LIGHT_TICKS + DEEP_TICKS + DREAM_TICKS; // 2 h

pub const PRESSURE_TRIGGER: f32 = 0.8;
pub const RETENTION_FLOOR: f32 = 0.02; // score (salience × strength) floor
pub const DOWNSCALE_FACTOR: f32 = 0.97;
pub const REPLAY_STRENGTHEN: f32 = 1.03;
pub const RECOLOR_CHANCE: f32 = 0.3;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StageWork {
    pub replayed: usize,
    pub recolored: usize,
    pub pruned: usize,
    pub gists: usize,
    pub emotional_regulated: bool,
    /// M6: sensory-integration work during sleep (§6.2 step 7).
    #[serde(default)]
    pub sensory_integrated: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SleepStage {
    pub stage: String,
    pub duration_ticks: u64,
    pub work: StageWork,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SleepReport {
    pub sleep_id: u64,
    pub started_at: u64,
    pub cycles: u32,
    pub stages: Vec<SleepStage>,
    pub dreams: Vec<u64>,
    pub modulator_normalized: bool,
    pub bias_actions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DreamFragment {
    pub modality: String, // "text" | "visual-motif" | "emotion" | "body"
    pub content: String,
    pub provenance: String, // "trace:<id>" | "node:<id>" | "interoception"
    pub bizarreness: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DreamLogEntry {
    pub dream_id: u64,
    pub sleep_id: u64,
    pub sim_time: u64,
    pub fragments: Vec<DreamFragment>,
    pub residue: Vec<String>,
    pub promoted: Option<u64>,
}

/// Sleep pressure accumulator (§10.1). Sources: elapsed time (0.05/h baseline),
/// memory pressure (capacity fullness), emotional load, and interoception
/// (low energy / high fatigue push pressure up). All per-tick at 10 Hz.
#[derive(Clone, Debug)]
pub struct SleepSystem {
    pub pressure: f32,
    pub emotional_load: f32,
    pub last_sleep_tick: u64,
    pub circadian_enabled: bool,
}

impl SleepSystem {
    pub fn new() -> Self {
        SleepSystem {
            pressure: 0.0,
            emotional_load: 0.0,
            last_sleep_tick: 0,
            circadian_enabled: false,
        }
    }

    pub fn step(&mut self, dt_ticks: u64, energy: f32, fatigue: f32, fullness: f32) {
        let d = dt_ticks as f32;
        self.pressure += 0.05 / 36_000.0 * d; // baseline 0.05/h
        self.pressure += fullness * 0.02 / 36_000.0 * d;
        self.pressure += self.emotional_load * 0.01 / 36_000.0 * d;
        self.pressure += (1.0 - energy) * 0.005 / 36_000.0 * d;
        self.pressure += fatigue * 0.01 / 36_000.0 * d;
        self.pressure = self.pressure.clamp(0.0, 1.0);
        self.emotional_load *= (1.0 - 0.0001 * d).max(0.0);
    }

    /// Register emotional intensity from processed events (|valence delta|).
    pub fn add_emotional_load(&mut self, magnitude: f32) {
        self.emotional_load = (self.emotional_load + magnitude.abs() * 0.1).min(2.0);
    }

    pub fn triggers(&self, fullness: f32) -> Vec<String> {
        let mut t = Vec::new();
        if self.pressure >= PRESSURE_TRIGGER {
            t.push("pressure".into());
        }
        if fullness >= 0.95 {
            t.push("memory-critical".into());
        }
        if self.emotional_load >= 0.5 {
            t.push("emotional-load-high".into());
        }
        t
    }

    pub fn reset(&mut self) {
        self.pressure = 0.05;
        self.emotional_load = 0.0;
    }
}

impl Default for SleepSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// One pass of light-stage replay over a trace: strengthen, count, and mark.
/// Returns (replayed, recolored) — decisions are made up front so the caller
/// can borrow stores disjointly.
pub fn plan_replay(
    store: &EpisodicStore,
    nodes: &SemanticStore,
    budget: usize,
    rng: &mut Rng,
) -> Vec<(usize, bool, Option<Vec<f32>>)> {
    let mut scored: Vec<(f32, usize)> = store
        .traces
        .iter()
        .enumerate()
        .map(|(i, t)| (t.score(), i))
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut plan = Vec::new();
    for (_, i) in scored.into_iter().take(budget) {
        let recolor = rng.next_f32() < RECOLOR_CHANCE;
        // Drift target: blend toward the nearest semantic node (or keep).
        let drift: Option<Vec<f32>> = if recolor {
            let t = &store.traces[i];
            let mut emb = t.embedding.clone();
            if let Some((nidx, c)) = nodes.nearest(&t.embedding) {
                if c > 0.5 {
                    let blend = 0.12f32;
                    for (e, f) in emb.iter_mut().zip(nodes.nodes[nidx].embedding.iter()) {
                        *e = *e * (1.0 - blend) + f * blend;
                    }
                }
            }
            // tiny seeded jitter
            for v in emb.iter_mut() {
                *v += (rng.next_f32() - 0.5) * 0.02;
            }
            let norm = (emb.iter().map(|v| v * v).sum::<f32>()).sqrt();
            if norm > 1e-9 {
                for v in emb.iter_mut() {
                    *v /= norm;
                }
            }
            Some(emb)
        } else {
            None
        };
        plan.push((i, recolor, drift));
    }
    plan
}

/// Cluster traces for deep-stage gist extraction (streaming k-means, capacity
/// capped). Pure function — the caller applies the results.
pub fn cluster_traces(traces: &[crate::memory::EpisodicTrace], max_clusters: usize) -> Vec<Cluster> {
    let mut clusters: Vec<Cluster> = Vec::new();
    for t in traces {
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in clusters.iter().enumerate() {
            let sim = cosine(&t.embedding, &c.centroid);
            if best.map_or(true, |(_, bs)| sim > bs) {
                best = Some((i, sim));
            }
        }
        match best {
            Some((i, sim)) if sim >= 0.6 && clusters.len() <= max_clusters => {
                let c = &mut clusters[i];
                c.members.push(t.id);
                c.sum_salience += t.salience;
                let n = c.members.len() as f32;
                for (e, f) in c.centroid.iter_mut().zip(t.embedding.iter()) {
                    *e = *e * ((n - 1.0) / n) + f * (1.0 / n);
                }
                for kw in &t.keywords {
                    *c.keywords.entry(kw.clone()).or_insert(0u32) += 1;
                }
            }
            _ => {
                if clusters.len() < max_clusters {
                    let mut keywords = std::collections::HashMap::new();
                    for kw in &t.keywords {
                        *keywords.entry(kw.clone()).or_insert(0u32) += 1;
                    }
                    clusters.push(Cluster {
                        centroid: t.embedding.clone(),
                        members: vec![t.id],
                        sum_salience: t.salience,
                        keywords,
                    });
                }
            }
        }
    }
    clusters
}

#[derive(Clone, Debug)]
pub struct Cluster {
    pub centroid: Vec<f32>,
    pub members: Vec<u64>,
    pub sum_salience: f32,
    pub keywords: std::collections::HashMap<String, u32>,
}

impl Cluster {
    /// Cohesion: mean cosine of members to centroid (cheap approximation).
    pub fn cohesion(&self, traces: &[crate::memory::EpisodicTrace]) -> f32 {
        let mut sum = 0.0f32;
        let mut n = 0usize;
        for t in traces {
            if self.members.contains(&t.id) {
                sum += cosine(&t.embedding, &self.centroid);
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            sum / n as f32
        }
    }

    pub fn top_keyword(&self) -> String {
        self.keywords
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "gist".into())
    }
}

/// Dream synthesis (§10.5): associative walk over the semantic graph from
/// residue (high-score traces + a random node), producing provenance-linked
/// fragments. Reads only; the caller appends to the dream log.
pub fn synthesize_dreams(
    sim_time: u64,
    sleep_id: u64,
    next_dream_id: &mut u64,
    traces: &EpisodicStore,
    nodes: &SemanticStore,
    interoception: (f32, f32, f32), // (load, saturation, fatigue)
    rng: &mut Rng,
) -> Vec<DreamLogEntry> {
    let mut entries = Vec::new();
    if traces.traces.is_empty() && nodes.nodes.is_empty() {
        return entries;
    }
    // Residue: top-3 traces by score.
    let mut scored: Vec<(f32, usize)> = traces
        .traces
        .iter()
        .enumerate()
        .map(|(i, t)| (t.score(), i))
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut seeds: Vec<Vec<f32>> = scored
        .iter()
        .take(3)
        .map(|(_, i)| traces.traces[*i].embedding.clone())
        .collect();
    if !nodes.nodes.is_empty() {
        let ni = rng.next_u64_below(nodes.nodes.len() as u64) as usize;
        seeds.push(nodes.nodes[ni].embedding.clone());
    }
    for seed in seeds {
        let mut fragments = Vec::new();
        let mut residue = Vec::new();
        let mut current = seed;
        let steps = 3 + rng.next_u64_below(4) as usize; // 3..6
        for step in 0..steps {
            if nodes.nodes.is_empty() {
                break;
            }
            // Next node: similarity-weighted with associative temperature.
            let temperature = 0.55f32;
            let mut best: Option<(usize, f32)> = None;
            for (i, n) in nodes.nodes.iter().enumerate() {
                let sim = cosine(&current, &n.embedding);
                let noisy = sim * (1.0 - temperature) + rng.next_f32() * temperature;
                if best.map_or(true, |(_, bs)| noisy > bs) {
                    best = Some((i, noisy));
                }
            }
            let (ni, _) = best.unwrap();
            let node = &nodes.nodes[ni];
            let jump = 1.0 - cosine(&current, &node.embedding);
            let modality = match step % 3 {
                0 => "text",
                1 => "visual-motif",
                _ => "emotion",
            };
            let content = match modality {
                "text" => node.label.clone(),
                "visual-motif" => node.label.clone(),
                _ => format!("emotion residue: {:+.2}", node.belief),
            };
            fragments.push(DreamFragment {
                modality: modality.to_string(),
                content: content.chars().take(60).collect(),
                provenance: format!("node:{}", node.id),
                bizarreness: jump.clamp(0.0, 1.0),
            });
            residue.push(format!("node:{}", node.id));
            current = node.embedding.clone();
        }
        // Body sensation fragment from interoception (if any fragments exist).
        if !fragments.is_empty() {
            fragments.push(DreamFragment {
                modality: "body".into(),
                content: format!(
                    "weightless, load {:.2}, saturation {:.2}, fatigue {:.2}",
                    interoception.0, interoception.1, interoception.2
                ),
                provenance: "interoception".into(),
                bizarreness: 0.1,
            });
        }
        if fragments.is_empty() {
            continue;
        }
        entries.push(DreamLogEntry {
            dream_id: *next_dream_id,
            sleep_id,
            sim_time,
            fragments,
            residue,
            promoted: None,
        });
        *next_dream_id += 1;
    }
    entries
}
