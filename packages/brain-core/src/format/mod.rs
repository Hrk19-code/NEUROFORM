//! NF1 `.brain` file writer/reader/verifier (DESIGN.md §16).
//!
//! Section order (fixed, length-stable — the shard index comes *last* so no
//! section's length depends on another section's offset values):
//!
//!   [header 0x100][key envelope][manifest][shard 1..N][shard index]
//!
//! Write path: prepare shard payloads → build key envelope → encrypt shards →
//! assemble → temp file → fsync → atomic rename. Read path: header CRC →
//! envelope unwrap → per-shard bounds + BLAKE2b-256 checksum → decrypt → decode.
//! Checksums cover the *stored* bytes (ciphertext), so corruption at rest is
//! caught before decryption is attempted.
//!
//! M0 deviations from DESIGN.md §16 (documented in docs/M0-NOTES.md):
//! checksums are BLAKE2b-256 (BLAKE3 deferred), compression is always "none",
//! the Ed25519 signature tail is absent (sig fields zeroed), and the key
//! envelope supports "plain-dev" mode when no passphrase is given.

pub mod crypto;
pub mod header;
pub mod manifest;
pub mod shard;

use std::fs;
use std::io::Write;
use std::path::Path;

use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;

use crate::capacity::CapacityLedger;
use crate::modulators::ModulatorSystem;
use crate::rng::Rng;
use crate::state::GlobalState;

pub use manifest::{Manifest, MigrationEntry, RawVaultRef, ShardIndexEntry};

#[derive(Debug)]
pub enum FormatError {
    Io(std::io::Error),
    Header(String),
    UnsupportedVersion(u32),
    Key(String),
    WrongPassphrase,
    Decrypt,
    Json(String),
    Corrupt(String),
    SizeMismatch { declared: u64, actual: u64 },
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Io(e) => write!(f, "io: {e}"),
            FormatError::Header(m) => write!(f, "header: {m}"),
            FormatError::UnsupportedVersion(v) => write!(f, "unsupported format version {v}"),
            FormatError::Key(m) => write!(f, "key: {m}"),
            FormatError::WrongPassphrase => write!(f, "wrong passphrase or corrupted key envelope"),
            FormatError::Decrypt => write!(f, "payload decryption failed"),
            FormatError::Json(m) => write!(f, "json: {m}"),
            FormatError::Corrupt(m) => write!(f, "corrupt: {m}"),
            FormatError::SizeMismatch { declared, actual } => {
                write!(f, "size mismatch: declared {declared}, actual {actual}")
            }
        }
    }
}

impl std::error::Error for FormatError {}

impl From<std::io::Error> for FormatError {
    fn from(e: std::io::Error) -> Self {
        FormatError::Io(e)
    }
}

/// BLAKE2b-256 hex (M0 checksum; DESIGN.md specifies BLAKE3 — see M0 notes).
pub fn checksum_hex(data: &[u8]) -> String {
    let mut hasher = Blake2bVar::new(32).expect("blake2b-256");
    hasher.update(data);
    let mut buf = [0u8; 32];
    hasher.finalize_variable(&mut buf).expect("blake2b finalize");
    hex::encode(buf)
}

/// A prepared shard: plaintext payload + metadata (before encryption).
#[derive(Clone)]
pub struct PreparedShard {
    pub id: String,
    pub shard_type: String,
    pub payload: Vec<u8>,
    pub schema_version: u32,
}

pub struct FileContents {
    pub manifest: Manifest,
    pub state: (u64, usize, Vec<f32>), // (sim_time, dim, g floats)
    pub modulators: Vec<crate::modulators::AxisSnapshot>,
    pub episodic: Option<Vec<crate::memory::EpisodicTrace>>,
    pub semantic: Option<(
        Vec<crate::semantic::SemanticNode>,
        Vec<crate::semantic::SemanticEdge>,
    )>,
    pub hormone: Option<crate::embodiment::HormoneProfile>,
    pub dreams: Option<(Vec<crate::sleep::DreamLogEntry>, Vec<crate::sleep::SleepReport>)>,
    pub writing: Option<crate::writing::WritingOrgan>,
    pub drawing: Option<crate::drawing::DrawingOrgan>,
    pub voice: Option<crate::voice::VoiceOrgan>,
    pub body: Option<crate::body::BodyOrgan>,
    pub network: Option<crate::network::NetworkOrgan>,
    pub physics: Option<crate::physics::PhysicsLearner>,
    pub capacity: CapacityLedger,
    pub corrupt: Vec<String>,
}

pub struct VerifyReport {
    pub ok: bool,
    pub manifest: Manifest,
    pub shard_checks: Vec<(String, bool, String)>, // (id, verified, detail)
    pub corrupt: Vec<String>,
    pub envelope_mode: String,
}

/// Build the M8 shards (STATE, MODULATORS, EPISODIC, SEMANTIC, HORMONE, DREAMS, DOCS, DRAW, VOICE, BODY, NET, PHYS).
pub fn prepare_shards(
    state: &GlobalState,
    mods: &ModulatorSystem,
    episodic: &crate::memory::EpisodicStore,
    semantic: &crate::semantic::SemanticStore,
    hormone: &crate::embodiment::HormoneProfile,
    dreams: &[crate::sleep::DreamLogEntry],
    sleep_reports: &[crate::sleep::SleepReport],
    writing: &crate::writing::WritingOrgan,
    drawing: &crate::drawing::DrawingOrgan,
    voice: &crate::voice::VoiceOrgan,
    body: &crate::body::BodyOrgan,
    network: &crate::network::NetworkOrgan,
    physics: &crate::physics::PhysicsLearner,
) -> Vec<PreparedShard> {
    vec![
        PreparedShard {
            id: shard::STATE_SHARD_ID.to_string(),
            shard_type: "state".to_string(),
            payload: shard::encode_state_shard(state),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::MODULATORS_SHARD_ID.to_string(),
            shard_type: "modulators".to_string(),
            payload: shard::encode_modulators_shard(&mods.snapshot()),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::EPISODIC_SHARD_ID.to_string(),
            shard_type: "episodic".to_string(),
            payload: shard::encode_episodic_shard(&episodic.traces),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::SEMANTIC_SHARD_ID.to_string(),
            shard_type: "semantic".to_string(),
            payload: shard::encode_semantic_shard(&semantic.nodes, &semantic.edges),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::HORMONE_SHARD_ID.to_string(),
            shard_type: "hormone".to_string(),
            payload: shard::encode_hormone_shard(hormone),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::DREAMS_SHARD_ID.to_string(),
            shard_type: "dreams".to_string(),
            payload: shard::encode_dreams_shard(dreams, sleep_reports),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::DOCS_SHARD_ID.to_string(),
            shard_type: "docs".to_string(),
            payload: shard::encode_docs_shard(writing),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::DRAW_SHARD_ID.to_string(),
            shard_type: "draw".to_string(),
            payload: shard::encode_draw_shard(drawing),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::VOICE_SHARD_ID.to_string(),
            shard_type: "voice".to_string(),
            payload: shard::encode_voice_shard(voice),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::BODY_SHARD_ID.to_string(),
            shard_type: "body".to_string(),
            payload: shard::encode_body_shard(body),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::NET_SHARD_ID.to_string(),
            shard_type: "network".to_string(),
            payload: shard::encode_network_shard(network),
            schema_version: 1,
        },
        PreparedShard {
            id: shard::PHYS_SHARD_ID.to_string(),
            shard_type: "physics".to_string(),
            payload: shard::encode_physics_shard(physics),
            schema_version: 1,
        },
    ]
}

/// Serialize a manifest to JSON bytes.
pub fn manifest_bytes(manifest: &Manifest) -> Vec<u8> {
    serde_json::to_vec(manifest).expect("manifest serialization")
}

/// Write a `.brain` file atomically (temp + fsync + rename). Returns total bytes.
pub fn write_file(
    path: &Path,
    manifest: &Manifest,
    shards: &[PreparedShard],
    passphrase: Option<&str>,
) -> Result<u64, FormatError> {
    let mut rng = Rng::new(manifest.seed ^ 0x5EED_CAFE);
    let (envelope, dek) = crypto::build_envelope(passphrase, &mut rng);
    let envelope_bytes =
        serde_json::to_vec(&envelope).map_err(|e| FormatError::Json(e.to_string()))?;
    let manifest_bytes = manifest_bytes(manifest);

    // Encrypt all shards first (ciphertext sizes drive the layout).
    let mut encrypted: Vec<(Vec<u8>, String)> = Vec::with_capacity(shards.len()); // (ct, nonce)
    for s in shards {
        let (ct, nonce) = crypto::encrypt_payload(&dek, &s.payload, &mut rng);
        encrypted.push((ct, nonce));
    }

    // Layout: header | envelope | manifest | shards | shard index
    let mut offset: u64 = header::HEADER_LEN as u64;
    let keyenv_off = offset;
    offset += envelope_bytes.len() as u64;
    let manifest_off = offset;
    offset += manifest_bytes.len() as u64;
    let mut shard_offsets: Vec<u64> = Vec::with_capacity(encrypted.len());
    for (ct, _) in encrypted.iter() {
        shard_offsets.push(offset);
        offset += ct.len() as u64;
    }
    let shardidx_off = offset;

    let entries: Vec<ShardIndexEntry> = shards
        .iter()
        .zip(encrypted.iter())
        .zip(shard_offsets.iter())
        .map(|((s, (ct, nonce)), off)| ShardIndexEntry {
            id: s.id.clone(),
            shard_type: s.shard_type.clone(),
            offset: *off,
            length: ct.len() as u64,
            compression: "none".to_string(),
            checksum: checksum_hex(ct),
            encrypted: true,
            nonce: Some(nonce.clone()),
            schema_version: s.schema_version,
        })
        .collect();
    let shard_index_bytes =
        serde_json::to_vec(&entries).map_err(|e| FormatError::Json(e.to_string()))?;
    let total_size = offset + shard_index_bytes.len() as u64;

    let header = header::Header {
        version: header::FORMAT_VERSION,
        total_size,
        manifest_off,
        manifest_len: manifest_bytes.len() as u64,
        keyenv_off,
        keyenv_len: envelope_bytes.len() as u64,
        shardidx_off,
        shardidx_len: shard_index_bytes.len() as u64,
        sig_off: 0,
        sig_len: 0,
    };

    // Assemble into temp file, fsync, atomic rename.
    let tmp_path = tmp_path_for(path);
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(&header.encode())?;
        f.write_all(&envelope_bytes)?;
        f.write_all(&manifest_bytes)?;
        for (ct, _) in encrypted.iter() {
            f.write_all(ct)?;
        }
        f.write_all(&shard_index_bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(total_size)
}

fn tmp_path_for(path: &Path) -> std::path::PathBuf {
    let mut os = path.as_os_str().to_owned();
    os.push(".tmp");
    std::path::PathBuf::from(os)
}

/// Read and verify a `.brain` file. Corrupt non-essential shards are reported in
/// `corrupt`; a corrupt STATE shard or manifest fails the load.
pub fn read_file(path: &Path, passphrase: Option<&str>) -> Result<FileContents, FormatError> {
    let bytes = fs::read(path)?;
    let h = header::Header::decode(&bytes)?;
    if h.total_size != bytes.len() as u64 {
        return Err(FormatError::SizeMismatch {
            declared: h.total_size,
            actual: bytes.len() as u64,
        });
    }
    let envelope_bytes = slice(&bytes, h.keyenv_off, h.keyenv_len)?;
    let envelope: crypto::KeyEnvelope =
        serde_json::from_slice(envelope_bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let dek = crypto::unwrap_envelope(&envelope, passphrase)?;
    let manifest_bytes = slice(&bytes, h.manifest_off, h.manifest_len)?;
    let manifest: Manifest =
        serde_json::from_slice(manifest_bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let idx_bytes = slice(&bytes, h.shardidx_off, h.shardidx_len)?;
    let entries: Vec<ShardIndexEntry> =
        serde_json::from_slice(idx_bytes).map_err(|e| FormatError::Json(e.to_string()))?;

    let capacity: CapacityLedger = serde_json::from_value(manifest.capacity.clone())
        .map_err(|e| FormatError::Json(e.to_string()))?;

    let mut corrupt: Vec<String> = Vec::new();
    let mut state: Option<(u64, usize, Vec<f32>)> = None;
    let mut modulators: Option<Vec<crate::modulators::AxisSnapshot>> = None;
    let mut episodic: Option<Vec<crate::memory::EpisodicTrace>> = None;
    let mut semantic: Option<(
        Vec<crate::semantic::SemanticNode>,
        Vec<crate::semantic::SemanticEdge>,
    )> = None;
    let mut hormone: Option<crate::embodiment::HormoneProfile> = None;
    let mut dreams: Option<(Vec<crate::sleep::DreamLogEntry>, Vec<crate::sleep::SleepReport>)> = None;
    let mut writing: Option<crate::writing::WritingOrgan> = None;
    let mut drawing: Option<crate::drawing::DrawingOrgan> = None;
    let mut voice: Option<crate::voice::VoiceOrgan> = None;
    let mut body: Option<crate::body::BodyOrgan> = None;
    let mut network: Option<crate::network::NetworkOrgan> = None;
    let mut physics: Option<crate::physics::PhysicsLearner> = None;

    for entry in &entries {
        let stored = match slice(&bytes, entry.offset, entry.length) {
            Ok(s) => s,
            Err(_) => {
                corrupt.push(format!("{}: out of bounds", entry.id));
                continue;
            }
        };
        if checksum_hex(stored) != entry.checksum {
            corrupt.push(format!("{}: checksum mismatch", entry.id));
            continue;
        }
        let plain = match crypto::decrypt_payload(
            &dek,
            stored,
            entry.nonce.as_deref().unwrap_or(""),
        ) {
            Ok(p) => p,
            Err(_) => {
                corrupt.push(format!("{}: decryption failed", entry.id));
                continue;
            }
        };
        match entry.shard_type.as_str() {
            "state" => match shard::decode_state_shard(&plain) {
                Ok(s) => state = Some(s),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "modulators" => match shard::decode_modulators_shard(&plain) {
                Ok(m) => modulators = Some(m),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "episodic" => match shard::decode_episodic_shard(&plain) {
                Ok(t) => episodic = Some(t),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "semantic" => match shard::decode_semantic_shard(&plain) {
                Ok(s) => semantic = Some(s),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "hormone" => match shard::decode_hormone_shard(&plain) {
                Ok(h) => hormone = Some(h),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "dreams" => match shard::decode_dreams_shard(&plain) {
                Ok(d) => dreams = Some(d),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "docs" => match shard::decode_docs_shard(&plain) {
                Ok(w) => writing = Some(w),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "draw" => match shard::decode_draw_shard(&plain) {
                Ok(d) => drawing = Some(d),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "voice" => match shard::decode_voice_shard(&plain) {
                Ok(v) => voice = Some(v),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "body" => match shard::decode_body_shard(&plain) {
                Ok(v) => body = Some(v),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "network" => match shard::decode_network_shard(&plain) {
                Ok(v) => network = Some(v),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            "physics" => match shard::decode_physics_shard(&plain) {
                Ok(v) => physics = Some(v),
                Err(e) => corrupt.push(format!("{}: {}", entry.id, e)),
            },
            other => corrupt.push(format!("{}: unknown shard type {other}", entry.id)),
        }
    }

    let state = state
        .ok_or_else(|| FormatError::Corrupt("STATE shard missing or unreadable".into()))?;
    let modulators = modulators
        .ok_or_else(|| FormatError::Corrupt("MODULATORS shard missing or unreadable".into()))?;

    Ok(FileContents {
        manifest,
        state,
        modulators,
        episodic,
        semantic,
        hormone,
        dreams,
        writing,
        drawing,
        voice,
        body,
        network,
        physics,
        capacity,
        corrupt,
    })
}

/// Structural verification without full decode (used by `verify`).
pub fn verify_file(path: &Path, passphrase: Option<&str>) -> Result<VerifyReport, FormatError> {
    let bytes = fs::read(path)?;
    let h = header::Header::decode(&bytes)?;
    if h.total_size != bytes.len() as u64 {
        return Err(FormatError::SizeMismatch {
            declared: h.total_size,
            actual: bytes.len() as u64,
        });
    }
    let envelope_bytes = slice(&bytes, h.keyenv_off, h.keyenv_len)?;
    let envelope: crypto::KeyEnvelope =
        serde_json::from_slice(envelope_bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let _dek = crypto::unwrap_envelope(&envelope, passphrase)?;
    let manifest_bytes = slice(&bytes, h.manifest_off, h.manifest_len)?;
    let manifest: Manifest =
        serde_json::from_slice(manifest_bytes).map_err(|e| FormatError::Json(e.to_string()))?;
    let idx_bytes = slice(&bytes, h.shardidx_off, h.shardidx_len)?;
    let entries: Vec<ShardIndexEntry> =
        serde_json::from_slice(idx_bytes).map_err(|e| FormatError::Json(e.to_string()))?;

    let mut checks = Vec::new();
    let mut corrupt = Vec::new();
    for entry in &entries {
        let stored = match slice(&bytes, entry.offset, entry.length) {
            Ok(s) => s,
            Err(_) => {
                corrupt.push(entry.id.clone());
                checks.push((entry.id.clone(), false, "out of bounds".into()));
                continue;
            }
        };
        let ok = checksum_hex(stored) == entry.checksum;
        if !ok {
            corrupt.push(entry.id.clone());
        }
        checks.push((
            entry.id.clone(),
            ok,
            if ok {
                "checksum ok".into()
            } else {
                "checksum mismatch".into()
            },
        ));
    }
    Ok(VerifyReport {
        ok: corrupt.is_empty(),
        manifest,
        shard_checks: checks,
        corrupt,
        envelope_mode: envelope.mode,
    })
}

fn slice<'a>(bytes: &'a [u8], offset: u64, len: u64) -> Result<&'a [u8], FormatError> {
    let start = offset as usize;
    let end = start
        .checked_add(len as usize)
        .ok_or_else(|| FormatError::Header("section overflow".into()))?;
    bytes
        .get(start..end)
        .ok_or_else(|| FormatError::Header("section out of bounds".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::Brain;
    use crate::capacity::TierName;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nf1_test_{tag}_{nanos}.brain"))
    }

    #[test]
    fn full_roundtrip_with_passphrase() {
        let path = tmp_path("pw");
        let mut brain = Brain::create(TierName::Standard, 42);
        brain.run_ticks(10_000);
        brain.save(&path, Some("hunter2")).unwrap();
        let loaded = Brain::load(&path, Some("hunter2")).unwrap();
        assert_eq!(brain.digest(), loaded.digest());
        assert_eq!(brain.brain_id, loaded.brain_id);
        assert!(loaded.capacity.tier == "standard");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn plain_dev_roundtrip() {
        let path = tmp_path("plain");
        let mut brain = Brain::create(TierName::Prototype, 7);
        brain.run_ticks(5_000);
        brain.save(&path, None).unwrap();
        let loaded = Brain::load(&path, None).unwrap();
        assert_eq!(brain.digest(), loaded.digest());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn wrong_passphrase_fails() {
        let path = tmp_path("wrongpw");
        let mut brain = Brain::create(TierName::Standard, 1);
        brain.run_ticks(100);
        brain.save(&path, Some("right")).unwrap();
        assert!(matches!(
            Brain::load(&path, Some("wrong")),
            Err(FormatError::WrongPassphrase)
        ));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn corruption_detected() {
        let path = tmp_path("corrupt");
        let mut brain = Brain::create(TierName::Standard, 2);
        brain.run_ticks(100);
        brain.save(&path, None).unwrap();
        let mut bytes = fs::read(&path).unwrap();
        let n = bytes.len();
        bytes[n - 1] ^= 0xFF; // last byte is inside the STATE shard ciphertext
        fs::write(&path, &bytes).unwrap();
        let res = Brain::load(&path, None);
        assert!(res.is_err(), "corrupt STATE shard must fail the load");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_reports_corrupt_shard() {
        let path = tmp_path("verify");
        let mut brain = Brain::create(TierName::Standard, 3);
        brain.run_ticks(100);
        brain.save(&path, None).unwrap();
        let report = verify_file(&path, None).unwrap();
        assert!(report.ok);
        assert_eq!(report.shard_checks.len(), 12, "STATE, MODULATORS, EPISODIC, SEMANTIC, HORMONE, DREAMS, DOCS, DRAW, VOICE, BODY, NET, PHYS");

        // Corrupt the MODULATORS shard and re-verify.
        let mut bytes = fs::read(&path).unwrap();
        let idx: Vec<ShardIndexEntry> = {
            let h = header::Header::decode(&bytes).unwrap();
            let idx_bytes = slice(&bytes, h.shardidx_off, h.shardidx_len).unwrap();
            serde_json::from_slice(idx_bytes).unwrap()
        };
        let mod_entry = idx.iter().find(|e| e.shard_type == "modulators").unwrap();
        let pos = mod_entry.offset as usize + 5;
        bytes[pos] ^= 0x01;
        fs::write(&path, &bytes).unwrap();
        let report = verify_file(&path, None).unwrap();
        assert!(!report.ok);
        assert!(report.corrupt.iter().any(|c| c.contains("MODULATORS")));
        fs::remove_file(&path).ok();
    }
}
