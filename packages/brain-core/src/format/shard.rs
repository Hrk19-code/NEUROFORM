//! Shard payload encodings (DESIGN.md §16.3).
//!
//! STATE shard: `[u32 envelope_len LE][envelope JSON][float32 LE × dim]` where
//! dim = named (26) + reserved. The envelope carries sim_time/dim/schema_version;
//! floats carry the whole-brain vector g.

use crate::format::FormatError;
use crate::state::{GlobalState, N_NAMED};

pub const STATE_SHARD_ID: &str = "STATE";
pub const MODULATORS_SHARD_ID: &str = "MODULATORS";
pub const EPISODIC_SHARD_ID: &str = "EPISODIC";
pub const SEMANTIC_SHARD_ID: &str = "SEMANTIC";
pub const HORMONE_SHARD_ID: &str = "HORMONE";
pub const DREAMS_SHARD_ID: &str = "DREAMS";
pub const DOCS_SHARD_ID: &str = "DOCS";
pub const DRAW_SHARD_ID: &str = "DRAW";
pub const VOICE_SHARD_ID: &str = "VOICE";
pub const BODY_SHARD_ID: &str = "BODY";
pub const NET_SHARD_ID: &str = "NET";
pub const PHYS_SHARD_ID: &str = "PHYS";

pub fn encode_state_shard(state: &GlobalState) -> Vec<u8> {
    let envelope = serde_json::json!({
        "schemaVersion": 1,
        "kind": "state",
        "simTime": state.sim_time,
        "dim": state.dim,
        "namedCount": N_NAMED,
    });
    let env = serde_json::to_vec(&envelope).expect("state envelope serialization");
    let g = state.g();
    let mut out = Vec::with_capacity(4 + env.len() + g.len() * 4);
    out.extend_from_slice(&(env.len() as u32).to_le_bytes());
    out.extend_from_slice(&env);
    for x in g {
        out.extend_from_slice(&x.to_bits().to_le_bytes());
    }
    out
}

pub fn decode_state_shard(bytes: &[u8]) -> Result<(u64, usize, Vec<f32>), FormatError> {
    if bytes.len() < 4 {
        return Err(FormatError::Corrupt("STATE shard too short".into()));
    }
    let env_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    if 4 + env_len > bytes.len() {
        return Err(FormatError::Corrupt("STATE shard envelope length out of range".into()));
    }
    let env: serde_json::Value = serde_json::from_slice(&bytes[4..4 + env_len])
        .map_err(|e| FormatError::Json(e.to_string()))?;
    let rest = &bytes[4 + env_len..];
    if rest.len() % 4 != 0 {
        return Err(FormatError::Corrupt("STATE shard float section misaligned".into()));
    }
    let floats: Vec<f32> = rest
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let sim_time = env["simTime"].as_u64().unwrap_or(0);
    let dim = env["dim"].as_u64().unwrap_or(0) as usize;
    Ok((sim_time, dim, floats))
}

pub fn encode_modulators_shard(axes: &[crate::modulators::AxisSnapshot]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "modulators",
        "axes": axes,
    }))
    .expect("modulator envelope serialization")
}

pub fn decode_modulators_shard(
    bytes: &[u8],
) -> Result<Vec<crate::modulators::AxisSnapshot>, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let axes: Vec<crate::modulators::AxisSnapshot> =
        serde_json::from_value(v["axes"].clone()).map_err(|e| FormatError::Json(e.to_string()))?;
    Ok(axes)
}

pub fn encode_episodic_shard(traces: &[crate::memory::EpisodicTrace]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "episodic",
        "traces": traces,
    }))
    .expect("episodic shard serialization")
}

pub fn decode_episodic_shard(
    bytes: &[u8],
) -> Result<Vec<crate::memory::EpisodicTrace>, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["traces"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_semantic_shard(
    nodes: &[crate::semantic::SemanticNode],
    edges: &[crate::semantic::SemanticEdge],
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "semantic",
        "nodes": nodes,
        "edges": edges,
    }))
    .expect("semantic shard serialization")
}

pub fn decode_semantic_shard(
    bytes: &[u8],
) -> Result<(Vec<crate::semantic::SemanticNode>, Vec<crate::semantic::SemanticEdge>), FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let nodes: Vec<crate::semantic::SemanticNode> =
        serde_json::from_value(v["nodes"].clone()).map_err(|e| FormatError::Json(e.to_string()))?;
    let edges: Vec<crate::semantic::SemanticEdge> =
        serde_json::from_value(v["edges"].clone()).map_err(|e| FormatError::Json(e.to_string()))?;
    Ok((nodes, edges))
}

pub fn encode_hormone_shard(profile: &crate::embodiment::HormoneProfile) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "hormone",
        "profile": profile,
    }))
    .expect("hormone shard serialization")
}

pub fn decode_hormone_shard(bytes: &[u8]) -> Result<crate::embodiment::HormoneProfile, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["profile"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_dreams_shard(
    dreams: &[crate::sleep::DreamLogEntry],
    reports: &[crate::sleep::SleepReport],
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "dreams",
        "dreams": dreams,
        "sleepReports": reports,
    }))
    .expect("dreams shard serialization")
}

pub fn decode_dreams_shard(
    bytes: &[u8],
) -> Result<(Vec<crate::sleep::DreamLogEntry>, Vec<crate::sleep::SleepReport>), FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let dreams: Vec<crate::sleep::DreamLogEntry> =
        serde_json::from_value(v["dreams"].clone()).map_err(|e| FormatError::Json(e.to_string()))?;
    let reports: Vec<crate::sleep::SleepReport> =
        serde_json::from_value(v["sleepReports"].clone())
            .map_err(|e| FormatError::Json(e.to_string()))?;
    Ok((dreams, reports))
}

pub fn encode_docs_shard(organ: &crate::writing::WritingOrgan) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "docs",
        "organ": organ,
    }))
    .expect("docs shard serialization")
}

pub fn decode_docs_shard(bytes: &[u8]) -> Result<crate::writing::WritingOrgan, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_draw_shard(organ: &crate::drawing::DrawingOrgan) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "draw",
        "organ": organ,
    }))
    .expect("draw shard serialization")
}

pub fn decode_draw_shard(bytes: &[u8]) -> Result<crate::drawing::DrawingOrgan, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_voice_shard(organ: &crate::voice::VoiceOrgan) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "voice",
        "organ": organ,
    }))
    .expect("voice shard serialization")
}

pub fn decode_voice_shard(bytes: &[u8]) -> Result<crate::voice::VoiceOrgan, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_body_shard(organ: &crate::body::BodyOrgan) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "body",
        "organ": organ,
    }))
    .expect("body shard serialization")
}

pub fn decode_body_shard(bytes: &[u8]) -> Result<crate::body::BodyOrgan, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_network_shard(organ: &crate::network::NetworkOrgan) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "network",
        "organ": organ,
    }))
    .expect("network shard serialization")
}

pub fn decode_network_shard(bytes: &[u8]) -> Result<crate::network::NetworkOrgan, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}

pub fn encode_physics_shard(p: &crate::physics::PhysicsLearner) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "kind": "physics",
        "organ": p,
    }))
    .unwrap_or_default()
}

pub fn decode_physics_shard(bytes: &[u8]) -> Result<crate::physics::PhysicsLearner, FormatError> {
    let v: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    serde_json::from_value(v["organ"].clone()).map_err(|e| FormatError::Json(e.to_string()))
}
