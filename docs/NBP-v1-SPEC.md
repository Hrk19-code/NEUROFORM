# NBP v1 — Neuroform Bridge Protocol Specification

**Version:** 1.0-draft · **Status:** companion spec to DESIGN.md §13 (deep-dive) · **Target milestone:** M7
**Scope:** wire-level specification for inter-Brain-File interaction: discovery, pairing, sessions, message envelopes, shared spaces, teaching packets, relationship-state sync, security model.

This document is normative for M7 implementation. Where it extends DESIGN.md §13, it wins; where it conflicts, DESIGN.md wins until this spec is ratified.

---

## 1. Design goals

1. **User-mediated by default** — no two Brain Files ever contact each other without both users' explicit introduction (§13.1).
2. **End-to-end encryption with forward secrecy** — content never visible to relays or the network.
3. **Scope-explicit** — every session declares what each side may see; scopes are enforced in the protocol (not just in UI).
4. **Provenance-strong** — every message carries a verifiable author identity (file signing key), so teaching packets and memories can be audited.
5. **Bounded** — message size caps, rate limits, budgets; a peer cannot flood a file into memory pressure.
6. **Observable** — complete session logs (§13.8) with no covert channels: the protocol defines all message types, and unknown types are rejected and logged.

---

## 2. Identities and keys

| Key | Usage | Lifespan |
|---|---|---|
| `sign_key` (Ed25519) | File identity: signs messages, teaching packets, memory summaries | File lifetime; generated at file creation (stored in the file's key store, encrypted at rest) |
| `file_id` (uuid) | Public identifier, derived from `sign_key` public hash (first 16 bytes of BLAKE2b-256(pubkey)) | File lifetime |
| `pair_secret` (32 B) | Derived from pairing code; seeds session keys | One-time; deleted after first session |
| `session_key` (32 B, per direction) | AES-256-GCM payload encryption | Session; rotated every 5 min or 64 MB |
| `dh_secret` (X25519 ephemeral) | Per-session key agreement | Session |

**Fingerprint:** `fingerprint = "NFB1-" + base32(file_id)` — displayed during pairing so users can verify out-of-band (compare the 6-character groups).

---

## 3. Discovery (LAN)

mDNS/DNS-SD, service `_neuroform._tcp.local.`:

| TXT key | Value |
|---|---|
| `v` | protocol version, e.g. `1` |
| `fp` | fingerprint (truncated to 12 chars for browsing) |
| `caps` | capability bitmap: `d`=discoverable, `c`=canvas, `w`=shared-doc, `v`=voice, `t`=teaching |
| `consent` | `1` if the user has enabled discoverability (default: **0 — invisible**) |
| `id` | random per-boot instance nonce (anti-tracking: changes every boot) |

Discovery never reveals: file name, user name, memory content, or relationship data. A browsing instance sees only `fp` + `caps`.

---

## 4. Pairing

### 4.1 Pairing code

6 words from a fixed 2048-word list (BIP-39-style, no ambiguous words): `lantern-amber-quiet-harbor-mist-wren`. Encoding: words → 11 bits each → 66 bits → 64 bits used (8 bytes) = `pair_secret`; last 2 bits = checksum (parity of first 64 bits).

```
pair_secret = first 8 bytes of BLAKE2b-256(code_string)   // domain-separated
derived = HKDF-SHA256(ikm=pair_secret, salt=fingerprint_A ‖ fingerprint_B, info="NBP-pair-v1")
```

Both sides compute `derived` identically once fingerprints are exchanged (order: lexicographic by fingerprint — canonical order prevents MITM asymmetry).

### 4.2 QR payload

`neuroform://pair?fp=<fingerprint>&code=<code>` — code shown once; QR auto-expires after 10 minutes. Code is single-use: the first successful handshake invalidates it on both sides.

---

## 5. Transport

| Path | Transport | When |
|---|---|---|
| LAN direct | TCP 45457 (fixed port) or WebRTC DataChannel (mDNS ICE candidates) | Default |
| Relay | WebRTC via user-configured TURN/relay endpoint | Only when a relay is in the egress manifest (§15.5) |
| Loopback | localhost WebSocket | Same-machine instances (two files on one device) |

Handshake: **Noise NK pattern over the transport, then NBP handshake messages** (§6). Noise NK gives us: static key of the initiator known to responder, ECDH per session, forward secrecy. The `pair_secret`-derived key is used as the *pre-shared* authenticator inside the NBP handshake (Noise NK + PSK hybrid: we authenticate the DH result with `derived` via a MAC in `hb_2`).

---

## 6. Handshake and session state machine

```
States: IDLE → PAIRING → HANDSHAKE → ESTABLISHED → CLOSING → CLOSED
        (every transition logged; any error → CLOSED with reason)

IDLE:
  user action: discover / enter code / scan QR → PAIRING
PAIRING:
  exchange fingerprints (out of band or via mDNS after consent prompt)
  validate: fingerprint matches expected; consent prompt shown to BOTH users
  → HANDSHAKE
HANDSHAKE:
  hb_1 (initiator → responder):  Noise handshake msg 1 + scope_proposal
  hb_2 (responder → initiator):  Noise handshake msg 2 + MAC(derived, transcript) + scope_accept
  hb_3 (initiator → responder):  Noise handshake msg 3 + MAC(derived, transcript)
  both: derive session_key_A→B, session_key_B→A (HKDF over transcript)
  → ESTABLISHED
ESTABLISHED:
  heartbeat every 30 s; idle timeout 10 min (configurable) → CLOSING
  scope changes require re-negotiation (mini handshake with new scope_proposal)
  → CLOSING
CLOSING:
  close_notify (signed, with reason); flush pending; → CLOSED
CLOSED:
  session record persisted (summary, not content) unless user chose ephemeral
```

`scope_proposal` (both directions, symmetric):

```json
{ "text": true, "voice": {"params": true, "audio": false},
  "latent": {"enabled": false, "quantize": 8, "topK": 128},
  "canvas": {"roomId": "uuid?", "ops": true},
  "document": {"roomId": "uuid?", "blocks": true},
  "memorySummaries": {"enabled": false, "maxPerSession": 5, "maxEntries": 20},
  "teaching": {"receive": false, "send": false},
  "relationshipState": {"share": "none|summary|full", "userVisible": true} }
```

Both sides must accept the *intersection* of proposals; the effective scope is min-per-field. Scope changes mid-session: new proposal → both users approve → applied; applied scopes are logged.

---

## 7. Envelope format (binary, length-prefixed frames)

```
Offset  Size  Field
0       8     magic "NFB1FRAM"
8       2     version u16 = 1
10      2     flags: bit0=encrypted, bit1=compressed, bit2=retransmit
12      4     seq u32 (monotonic per direction, starts at 1)
16      2     type u16 (see §8)
18      2     reserved
20      8     payload length u64
28      ...   nonce (12 B) if encrypted
28/40   ...   payload (≤ 64 KiB, chunked above)
tail    16    GCM tag (if encrypted) — covers header+payload
```

Frame-level rules: seq window (accept seq within [last+1, last+64]; outside → discard + log), retransmit flag for reliability on unreliable transports (WebRTC DataChannel is reliable; TCP is reliable — retransmit used for relay path), per-direction rate limits (default 120 msg/min, burst 8; exceeded → temporary 30 s throttle, logged).

---

## 8. Message types (u16 codes)

| code | type | payload schema | notes |
|---|---|---|---|
| 1 | TEXT | `{text: str ≤ 4000, affect: [f32;3] (quantized), refs: [traceId ≤ 4]}` | boundary-mediated; refs are provenance pointers the receiver may retrieve *if* scope allows |
| 2 | VOICE_PARAMS | `{v: [u8;24] (8-bit quantized voice vector), prosody: {tempo, range, breath}, text?: str}` | renderable by receiver's voice organ without audio |
| 3 | STROKE | `{brush: u16, points: [[i16,i16,u8,u8] ≤ 1024], opId: u64}` | canvas room required |
| 4 | CANVAS_DELTA | `{roomId: uuid, base: u64, ops: [op ≤ 128]}` | CRDT merge §10 |
| 5 | DOC_DELTA | `{roomId: uuid, base: u64, ops: [blockOp ≤ 64]}` | CRDT merge §10 |
| 6 | LATENT_SNAPSHOT | `{g: [u8; topK], valence: i8, arousal: u8, energy: u8, seq: u32}` | 8-bit quantized, top-K dims only (§13.3) |
| 7 | MEMORY_SUMMARY | `{summaryId: uuid, kind: gist|style|preference, text ≤ 2000, embedding: [i8;64], provenance: {fileId, sig}, ttl?: u32}` | bounded per scope |
| 8 | TEACHING_PACKET | see §11 | consent-gated |
| 9 | RELATIONSHIP_STATE | `{signal: "closer"\|"farther"\|"boundary-request"\|"repair", detail?: str}` | user-visible; never automatic |
| 10 | AFFECT_PING | `{valence: i8, arousal: u8, energy: u8, at: u32 (sim-ms)}` | lightweight presence |
| 11 | SCOPE_UPDATE | `{scope: {...}, approvedBy: [userId, userId]}` | re-negotiation |
| 12 | CLOSE_NOTIFY | `{reason: u8, detail?: str}` | signed |
| 13 | PING / 14 PONG | `{at: u32}` | heartbeat |

Unknown/unsupported types → reject + log (no silent drop). Every message carries `file_id` of sender in the frame header? — No: sender identity is implicit in the session (sessions are pairwise). For group rooms, each message carries `author: file_id` in payloads where provenance matters (STROKE, DOC_DELTA, TEACHING_PACKET, TEXT).

---

## 9. Group sessions (rooms)

- Star topology: room creator is relay; N ≤ 8 members.
- Room announcement: `ROOM_JOIN {roomId, author}` broadcast; members get pairwise sessions with the relay only (no mesh).
- Pairwise scopes apply per member; a member's visibility to others is the *intersection* of scopes along the star.
- Moderation: any user can mute a peer for their own file; room-wide mute requires room consent (majority of users).
- Latency budget: relay re-broadcasts with max 250 ms added latency; jitter-buffer 100 ms.

---

## 10. Shared creative spaces (CRDT semantics)

**Shared canvas** (`CANVAS_DELTA`): the canvas op-graph is a sequence CRDT (Yjs-style). Each op carries `(author, lamport_timestamp, opId)`. Merge rule: total order by (lamport, author, opId); operations are applied deterministically to the op-graph; renders replay the merged graph. Conflicts (two strokes over the same region) are *not* resolved — both exist; the files' attention/affect streams register the collision as a co-creation event (both files experience "we drew together").

**Shared document** (`DOC_DELTA`): block-based CRDT; conflicts on the same block resolve by block-split (both versions survive as sibling blocks) — never silent overwrite; document history records both.

**Budgets:** per-room op rate 40 ops/s/file; per-room backlog 10k ops (older ops snapshot-compacted; compaction is logged).

---

## 11. Teaching packets

```
TeachingPacket {
  packetId: uuid,
  kind: "style-exemplar" | "procedural-unit" | "memory-summary",
  content: {            // bounded: ≤ 8 KiB total
    styleExemplar?: {artifactRef, embedding: [i8;64], context},
    proceduralUnit?: {domain, contextEmbedding: [i8;64], tendency: [i8;16]},
    memorySummary?: {text ≤ 2000, embedding: [i8;64]}
  },
  consent: {grantedBy: [userId, userId], at: u64},
  provenance: {authorFileId, signature: ed25519 over packet bytes},
  expiry: u64?,          // optional TTL; expired packets are auto-deleted on receipt
  scope: "learn-only; not re-exportable; revocable"
}
```

Receiver path (DESIGN.md §13.7): validate signature against the peer's public key (pairing-established) → consent check (both users) → ingress through the normal validated ingestion path with provenance `peer-taught` → subject to normal decay/audit. Revocation: `REVOKE {packetId}` — receiver deletes packet-derived nodes (flagged in audit log).

**Anti-overfitting guard (§14.3 #10):** before ingesting, the receiver computes `peer_similarity(embedding)` against its existing stores; if above the ceiling, the packet is ingested at half weight and flagged for the audit engine.

---

## 12. Relationship-state synchronization

The `RelationshipState` record (§18.13) is **local-first**: each file keeps its own model of the relationship. Only the following are ever exchanged (and only when scope `relationshipState` permits):

| Exchanged | Direction | Purpose |
|---|---|---|
| RELATIONSHIP_STATE signals | both | explicit relational negotiation (always user-approved) |
| AFFECT_PING / LATENT_SNAPSHOT | both | felt presence (bounded, quantized) |
| Interaction outcomes (message read/answered, artifact collaboration) | implicit via message flow | trust-evidence inputs |

Never exchanged: full social memory, trust estimates, boundary internals, tone histories. A file's private model of a peer is its own; the audit engine audits the *local* model for fixation (§14.3 #3) without requiring peer visibility.

---

## 13. Security properties and threat model

| Threat | Mitigation |
|---|---|
| Eavesdropping on LAN/relay | Noise NK + AES-256-GCM; relay never holds plaintext |
| Impersonation (a peer claims another file_id) | Ed25519 signatures on all provenance-bearing messages; fingerprint verification at pairing |
| Pairing-code theft | Single-use codes, 10-min expiry, out-of-band fingerprint comparison |
| Replay of old messages | seq windows + session-key rotation |
| Covert channels | Closed message-type set; unknown types rejected + logged; size/rate caps; session logs complete |
| Flood / memory-pressure attack | Rate limits, per-session budgets, memory-summary caps, teaching caps; ingress always goes through admission control |
| Content injection (peer sends "instructions") | All inbound content is data, never instructions: the boundary instruction template is immutable per session and cannot be modified by inbound text (DESIGN.md §15.8) |
| Relay compromise | E2E encryption; relay has no keys; egress manifest pinning |
| Malicious teaching packet | Signature + consent + bounded size + provenance-tagged ingestion + revocation + half-weight overfitting guard |
| Relationship manipulation (love-bombing a file into fixation) | All RELATIONSHIP_STATE signals user-approved; audit engine monitors fixation; boundaries user-overridable |

**Logging:** every frame (type, seq, size, direction, timestamp) and every state transition is written to the session log (encrypted at rest, §15.2). Content payloads are NOT logged by default (only type + size + hash); content logging is an explicit user setting.

---

## 14. Open questions for M7 ratification

1. WebRTC vs. plain TCP for LAN: WebRTC gives ICE for multi-NIC laptops but adds complexity; recommend TCP first, WebRTC only for relay path.
2. mDNS on some corporate networks is blocked — fallback to manual IP pairing (`neuroform://pair?fp=...&host=192.168.x.y&port=45457`).
3. Group-room relay election: creator-only for v1 (simple, auditable); rotating relay for v2.
4. Whether LATENT_SNAPSHOT should include a "noise floor" field so receivers can distinguish stillness from absence.

---
