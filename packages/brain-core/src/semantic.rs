//! System 4 — Semantic memory (DESIGN.md §4.4), M1 "gist-lite".
//!
//! Streaming distillation: when a bound episode lands near an existing node
//! (cosine > MATCH), the node's belief grows and its embedding blends toward
//! the episode; otherwise a new node is created. Full sleep-based clustering
//! and edge construction arrive in M2. Belief decays toward a floor.

use serde::{Deserialize, Serialize};

use crate::events::cosine;
use crate::memory::EpisodicTrace;

pub const MATCH_COSINE: f32 = 0.75;
pub const BELIEF_GAIN: f32 = 0.05;
pub const NEW_NODE_BELIEF: f32 = 0.15;
pub const BELIEF_FLOOR: f32 = 0.10;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticNode {
    pub id: u64,
    pub label: String, // first keywords joined, or "gist-<id>"
    pub embedding: Vec<f32>,
    pub belief: f32,
    pub source_episodes: Vec<u64>,
    pub provenance: ProvenanceWeights,
    pub created: u64,
    pub updated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProvenanceWeights {
    pub user: f32,
    pub llm: f32,
    pub peer: f32,
    pub gist: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticEdge {
    pub from: u64,
    pub to: u64,
    pub kind: String, // "is-a" | "causes" | "likes" | ... (created from M2)
    pub strength: f32,
}

pub struct SemanticStore {
    pub nodes: Vec<SemanticNode>,
    pub edges: Vec<SemanticEdge>,
    pub capacity: usize,
    pub next_id: u64,
    pub pruned_count: u64,
}

impl SemanticStore {
    pub fn new(capacity: usize) -> Self {
        SemanticStore {
            nodes: Vec::new(),
            edges: Vec::new(),
            capacity,
            next_id: 1,
            pruned_count: 0,
        }
    }

    pub fn nearest(&self, embedding: &[f32]) -> Option<(usize, f32)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, n) in self.nodes.iter().enumerate() {
            let c = cosine(embedding, &n.embedding);
            if best.map_or(true, |(_, bc)| c > bc) {
                best = Some((i, c));
            }
        }
        best
    }

    /// Distill a bound episode into the semantic store (gist-lite).
    pub fn ingest_trace(&mut self, trace: &EpisodicTrace) {
        match self.nearest(&trace.embedding) {
            Some((i, c)) if c >= MATCH_COSINE => {
                let node = &mut self.nodes[i];
                node.belief = (node.belief + BELIEF_GAIN * trace.salience).min(1.0);
                let blend = 0.15f32;
                for (e, f) in node.embedding.iter_mut().zip(trace.embedding.iter()) {
                    *e = *e * (1.0 - blend) + f * blend;
                }
                let norm = (node.embedding.iter().map(|v| v * v).sum::<f32>()).sqrt();
                if norm > 1e-9 {
                    for e in node.embedding.iter_mut() {
                        *e /= norm;
                    }
                }
                node.updated = trace.sim_time;
                node.source_episodes.push(trace.id);
                // Attribution follows the trace's source (gist provenance is
                // reserved for sleep-stage distillation).
                match trace.source.as_str() {
                    "user" => node.provenance.user += BELIEF_GAIN * trace.salience,
                    "teacher" => node.provenance.llm += BELIEF_GAIN * trace.salience,
                    "peer" => node.provenance.peer += BELIEF_GAIN * trace.salience,
                    _ => node.provenance.gist += BELIEF_GAIN * trace.salience,
                }
                if !trace.keywords.is_empty() && node.label.starts_with("gist-") {
                    node.label = trace.keywords[0].clone();
                }
            }
            _ => {
                if self.nodes.len() >= self.capacity {
                    if let Some(idx) = self.weakest_index() {
                        self.pruned_count += 1;
                        self.nodes.remove(idx);
                    }
                }
                let label = trace
                    .keywords
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("gist-{}", self.next_id));
                let mut prov = ProvenanceWeights::default();
                match trace.source.as_str() {
                    "user" => prov.user = 0.8 * trace.salience,
                    "teacher" => prov.llm = 0.6 * trace.salience,
                    "peer" => prov.peer = 0.6 * trace.salience,
                    _ => prov.gist = 0.5 * trace.salience,
                }
                self.nodes.push(SemanticNode {
                    id: self.next_id,
                    label,
                    embedding: trace.embedding.clone(),
                    belief: NEW_NODE_BELIEF + 0.3 * trace.salience,
                    source_episodes: vec![trace.id],
                    provenance: prov,
                    created: trace.sim_time,
                    updated: trace.sim_time,
                });
                self.next_id += 1;
            }
        }
    }

    /// Deep-stage gist distillation: a mature cluster becomes/strengthens a
    /// semantic node with provenance `gist` and linked source episodes.
    pub fn ingest_gist(
        &mut self,
        centroid: &[f32],
        label: &str,
        member_count: u32,
        salience: f32,
        member_ids: &[u64],
        now: u64,
    ) {
        match self.nearest(centroid) {
            Some((i, c)) if c >= 0.75 => {
                let node = &mut self.nodes[i];
                node.belief = (node.belief + 0.1 * salience).min(1.0);
                node.updated = now;
                node.provenance.gist += 0.1 * salience * member_count as f32;
                for id in member_ids {
                    if !node.source_episodes.contains(id) {
                        node.source_episodes.push(*id);
                    }
                }
            }
            _ => {
                if self.nodes.len() >= self.capacity {
                    if let Some(idx) = self.weakest_index() {
                        self.pruned_count += 1;
                        self.nodes.remove(idx);
                    }
                }
                self.nodes.push(SemanticNode {
                    id: self.next_id,
                    label: label.to_string(),
                    embedding: centroid.to_vec(),
                    belief: (0.2 + 0.05 * member_count as f32).min(1.0),
                    source_episodes: member_ids.to_vec(),
                    provenance: ProvenanceWeights {
                        gist: 0.5 * salience,
                        ..Default::default()
                    },
                    created: now,
                    updated: now,
                });
                self.next_id += 1;
            }
        }
    }

    fn weakest_index(&self) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, n) in self.nodes.iter().enumerate() {
            if best.map_or(true, |(_, bs)| n.belief < bs) {
                best = Some((i, n.belief));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Belief relaxes toward the floor (30-day-ish half-life).
    pub fn decay(&mut self, dt_ticks: f32) {
        let f = 0.999_999_0f32.powf(dt_ticks);
        for n in self.nodes.iter_mut() {
            n.belief = BELIEF_FLOOR + (n.belief - BELIEF_FLOOR) * f;
        }
    }

    /// Budgeted node retrieval: top-k by cosine × belief.
    pub fn retrieve(&self, query: &[f32], k: usize) -> Vec<(SemanticNode, f32)> {
        let mut scored: Vec<(f32, usize)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (cosine(query, &n.embedding) * n.belief, i))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(k)
            .map(|(s, i)| (self.nodes[i].clone(), s))
            .collect()
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.nodes.len().to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        for n in self.nodes.iter() {
            for b in n.id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in n.belief.to_bits().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in n.label.as_bytes() {
                h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::embed_keywords;
    use crate::memory::PendingEvent;
    use crate::memory::EpisodicStore;
    use crate::events::StreamKind;

    fn bind_one(store: &mut EpisodicStore, sem: &mut SemanticStore, text: &str, t: u64) {
        let kw: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        let emb = embed_keywords(&kw, 9, 64);
        let trace = store
            .bind(
                &[PendingEvent {
                    sim_time: t,
                    features: emb,
                    keywords: kw,
                    valence: 0.3,
                    arousal: 0.3,
                    stream: StreamKind::Text,
                    source: "user".into(),
                }],
                t,
            )
            .unwrap();
        sem.ingest_trace(&trace);
    }

    #[test]
    fn repeated_concepts_consolidate() {
        let mut store = EpisodicStore::new(100);
        let mut sem = SemanticStore::new(50);
        for t in 0..20 {
            bind_one(&mut store, &mut sem, "gardening tomatoes every morning", t * 100);
        }
        assert_eq!(sem.nodes.len(), 1, "one gist node for one concept");
        let node = &sem.nodes[0];
        assert!(node.belief > 0.5, "belief grew: {}", node.belief);
        assert_eq!(node.source_episodes.len(), 20);
    }

    #[test]
    fn distinct_concepts_stay_distinct() {
        let mut store = EpisodicStore::new(100);
        let mut sem = SemanticStore::new(50);
        for (i, text) in ["gardening tomatoes", "space rocket launch", "baking sourdough bread", "hiking mountain trails"].iter().enumerate() {
            bind_one(&mut store, &mut sem, text, i as u64 * 100);
        }
        assert_eq!(sem.nodes.len(), 4);
    }

    #[test]
    fn belief_decays_to_floor() {
        let mut store = EpisodicStore::new(100);
        let mut sem = SemanticStore::new(50);
        bind_one(&mut store, &mut sem, "gardening tomatoes", 0);
        let b0 = sem.nodes[0].belief;
        sem.decay(2_592_000.0 * 4.0); // ~120 sim-days
        assert!(sem.nodes[0].belief < b0);
        assert!(sem.nodes[0].belief >= BELIEF_FLOOR - 1e-4);
    }
}
