# M7 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M7 (Network / inter-brain interaction) — per DESIGN.md §13, docs/NBP-v1-SPEC.md, §20 and master-prompt Tab 6.

## Scope delivered (headless core; wire transport is the shell milestone)

- **Relationship store** (§12, §18.13, local-first): per-peer familiarity / trust / tone / boundary tightness / message counters / shared artifacts / bounded history. **Non-permanence by construction**: familiarity and trust decay without interaction (verified by test — 50k idle ticks erode a relationship that rose through 10 exchanges); boundary tightness drifts toward baseline. The file's private model of a peer is its own — never shared wholesale.
- **Session state machine** (§6): IDLE → PAIRING → HANDSHAKE → ESTABLISHED → CLOSING → CLOSED, every transition logged with reason; idle sessions swept to CLOSED after 10 sim-minutes; closed records persist (decayable).
- **Pairing with key exchange** (§4): `pair_with_key` — the peer's file key is exchanged out-of-band like NBP fingerprints. Inbound MACs verify against the *peer's* key; **no key exchanged → inbound rejected** (secure default, tested). Deterministic per-file signing key derived from the file seed.
- **Closed message-type set** (§8, 14 types): unknown types rejected + logged; scope-gated send/receive (min-per-field intersection of proposals, tested); seq window [last+1, last+64] enforced; rate limit 120 msg/min enforced; tampered payloads rejected (MAC, tested); author binding on every frame.
- **Provenance**: keyed-BLAKE2b MACs over (author|type|seq|payload) — constant-time comparison. Spec's Ed25519 upgrade is the wire milestone; the audit property (tamper-evident author binding) holds today.
- **Teaching packets** (§11): MAC + author-match + expiry validation, anti-overfitting note; rejected on any tamper (tested). Consent-gated by scope (`teaching` flag).
- **CRDT merge** (§10): deterministic total order (lamport, author, opId) — commutative, lossless, conflicts survive (never silent overwrite) — tested with real tie-break data.
- **Social effects** (brain level): inbound TEXT/AFFECT_PING bind as **social-stream percepts** with peer-id keywords and affect nudges from the peer's quantized affect — the peer's felt presence enters memory, retrieval, dreams, and decay like any other stream. Relationship tone/familiarity update on exchange.
- **Format**: 11th shard `NET`; serde-defaulted (pre-M7 files load a fresh organ); validator extended.
- **CLI**: `net status|key|pair --peer --peer-key|establish --session|send --text|inject --text|signal --closer/--farther/--repair|close|discover --on`; `inspect`/snapshot network line (discoverable, relationships, sessions, message totals). Discoverability defaults **OFF** (invisible, §3).
- **Tests**: 94 total (was 85) — 9 new: session lifecycle + logging, scope intersection, MAC/seq/rate validation + no-key rejection, relationship decay, idle sweep, teaching provenance (tamper/author/expiry), CRDT determinism, discoverability/digest, persistence round-trip.

## Bugs caught (all fixed)

1. **Familiarity never rose at organ level** (design gap, caught by my own test): `send`/`receive` counted messages but only the brain layer bumped familiarity — a headless relationship would never form. Organ `receive` now nudges familiarity (matches brain level).
2. **Test fixture errors** (mine): CRDT tiebreak test had no actual lamport tie (a's ops were at 1 and 3, b's at 2 — no author tie to break); decay test asserted familiarity decay from 0.0. Fixed fixtures + added the organ-level bump above.
3. **Suite L-step grep case** (script): status prints `{:?}` Debug (`Established`), suite grepped `ESTABLISHED`. Fixed.

## Exit criteria status (M7, headless-core reading)

- [x] Full session lifecycle with consent (pair → establish → exchange → close, logged)
- [x] Message types (closed set, 14 types) + envelopes with seq/rate/scope enforcement
- [x] Shared-space CRDT merge semantics (deterministic total order, conflict-preserving)
- [x] Relationship decays when idle (verified)
- [x] Teaching-packet provenance verifies (MAC + author + expiry; tamper rejected)
- [x] Social memory: peer messages bind as retrievable percepts; relationship model auditable
- [x] NET shard + validator + serde-default migration
- [ ] LAN wire transport (TCP 45457 / mDNS discovery / Noise NK + Ed25519 / WebRTC relay) — desktop-shell milestone
- [ ] Group rooms (star topology, N ≤ 8), shared canvas/doc rooms with live merge — shell milestone
- [ ] Latent-state exchange (LATENT_SNAPSHOT, quantized top-K) — shell milestone

## Two-brain demo (verified end-to-end via CLI)

Brain A (seed 11) and Brain B (seed 22) paired with key exchange, both established (text scope), A sent "the garden is full of tomatoes this year", B received it through the validated path and bound `ep #1 src=peer [peer brain-a garden full tomatoes this year]` — retrievable via `retrieve --query "tomatoes"`. B replied, A received. B's relationship with A: familiarity 0.04, trust 0.30, tone +0.01, boundary 0.40, msgs 1→1. The conversation is normal memory — decayable, dreamable, auditable.
