#!/usr/bin/env python3
"""NF1 .brain format conformance validator (independent implementation).

Verifies, without any Rust code:
  - header magic / version / declared size
  - header CRC32C over bytes [0x0000, 0x0014)
  - manifest JSON parses and reports the expected fields
  - shard index entries are in-bounds and their BLAKE2b-256 checksums match the
    stored bytes (checksums cover ciphertext; this validator does not decrypt)

Usage:  python validate_nf1.py <file.brain>
Exit:   0 = PASS, 1 = FAIL, 2 = usage error
"""

import hashlib
import json
import struct
import sys

MAGIC = b"NF1BRAIN"
HEADER_LEN = 0x100
CRC_COVER_END = 0x14
FORMAT_VERSION = 1

# CRC32C (Castagnoli) table, reflected, poly 0x82F63B78
_CRC_TABLE = []
for i in range(256):
    c = i
    for _ in range(8):
        c = (0x82F63B78 ^ (c >> 1)) if (c & 1) else (c >> 1)
    _CRC_TABLE.append(c)


def crc32c(data: bytes) -> int:
    crc = 0xFFFFFFFF
    for b in data:
        crc = _CRC_TABLE[(crc ^ b) & 0xFF] ^ (crc >> 8)
    return crc ^ 0xFFFFFFFF


def blake2b_256(data: bytes) -> str:
    return hashlib.blake2b(data, digest_size=32).hexdigest()


def parse_header(b: bytes):
    if len(b) < HEADER_LEN:
        raise ValueError(f"file too short for header ({len(b)} < {HEADER_LEN})")
    magic = b[0:8]
    if magic != MAGIC:
        raise ValueError(f"bad magic: {magic!r} (expected {MAGIC!r})")
    version = struct.unpack_from("<I", b, 8)[0]
    if version != FORMAT_VERSION:
        raise ValueError(f"unsupported version {version} (expected {FORMAT_VERSION})")
    total_size = struct.unpack_from("<Q", b, 12)[0]
    stored_crc = struct.unpack_from("<I", b, 20)[0]
    actual_crc = crc32c(b[0:CRC_COVER_END])
    if stored_crc != actual_crc:
        raise ValueError(f"header CRC32C mismatch: stored {stored_crc:08x}, actual {actual_crc:08x}")
    (manifest_off, manifest_len) = struct.unpack_from("<QQ", b, 24)
    (keyenv_off, keyenv_len) = struct.unpack_from("<QQ", b, 40)
    (shardidx_off, shardidx_len) = struct.unpack_from("<QQ", b, 56)
    (sig_off, sig_len) = struct.unpack_from("<QQ", b, 72)
    return {
        "version": version,
        "total_size": total_size,
        "manifest_off": manifest_off,
        "manifest_len": manifest_len,
        "keyenv_off": keyenv_off,
        "keyenv_len": keyenv_len,
        "shardidx_off": shardidx_off,
        "shardidx_len": shardidx_len,
        "sig_off": sig_off,
        "sig_len": sig_len,
    }


def section(data: bytes, off: int, length: int, name: str) -> bytes:
    if off < HEADER_LEN or off + length > len(data):
        raise ValueError(f"{name} section out of bounds (off={off}, len={length}, file={len(data)})")
    return data[off : off + length]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: python validate_nf1.py <file.brain>")
        return 2
    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()

    checks = []
    try:
        h = parse_header(data)
        checks.append(("header", True, f"magic ok, version {h['version']}, size {h['total_size']}"))
        if h["total_size"] != len(data):
            raise ValueError(f"declared size {h['total_size']} != actual {len(data)}")

        env_bytes = section(data, h["keyenv_off"], h["keyenv_len"], "key envelope")
        env = json.loads(env_bytes)
        mode = env.get("mode")
        if mode not in ("passphrase", "plain-dev"):
            raise ValueError(f"unknown envelope mode {mode!r}")
        checks.append(("key envelope", True, f"mode={mode}"))

        man_bytes = section(data, h["manifest_off"], h["manifest_len"], "manifest")
        man = json.loads(man_bytes)
        for field in ("format", "version", "brain_id", "created_at", "seed", "rng_state",
                      "event_counter", "dropped_events", "sleep_pressure", "sleep_emotional_load",
                      "autonomy_enabled", "autonomy_quiet_start", "autonomy_quiet_end",
                      "capacity_tier", "migration_chain", "raw_vault_ref", "capacity"):
            if field not in man:
                raise ValueError(f"manifest missing field {field!r}")
        if man["format"] != "neuroform":
            raise ValueError(f"manifest format {man['format']!r}")
        cap = man["capacity"]
        for field in ("tier", "total_bytes", "total_budget", "shards"):
            if field not in cap:
                raise ValueError(f"capacity ledger missing field {field!r}")
        checks.append(("manifest", True,
                       f"brain {man['brain_id'][:8]}… tier {man['capacity_tier']} seed {man['seed']}"))

        idx_bytes = section(data, h["shardidx_off"], h["shardidx_len"], "shard index")
        entries = json.loads(idx_bytes)
        if not isinstance(entries, list) or not entries:
            raise ValueError("shard index empty or not a list")
        checks.append(("shard index", True, f"{len(entries)} shard(s)"))

        if h["sig_len"] != 0:
            checks.append(("signature", True, "present (M0 writer leaves sig zeroed)"))

        for e in entries:
            eid = e.get("id", "?")
            sid = e.get("id", "?")
            off, length = e["offset"], e["length"]
            stored = section(data, off, length, f"shard {eid}")
            if blake2b_256(stored) != e["checksum"]:
                raise ValueError(f"shard {eid} checksum mismatch")
            if not e.get("encrypted", False) or not e.get("nonce"):
                raise ValueError(f"shard {eid} not flagged encrypted")
            if e.get("compression") != "none":
                raise ValueError(f"shard {eid} compression {e.get('compression')!r} (M0 supports none)")
            checks.append((f"shard {sid}", True, f"{length} bytes, checksum ok, {e['shard_type']}"))
    except Exception as exc:  # noqa: BLE001 — validator reports any failure
        print(f"FAIL: {path}")
        print(f"  {exc}")
        for name, ok, detail in checks:
            print(f"  [{'ok' if ok else 'FAIL'}] {name}: {detail}")
        return 1

    print(f"PASS: {path}")
    for name, _ok, detail in checks:
        print(f"  [ok] {name}: {detail}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
