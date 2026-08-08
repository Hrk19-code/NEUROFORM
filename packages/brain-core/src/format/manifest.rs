//! Manifest (DESIGN.md §16.1) and shard index entries.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MigrationEntry {
    pub from: String,
    pub to: String,
    pub at: u64,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RawVaultRef {
    pub enabled: bool,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub format: String,
    pub version: String,
    pub brain_id: String,
    pub created_at: u64,
    pub last_opened_at: u64,
    pub seed: u64,
    pub rng_state: u64,
    #[serde(default)]
    pub event_counter: u64,
    #[serde(default)]
    pub dropped_events: u64,
    #[serde(default)]
    pub sleep_pressure: f32,
    #[serde(default)]
    pub sleep_emotional_load: f32,
    #[serde(default)]
    pub autonomy_enabled: bool,
    #[serde(default)]
    pub autonomy_quiet_start: u8,
    #[serde(default)]
    pub autonomy_quiet_end: u8,
    pub capacity_tier: String,
    pub migration_chain: Vec<MigrationEntry>,
    pub raw_vault_ref: RawVaultRef,
    pub capacity: serde_json::Value,
    /// Lineage (who this file came from) — data, not behavior.
    #[serde(default)]
    pub lineage: crate::brain::Lineage,
    /// Feature encoder chosen at creation: "" (handcrafted) | "onnx" | "jepa".
    /// Immutable for the file's life (BUILD-THE-BODY Phase 0). Skipped when
    /// empty so pre-encoder files round-trip byte-identically.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub encoder: String,
    /// sha256 of the frozen encoder model (only when encoder != "").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoder_model_sha256: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShardIndexEntry {
    pub id: String,
    pub shard_type: String,
    pub offset: u64,
    pub length: u64,
    pub compression: String, // "none" (zstd arrives post-M0)
    pub checksum: String,    // hex, BLAKE2b-256 over stored bytes
    pub encrypted: bool,
    pub nonce: Option<String>,
    pub schema_version: u32,
}
