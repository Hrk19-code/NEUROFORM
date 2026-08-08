//! Key envelope + per-shard encryption (DESIGN.md §16.1, §16.2).
//!
//! Passphrase mode: Argon2id (m=64 MiB, t=3, p=4 — spec values) derives a KDF key
//! from the passphrase + random salt; the KDF key wraps a random 32-byte DEK
//! (XChaCha20-Poly1305); shard payloads are encrypted with the DEK, each with its
//! own 24-byte nonce.
//!
//! Plain-dev mode (no passphrase): DEK stored hex in the envelope. This exists so
//! M0 tests and scripts can run without interactive secrets; the OS-keychain slot
//! and full passphrase enforcement arrive in M1. Plain-dev files carry
//! `mode: "plain-dev"` and are explicitly flagged on load.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};

use crate::format::FormatError;
use crate::rng::Rng;

pub const DEK_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const SALT_LEN: usize = 16;
pub const ARGON_M_KIB: u32 = 65_536; // 64 MiB
pub const ARGON_T: u32 = 3;
pub const ARGON_P: u32 = 4;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyEnvelope {
    pub mode: String, // "passphrase" | "plain-dev"
    pub kdf: Option<String>,
    pub kdf_params: Option<KdfParams>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub wrapped_dek: Option<String>,
    pub dek_hex: Option<String>,
}

pub fn build_envelope(passphrase: Option<&str>, rng: &mut Rng) -> (KeyEnvelope, [u8; DEK_LEN]) {
    let mut dek = [0u8; DEK_LEN];
    rng.fill_bytes(&mut dek);
    match passphrase {
        Some(pw) if !pw.is_empty() => {
            let mut salt = [0u8; SALT_LEN];
            rng.fill_bytes(&mut salt);
            let params = Params::new(ARGON_M_KIB, ARGON_T, ARGON_P, Some(DEK_LEN))
                .expect("valid argon2 params");
            let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut kdf_key = [0u8; DEK_LEN];
            argon
                .hash_password_into(pw.as_bytes(), &salt, &mut kdf_key)
                .expect("argon2 hashing");
            let mut env_nonce = [0u8; NONCE_LEN];
            rng.fill_bytes(&mut env_nonce);
            let cipher = XChaCha20Poly1305::new_from_slice(&kdf_key).expect("valid key length");
            let wrapped = cipher
                .encrypt(XNonce::from_slice(&env_nonce), dek.as_slice())
                .expect("envelope encryption");
            (
                KeyEnvelope {
                    mode: "passphrase".to_string(),
                    kdf: Some("argon2id".to_string()),
                    kdf_params: Some(KdfParams {
                        m_kib: ARGON_M_KIB,
                        t: ARGON_T,
                        p: ARGON_P,
                    }),
                    salt: Some(hex::encode(salt)),
                    nonce: Some(hex::encode(env_nonce)),
                    wrapped_dek: Some(hex::encode(wrapped)),
                    dek_hex: None,
                },
                dek,
            )
        }
        _ => (
            KeyEnvelope {
                mode: "plain-dev".to_string(),
                kdf: None,
                kdf_params: None,
                salt: None,
                nonce: None,
                wrapped_dek: None,
                dek_hex: Some(hex::encode(dek)),
            },
            dek,
        ),
    }
}

pub fn unwrap_envelope(env: &KeyEnvelope, passphrase: Option<&str>) -> Result<[u8; DEK_LEN], FormatError> {
    match env.mode.as_str() {
        "plain-dev" => {
            let hex_str = env.dek_hex.as_deref().ok_or(FormatError::Key("plain-dev envelope missing dek".into()))?;
            let dek = hex::decode(hex_str).map_err(|_| FormatError::Key("bad dek hex".into()))?;
            if dek.len() != DEK_LEN {
                return Err(FormatError::Key("dek wrong length".into()));
            }
            let mut out = [0u8; DEK_LEN];
            out.copy_from_slice(&dek);
            Ok(out)
        }
        "passphrase" => {
            let pw = passphrase.ok_or(FormatError::WrongPassphrase)?;
            let p = env.kdf_params.as_ref().ok_or(FormatError::Key("missing kdf params".into()))?;
            let params = Params::new(p.m_kib, p.t, p.p, Some(DEK_LEN)).map_err(|_| FormatError::Key("bad kdf params".into()))?;
            let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let salt = hex::decode(env.salt.as_deref().ok_or(FormatError::Key("missing salt".into()))?)
                .map_err(|_| FormatError::Key("bad salt hex".into()))?;
            let env_nonce = hex::decode(env.nonce.as_deref().ok_or(FormatError::Key("missing nonce".into()))?)
                .map_err(|_| FormatError::Key("bad nonce hex".into()))?;
            let wrapped = hex::decode(env.wrapped_dek.as_deref().ok_or(FormatError::Key("missing wrapped dek".into()))?)
                .map_err(|_| FormatError::Key("bad wrapped dek hex".into()))?;
            let mut kdf_key = [0u8; DEK_LEN];
            argon
                .hash_password_into(pw.as_bytes(), &salt, &mut kdf_key)
                .map_err(|_| FormatError::Key("argon2 failed".into()))?;
            let cipher = XChaCha20Poly1305::new_from_slice(&kdf_key).expect("valid key length");
            let dek = cipher
                .decrypt(XNonce::from_slice(&env_nonce), wrapped.as_slice())
                .map_err(|_| FormatError::WrongPassphrase)?;
            if dek.len() != DEK_LEN {
                return Err(FormatError::Key("dek wrong length".into()));
            }
            let mut out = [0u8; DEK_LEN];
            out.copy_from_slice(&dek);
            Ok(out)
        }
        other => Err(FormatError::Key(format!("unknown envelope mode: {other}"))),
    }
}

/// Encrypt a shard payload with the DEK; returns (ciphertext, nonce_hex).
pub fn encrypt_payload(dek: &[u8; DEK_LEN], payload: &[u8], rng: &mut Rng) -> (Vec<u8>, String) {
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill_bytes(&mut nonce);
    let cipher = XChaCha20Poly1305::new_from_slice(dek).expect("valid key length");
    let ct = cipher
        .encrypt(XNonce::from_slice(&nonce), payload)
        .expect("payload encryption");
    (ct, hex::encode(nonce))
}

pub fn decrypt_payload(dek: &[u8; DEK_LEN], ciphertext: &[u8], nonce_hex: &str) -> Result<Vec<u8>, FormatError> {
    let nonce = hex::decode(nonce_hex).map_err(|_| FormatError::Key("bad nonce hex".into()))?;
    if nonce.len() != NONCE_LEN {
        return Err(FormatError::Key("nonce wrong length".into()));
    }
    let cipher = XChaCha20Poly1305::new_from_slice(dek).expect("valid key length");
    cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext)
        .map_err(|_| FormatError::Decrypt)
}
