//! NF1 header (DESIGN.md §16.1).
//!
//! Layout (all little-endian):
//!   0x0000  8  magic "NF1BRAIN"   (8 bytes; the format version lives in the
//!                                  version field — DESIGN.md's table shows the
//!                                  magic plus an inline \x01, which would be
//!                                  9 bytes; we keep magic at exactly 8 and put
//!                                  version = 1 in the u32. See docs/M0-NOTES.md.)
//!   0x0008  4  format version u32 = 1
//!   0x000C  8  total file size u64
//!   0x0014  4  header CRC32C over bytes [0x0000, 0x0014)
//!   0x0018  8  manifest offset
//!   0x0020  8  manifest length
//!   0x0028  8  key envelope offset
//!   0x0030  8  key envelope length
//!   0x0038  8  shard index offset
//!   0x0040  8  shard index length
//!   0x0048  8  signature offset   (0 = none)
//!   0x0050  8  signature length   (0 = none)
//!   0x0058  168 reserved, zero-filled
//!   0x0100  sections begin

pub const MAGIC: [u8; 8] = *b"NF1BRAIN";
pub const FORMAT_VERSION: u32 = 1;
pub const HEADER_LEN: usize = 0x100;
pub const CRC_COVER_END: usize = 0x14; // CRC covers [0, 0x14)

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
    pub version: u32,
    pub total_size: u64,
    pub manifest_off: u64,
    pub manifest_len: u64,
    pub keyenv_off: u64,
    pub keyenv_len: u64,
    pub shardidx_off: u64,
    pub shardidx_len: u64,
    pub sig_off: u64,
    pub sig_len: u64,
}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut b = [0u8; HEADER_LEN];
        b[0..8].copy_from_slice(&MAGIC);
        b[8..12].copy_from_slice(&self.version.to_le_bytes());
        b[12..20].copy_from_slice(&self.total_size.to_le_bytes());
        let crc = crc32c(&b[0..CRC_COVER_END]);
        b[20..24].copy_from_slice(&crc.to_le_bytes());
        b[24..32].copy_from_slice(&self.manifest_off.to_le_bytes());
        b[32..40].copy_from_slice(&self.manifest_len.to_le_bytes());
        b[40..48].copy_from_slice(&self.keyenv_off.to_le_bytes());
        b[48..56].copy_from_slice(&self.keyenv_len.to_le_bytes());
        b[56..64].copy_from_slice(&self.shardidx_off.to_le_bytes());
        b[64..72].copy_from_slice(&self.shardidx_len.to_le_bytes());
        b[72..80].copy_from_slice(&self.sig_off.to_le_bytes());
        b[80..88].copy_from_slice(&self.sig_len.to_le_bytes());
        b
    }

    pub fn decode(bytes: &[u8]) -> Result<Header, crate::format::FormatError> {
        use crate::format::FormatError;
        if bytes.len() < HEADER_LEN {
            return Err(FormatError::Header("file shorter than header".into()));
        }
        if &bytes[0..8] != &MAGIC {
            return Err(FormatError::Header("bad magic".into()));
        }
        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        if version != FORMAT_VERSION {
            return Err(FormatError::UnsupportedVersion(version));
        }
        let stored_crc = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        let actual_crc = crc32c(&bytes[0..CRC_COVER_END]);
        if stored_crc != actual_crc {
            return Err(FormatError::Header("header CRC mismatch".into()));
        }
        Ok(Header {
            version,
            total_size: u64::from_le_bytes(bytes[12..20].try_into().unwrap()),
            manifest_off: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            manifest_len: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            keyenv_off: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            keyenv_len: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            shardidx_off: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            shardidx_len: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            sig_off: u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
            sig_len: u64::from_le_bytes(bytes[80..88].try_into().unwrap()),
        })
    }
}

/// CRC32C (Castagnoli) — table-driven, pure Rust, no C compiler needed.
pub fn crc32c(data: &[u8]) -> u32 {
    const POLY: u32 = 0x82F6_3B78;
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for i in 0..256 {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { POLY ^ (c >> 1) } else { c >> 1 };
            }
            t[i] = c;
        }
        t
    });
    let mut crc = !0u32;
    for &byte in data {
        crc = table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32c_known_vector() {
        // Standard check value for CRC32C("123456789") = 0xE3069283.
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn header_roundtrip() {
        let h = Header {
            version: 1,
            total_size: 123456,
            manifest_off: 0x100,
            manifest_len: 500,
            keyenv_off: 0x100 + 500,
            keyenv_len: 200,
            shardidx_off: 0x100 + 700,
            shardidx_len: 300,
            sig_off: 0,
            sig_len: 0,
        };
        let enc = h.encode();
        let dec = Header::decode(&enc).unwrap();
        assert_eq!(h, dec);
    }

    #[test]
    fn header_detects_tamper() {
        let h = Header {
            version: 1,
            total_size: 100,
            manifest_off: 0x100,
            manifest_len: 10,
            keyenv_off: 0,
            keyenv_len: 0,
            shardidx_off: 0,
            shardidx_len: 0,
            sig_off: 0,
            sig_len: 0,
        };
        let mut enc = h.encode();
        enc[5] ^= 0xFF; // inside magic
        assert!(Header::decode(&enc).is_err());
        let mut enc2 = h.encode();
        enc2[0x12] ^= 0x01; // inside total_size (CRC-covered)
        assert!(Header::decode(&enc2).is_err());
        let mut enc3 = h.encode();
        enc3[0x18] ^= 0x01; // manifest_off — NOT CRC-covered (by design)
        assert!(Header::decode(&enc3).is_ok());
    }
}
