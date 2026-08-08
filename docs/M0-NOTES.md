# M0 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M0 (Skeleton) — per DESIGN.md §20.

## Scope delivered

- Repository scaffold per DESIGN.md §23 (workspace layout: `packages/brain-core`, `apps/cli`, `tools/`, `docs/`).
- **NF1 `.brain` format** (`packages/brain-core/src/format/`):
  - 0x100-byte header, magic `NF1BRAIN`, version u32, header CRC32C over `[0x0000, 0x0014)`.
  - Key envelope: Argon2id (m=64 MiB, t=3, p=4) wraps a random DEK; XChaCha20-Poly1305 per shard, per-shard 24-byte nonce.
  - Shard index entries: id, type, offset, length, compression, BLAKE2b-256 checksum over stored (ciphertext) bytes, encrypted flag, nonce, schema version.
  - Atomic writes: temp file + `sync_all` + rename.
  - Sections ordered header | key envelope | manifest | shards | shard index (index last ⇒ offsets are length-stable).
- **Tick loop** (`brain.rs`): 10 Hz, damped baseline-attractor integration of the 26 named state channels + reserved vector (dim per tier), 8 modulator axes, seeded SplitMix64 RNG.
- **Capacity ledger** (`capacity.rs`): 4 tiers with spec table values, per-shard accounting, admission flags (Ok/Flag/Critical).
- **CLI** (`apps/cli`): `create`, `verify`, `tick`, `inspect` (+ `--json`, `--out`, `--snapshot`).
- **Python validator** (`tools/validator/validate_nf1.py`): independent implementation — header/CRC/manifest/bounds/checksums.
- **Cortex Canvas scaffold** (`packages/cortex-canvas/`): static-snapshot renderer; region→metric bindings verified against `Brain::snapshot_json()` output.
- **Tests** (in-crate): RNG determinism, state bounds/determinism, modulator determinism, ledger accounting, header round-trip + tamper, passphrase round-trip, plain-dev round-trip, wrong-passphrase rejection, corrupt-shard detection, verify report, 1M-tick determinism, save/load continuity.

## Deviations from DESIGN.md §16 (deliberate, M0-scoped)

| Spec | M0 implementation | Why / when fixed |
|---|---|---|
| Magic `"NF1BRAIN\x01"` (9 bytes in spec table) | 8-byte magic `"NF1BRAIN"`; version in u32 field | The spec table's string is 9 bytes against an 8-byte field; version field carries the `1`. Format table in §16.1 otherwise unchanged. |
| BLAKE3 checksums | BLAKE2b-256 | Zero-dependency + stdlib-verifiable in the Python validator (`hashlib.blake2b`); field format identical (hex string). Swap to BLAKE3 in M1 if desired. |
| Compression `none\|zstd` | `"none"` only | zstd deferred; field reserved. |
| Ed25519 signature tail | Absent (sig offsets zeroed) | Signing arrives with inter-brain provenance (M7). |
| OS keychain key slot | Not implemented; `plain-dev` mode stores DEK hex in envelope when no passphrase | M1. `plain-dev` files are explicitly flagged on load/verify. |
| Capacity admission → pruning | Accounting + flags only; STATE/MODULATORS get budget slices (1/10, 1/50 of file cap) | Slot budgets and write-time enforcement arrive with memory stores (M1). |
| Cortex Canvas live 10 Hz binding | Static snapshot | M1 (state channel of the cognitive bus). |

## Exit criteria status (M0)

- [x] Round-trip file save/load with checksums — tests `full_roundtrip_with_passphrase`, `plain_dev_roundtrip`.
- [x] Deterministic replay of 1M ticks — test `million_ticks_are_deterministic`.
- [x] Format corruption tests — `corruption_detected`, `verify_reports_corrupt_shard`, header tamper tests.
- [ ] Cortex Canvas scaffold — delivered as static snapshot; live binding is M1 (see deviations).

## Verified runs (real output, 2026-08-04)

```
$ cargo test --release
test result: ok. 16 passed; 0 failed  (incl. million_ticks_are_deterministic,
  save_load_preserves_continuity, full_roundtrip_with_passphrase,
  wrong_passphrase_fails, corruption_detected, verify_reports_corrupt_shard)

$ neuroform create demo.brain --tier standard --seed 42
created demo.brain (tier standard, seed 42, 3904 bytes, key mode: plain-dev)
  brain id: 7e3c6d97-...  digest: f3e14a30367a7e33

$ neuroform tick demo.brain --ticks 1000000 --save
ticked 1000000 ticks in 0.451s (2219474 ticks/s); sim_time now 1000000

$ neuroform verify demo.brain
verify PASS — shard STATE: ok (checksum ok); shard MODULATORS: ok (checksum ok)

$ python tools/validator/validate_nf1.py demo.brain
PASS: header CRC32C ok, manifest ok, 2 shards, checksums ok

$ neuroform create secret.brain --passphrase "test-pass"   # argon2id 64MiB/3/4
$ neuroform verify secret.brain --passphrase "wrong"
error: wrong passphrase or corrupted key envelope   ← correctly rejected

$ python tools/make_preview.py packages/cortex-canvas   # → preview.html (renders snapshot)
```

Note: 1M-tick digest `a3e21097be6c1daa` (pre-fix binary) vs `6f1aee9fc9f668b9` (post-fix)
differ because the RNG-state persistence changed the post-save stream — the
*in-memory* determinism (same seed → same digest) is what the tests assert, and
they pass: `million_ticks_are_deterministic` (1M ticks, two fresh runs, equal digests)
and `save_load_preserves_continuity` (save/load mid-life, continued ticks match a
never-saved twin).

## Toolchain note (Windows)

This machine had no working C linker. Setup that made the build work:
`rustup` (MSVC toolchain) + mingw-w64 gcc at `<tools-dir>\winlibs-mingw64`
(space-free path — gcc's driver breaks on `<user-home>\...`), GNU Rust
toolchain via `rustup toolchain install stable-x86_64-pc-windows-gnu`, override
set in the repo. PATH lines appended to `~/.bashrc`.

## Next (M1 — Core life)

Event inbox + SensoryEvent envelope; episodic binder + salience; retrieval; LLM boundary (UtterancePacket, attach/detach); OS keychain slot; zstd; snapshot auto-save every 3000 ticks; live Cortex Canvas binding; bias-audit skeleton.
