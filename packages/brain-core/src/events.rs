//! Sensory event envelope + deterministic keyword embedding (DESIGN.md §4.9, §6.3).
//!
//! M1 note: organs are not built yet, so `embed_keywords` is the *factory*
//! embedding: a deterministic hash-based vector per keyword set. It gives real,
//! testable cosine retrieval semantics (same keywords → same vector). Production
//! organs replace this with trained encoders (M3+); the event envelope and the
//! ingest path are encoder-agnostic.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    Visual,
    Auditory,
    Text,
    Touch,
    Motion,
    Interoception,
    Ui,
    Social,
}

impl StreamKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamKind::Visual => "visual",
            StreamKind::Auditory => "auditory",
            StreamKind::Text => "text",
            StreamKind::Touch => "touch",
            StreamKind::Motion => "motion",
            StreamKind::Interoception => "interoception",
            StreamKind::Ui => "ui",
            StreamKind::Social => "social",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensoryEvent {
    pub stream: StreamKind,
    pub sim_time: u64,
    /// Compressed feature vector (dim = file latent dim in production; M1
    /// factory embeddings use the same dim).
    pub features: Vec<f32>,
    /// Retrieval cues.
    pub keywords: Vec<String>,
    /// (valence, arousal) guess, confidence-weighted by the producer.
    pub affect_guess: (f32, f32),
    pub confidence: f32,
    pub source: String, // "user" | "self" | "teacher" | "peer" | "system"
}

impl SensoryEvent {
    /// Simple tokenization for retrieval cues (lowercase, alnum words, min len 3,
    /// no stopwords beyond a tiny set).
    pub fn keywords_from_text(text: &str) -> Vec<String> {
        const STOP: &[&str] = &["the", "and", "for", "with", "you", "are", "was", "had", "but"];
        text.split(|c: char| !c.is_alphanumeric())
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() >= 3)
            .filter(|w| !STOP.contains(&w.as_str()))
            .collect()
    }
}

/// FNV-1a 64 (shared with digest code; kept here for embedding seeds).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Deterministic L2-normalized embedding for a keyword set.
/// Order-independent (sum of per-keyword vectors), seeded by `base`.
pub fn embed_keywords(keywords: &[String], base: u64, dim: usize) -> Vec<f32> {
    if dim == 0 {
        return Vec::new();
    }
    let mut acc = vec![0.0f32; dim];
    for kw in keywords {
        let mut rng = Rng::new(fnv1a(kw.as_bytes()).wrapping_add(base));
        for v in acc.iter_mut() {
            *v += rng.next_f32() - 0.5;
        }
    }
    let norm = (acc.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if norm > 1e-9 {
        for v in acc.iter_mut() {
            *v /= norm;
        }
    }
    acc
}

/// Deterministic projection of a raw encoder embedding into the file's latent
/// space (BUILD-THE-BODY Phase 0): seeded random matrix, L2-normalized.
/// Handcrafted 16-dim features already fit any tier's latent dim and are
/// returned unchanged; richer embeddings (e.g. 1024-dim V-JEPA 2) project so
/// every memory lives in one consistent space per file. Same input + same
/// base → same output, always.
pub fn project_features(features: &[f32], base: u64, dim: usize) -> Vec<f32> {
    if dim == 0 || features.len() <= dim {
        return features.to_vec();
    }
    let mut rng = Rng::new(base.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut out = vec![0.0f32; dim];
    for v in out.iter_mut() {
        let mut acc = 0.0f32;
        for f in features {
            acc += (rng.next_f32() - 0.5) * f;
        }
        *v = acc;
    }
    let norm = (out.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if norm > 1e-9 {
        for v in out.iter_mut() {
            *v /= norm;
        }
    }
    out
}

/// Cosine similarity between two vectors (both assumed L2-normalized or not —
/// works either way via the dot product / norms).
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na * nb).sqrt();
    if denom < 1e-9 {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_features_is_deterministic_and_l2_normalized() {
        let feats: Vec<f32> = (0..1024).map(|i| (i as f32) / 1024.0).collect();
        let a = project_features(&feats, 42, 256);
        let b = project_features(&feats, 42, 256);
        assert_eq!(a, b, "same input + same seed must give the same vector");
        assert_eq!(a.len(), 256);
        let norm: f32 = a.iter().map(|v| v * v).sum();
        assert!((norm - 1.0).abs() < 1e-4, "not L2-normalized: {norm}");
        let c = project_features(&feats, 43, 256);
        assert_ne!(a, c, "different seed must give a different projection");
    }

    #[test]
    fn project_features_passes_small_vectors_through() {
        let small = vec![0.1f32, 0.2, 0.3];
        // Handcrafted 16-dim already fits any tier's latent dim (192+):
        // it must pass through untouched.
        assert_eq!(project_features(&small, 42, 192), small);
        assert_eq!(project_features(&small, 42, 16), small);
    }
}
