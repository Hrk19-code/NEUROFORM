//! Writing organ — external verbal memory (DESIGN.md §8).
//!
//! M3 core (headless): document model with modes, local style fingerprinting,
//! a continuity ledger with contradiction detection, and the extraction
//! pipeline that turns writing into episodic traces, semantic nodes, style
//! memory, preference signals, and procedural habits. The editor UI (rich
//! text, version history, scene cards, etc.) is the desktop-shell milestone;
//! every feature in §8.2–8.3 consumes this model.
//!
//! All analysis is local and deterministic — no cloud, no hidden uploads.

use serde::{Deserialize, Serialize};

// --- document model ----------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DocMode {
    Prose,
    Journal,
    Worldbuilding,
    Lorebook,
    Markdown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocBlock {
    pub id: u64,
    pub kind: String, // "heading" | "para" | "quote" | "list" | "scene-card" | "entity-card" | "beat" | "note"
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Document {
    pub id: u64,
    pub title: String,
    pub mode: DocMode,
    pub blocks: Vec<DocBlock>,
    pub created: u64,
    pub updated: u64,
    pub style: StyleFingerprint,
}

// --- style fingerprint -------------------------------------------------------

/// Rolling local style features (DESIGN.md §8.4 step 2).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StyleFingerprint {
    pub sentence_len_mean: f32,
    pub sentence_len_std: f32,
    pub lexical_density: f32, // content-word fraction (approx)
    pub clause_complexity: f32, // mean commas + connectives per sentence
    pub metaphor_load: f32,   // approx: unusual-noun-adj pairs per sentence
    pub sentiment_mean: f32,  // −1..1 from a small local lexicon
    pub sentiment_range: f32,
    pub dialogue_ratio: f32, // quoted fraction
    pub samples: u32,
}

const POSITIVE: [&str; 20] = [
    "warm", "kind", "glad", "soft", "bright", "gentle", "calm", "loved", "safe",
    "hopeful", "tender", "sweet", "joy", "peace", "light", "golden", "quiet",
    "good", "happy", "wonder",
];
const NEGATIVE: [&str; 20] = [
    "cold", "cruel", "sad", "hard", "dark", "harsh", "fear", "alone", "lost",
    "grief", "bitter", "rot", "broken", "hunger", "storm", "sour", "silent",
    "bad", "empty", "sharp",
];
const CONNECTIVES: [&str; 10] = [
    "and", "but", "because", "although", "while", "though", "so", "then",
    "when", "yet",
];
const STOP: [&str; 24] = [
    "the", "a", "an", "of", "to", "in", "on", "at", "by", "for", "with",
    "from", "as", "is", "are", "was", "were", "be", "been", "it", "its",
    "this", "that", "and",
];

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

fn sentences(text: &str) -> Vec<String> {
    text.split(|c: char| c == '.' || c == '!' || c == '?' || c == ';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Local sentiment score in [−1, 1].
pub fn sentiment(text: &str) -> f32 {
    let ws = words(text);
    if ws.is_empty() {
        return 0.0;
    }
    let mut score = 0.0f32;
    for w in &ws {
        if POSITIVE.contains(&w.as_str()) {
            score += 1.0;
        } else if NEGATIVE.contains(&w.as_str()) {
            score -= 1.0;
        }
    }
    (score / ws.len() as f32 * 4.0).clamp(-1.0, 1.0)
}

/// Analyze one text sample and fold it into the rolling fingerprint.
pub fn fold_style(fp: &mut StyleFingerprint, text: &str) {
    let sents = sentences(text);
    if sents.is_empty() {
        return;
    }
    let mut lens = Vec::new();
    let mut clause = 0.0f32;
    let mut senti = Vec::new();
    let total_chars = text.chars().count().max(1);
    for s in &sents {
        let ws = words(s);
        lens.push(ws.len() as f32);
        clause += ws
            .iter()
            .filter(|w| CONNECTIVES.contains(&w.as_str()))
            .count() as f32;
        senti.push(sentiment(s));
    }
    // lexical density: non-stopword fraction
    let all = words(text);
    let content = all.iter().filter(|w| !STOP.contains(&w.as_str())).count();
    let density = if all.is_empty() { 0.5 } else { content as f32 / all.len() as f32 };
    let n = sents.len() as f32;
    let mean = lens.iter().sum::<f32>() / n;
    let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / n;
    let s_mean = senti.iter().sum::<f32>() / n;
    let s_min = senti.iter().cloned().fold(f32::MAX, f32::min);
    let s_max = senti.iter().cloned().fold(f32::MIN, f32::max);
    let quoted = text.matches('"').count() / 2;
    let dialogue = quoted as f32 * 2.0 * 24.0 / total_chars as f32; // ~2×24 chars per quote pair
    let dialogue = dialogue.min(1.0);
    let m = fp.samples as f32;
    fp.sentence_len_mean = (fp.sentence_len_mean * m + mean) / (m + 1.0);
    fp.sentence_len_std = (fp.sentence_len_std * m + var.sqrt()) / (m + 1.0);
    fp.lexical_density = (fp.lexical_density * m + density) / (m + 1.0);
    fp.clause_complexity = (fp.clause_complexity * m + clause / n) / (m + 1.0);
    fp.metaphor_load = (fp.metaphor_load * m + 0.02) / (m + 1.0); // placeholder, refined in M4
    fp.sentiment_mean = (fp.sentiment_mean * m + s_mean) / (m + 1.0);
    fp.sentiment_range = (fp.sentiment_range * m + (s_max - s_min)) / (m + 1.0);
    fp.dialogue_ratio = (fp.dialogue_ratio * m + dialogue) / (m + 1.0);
    fp.samples += 1;
}

// --- continuity ledger -------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityFacts {
    pub name: String,
    pub first_seen: u64,
    pub last_seen: u64,
    pub mentions: u32,
    pub properties: Vec<(String, String, u64)>, // (property, value, tick)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContinuityFlag {
    pub kind: String, // "property-conflict" | "timeline-conflict"
    pub entity: String,
    pub detail: String,
    pub at: u64,
    pub resolved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContinuityLedger {
    pub entities: Vec<EntityFacts>,
    pub flags: Vec<ContinuityFlag>,
}

/// Property-conflict pairs for simple contradiction detection.
const CONFLICT_PAIRS: [(&str, &str); 6] = [
    ("old", "new"),
    ("small", "large"),
    ("bright", "dark"),
    ("near", "far"),
    ("alive", "dead"),
    ("open", "closed"),
];

impl ContinuityLedger {
    /// Ingest a block; returns any new contradiction flags.
    pub fn ingest(&mut self, text: &str, tick: u64) -> Vec<ContinuityFlag> {
        let mut flags = Vec::new();
        // Entity extraction: capitalized words (proper nouns) + "the X" pattern.
        let mut entities: Vec<String> = Vec::new();
        let tokens: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .filter(|w| !w.is_empty())
            .collect();
        for (i, w) in tokens.iter().enumerate() {
            let lower = w.to_lowercase();
            let is_noise = lower == "the"
                || CONFLICT_PAIRS
                    .iter()
                    .any(|(a, b)| a == &lower || b == &lower);
            if !is_noise
                && w.chars().next().unwrap().is_uppercase()
                && w.len() > 2
                && w.chars().all(|c| c.is_alphabetic())
            {
                entities.push(lower.clone());
            }
            if !is_noise && i > 0 && tokens[i - 1].eq_ignore_ascii_case("the") && w.len() >= 3 {
                entities.push(lower);
            }
        }
        let ws = words(text);
        for name in entities {
            let mut e = self.entities.iter_mut().find(|e| e.name == name);
            let mentions = e.as_ref().map(|e| e.mentions).unwrap_or(0);
            if e.is_none() {
                self.entities.push(EntityFacts {
                    name: name.clone(),
                    first_seen: tick,
                    last_seen: tick,
                    mentions: 1,
                    properties: Vec::new(),
                });
                e = self.entities.iter_mut().find(|e| e.name == name);
            }
            if let Some(ef) = e {
                ef.mentions += 1;
                ef.last_seen = tick;
                // property detection: adjective adjacent to entity mention
                for (i, w) in ws.iter().enumerate() {
                    if *w == name && i > 0 {
                        let adj = ws[i - 1].clone();
                        if let Some((a, b)) = CONFLICT_PAIRS.iter().find(|(a, b)| *a == adj || *b == adj) {
                            let value = if *a == adj { *a } else { *b };
                            let (other, other_tick) = ef
                                .properties
                                .iter()
                                .rev()
                                .find(|(p, _, _)| p == a || p == b)
                                .map(|(p, _, t)| (p.clone(), *t))
                                .unwrap_or(("".to_string(), 0));
                            if !other.is_empty() && other != value {
                                let flag = ContinuityFlag {
                                    kind: "property-conflict".into(),
                                    entity: name.clone(),
                                    detail: format!(
                                        "{} described as both '{}' (t={}) and '{}' (t={})",
                                        name, other, other_tick, value, tick
                                    ),
                                    at: tick,
                                    resolved: false,
                                };
                                flags.push(flag.clone());
                                self.flags.push(flag);
                            } else if other.is_empty() {
                                ef.properties.push(((*a).to_string(), value.to_string(), tick));
                            }
                            let _ = mentions;
                            break;
                        }
                    }
                }
            }
        }
        flags
    }
}

// --- extraction pipeline -----------------------------------------------------

/// Result of running the pipeline over one writing session.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionReport {
    pub traces_bound: usize,
    pub nodes_updated: usize,
    pub style_samples: u32,
    pub entities_seen: usize,
    pub contradiction_flags: usize,
    pub preference_signals: usize,
}

/// Result of analyzing one written block (organ state updates + binding data).
pub struct BlockAnalysis {
    pub report: ExtractionReport,
    pub keywords: Vec<String>,
    pub sentiment: f32,
}

/// The writing organ's bridge into the substrate: turns document events into
/// pending percepts (bound by the normal binder), semantic nodes, style memory,
/// preference signals, and procedural habit signals. All local, all deterministic.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WritingOrgan {
    pub documents: Vec<Document>,
    pub ledger: ContinuityLedger,
    pub next_doc_id: u64,
    pub next_block_id: u64,
    pub preference_signals: Vec<(String, u64, u32)>, // (topic, tick, count)
}

impl WritingOrgan {
    pub fn new() -> Self {
        WritingOrgan {
            documents: Vec::new(),
            ledger: ContinuityLedger::default(),
            next_doc_id: 1,
            next_block_id: 1,
            preference_signals: Vec::new(),
        }
    }

    pub fn create_document(&mut self, title: &str, mode: DocMode, tick: u64) -> Document {
        let doc = Document {
            id: self.next_doc_id,
            title: title.to_string(),
            mode,
            blocks: Vec::new(),
            created: tick,
            updated: tick,
            style: StyleFingerprint::default(),
        };
        self.next_doc_id += 1;
        self.documents.push(doc.clone());
        doc
    }

    /// Append a block, update the rolling style fingerprint, run the continuity
    /// ledger, register preference signals, and return the binding data
    /// (keywords + sentiment) so the Brain can emit the percept.
    pub fn analyze_block(
        &mut self,
        doc_id: u64,
        kind: &str,
        text: &str,
        tick: u64,
    ) -> Option<BlockAnalysis> {
        let mut report = ExtractionReport::default();
        let doc = self.documents.iter_mut().find(|d| d.id == doc_id)?;
        doc.blocks.push(DocBlock {
            id: self.next_block_id,
            kind: kind.to_string(),
            text: text.to_string(),
        });
        self.next_block_id += 1;
        doc.updated = tick;
        fold_style(&mut doc.style, text);
        report.style_samples = doc.style.samples;

        // Continuity ledger.
        let flags = self.ledger.ingest(text, tick);
        report.contradiction_flags = flags.len();
        report.entities_seen = self.ledger.entities.len();

        // Preference signals: topic = most frequent content word.
        let ws = words(text);
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for w in ws.iter().filter(|w| !STOP.contains(&w.as_str())) {
            *counts.entry(w.clone()).or_insert(0) += 1;
        }
        if let Some((topic, _)) = counts.iter().max_by(|a, b| a.1.cmp(b.1)) {
            match self.preference_signals.iter_mut().find(|(t, _, _)| t == topic) {
                Some(sig) => {
                    sig.1 = tick;
                    sig.2 += 1;
                }
                None => self.preference_signals.push((topic.clone(), tick, 1)),
            }
            report.preference_signals = 1;
        }

        let keywords = words(text);
        let sentiment = sentiment(text);
        Some(BlockAnalysis {
            report,
            keywords,
            sentiment,
        })
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for d in self.documents.iter() {
            for b in d.id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for blk in d.blocks.iter() {
                for b in blk.text.as_bytes() {
                    h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        for f in self.ledger.flags.iter() {
            for b in f.detail.as_bytes() {
                h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_fingerprint_distinguishes_prose() {
        let mut short = StyleFingerprint::default();
        let mut long = StyleFingerprint::default();
        for _ in 0..5 {
            fold_style(&mut short, "The cat slept. It dreamed. Morning came.");
            fold_style(&mut long, "The cat, having wandered the quiet streets all night and observed the slow turning of the world, finally slept beneath the old bridge, dreaming of warm kitchens and gentle hands.");
        }
        assert!(long.sentence_len_mean > short.sentence_len_mean * 2.0);
        assert!(long.clause_complexity > short.clause_complexity);
    }

    #[test]
    fn sentiment_is_bounded_and_directional() {
        assert!(sentiment("a warm kind gentle morning") > 0.3);
        assert!(sentiment("a cold cruel harsh night") < -0.3);
        assert!((-1.0..=1.0).contains(&sentiment("neutral factual statement")));
    }

    #[test]
    fn continuity_detects_property_conflicts() {
        let mut ledger = ContinuityLedger::default();
        ledger.ingest("The old Bridge spans the river.", 100);
        let flags = ledger.ingest("The new Bridge glows at night.", 200);
        assert_eq!(flags.len(), 1, "property conflict flagged");
        assert_eq!(flags[0].kind, "property-conflict");
        assert!(flags[0].detail.contains("bridge"), "detail: {}", flags[0].detail);
    }

    #[test]
    fn organ_writes_blocks_and_reports() {
        let mut organ = WritingOrgan::new();
        let doc = organ.create_document("The Garden", DocMode::Prose, 0);
        let analysis = organ
            .analyze_block(doc.id, "para", "The garden blooms with warm bright flowers.", 10)
            .unwrap();
        assert_eq!(analysis.report.style_samples, 1);
        assert!(organ.ledger.entities.iter().any(|e| e.name == "garden"));
        assert_eq!(organ.documents[0].blocks.len(), 1);
        assert!(analysis.sentiment > 0.0);
        assert!(analysis.keywords.contains(&"garden".to_string()));
    }
}
