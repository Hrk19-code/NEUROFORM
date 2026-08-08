# FINDINGS — independent review + landscape scan (2026-08-08)

An outside agent (Hermes, kimi-k3) audited this repository cold on
2026-08-08 — no prior context, source rebuilt from scratch, claims checked
against live execution. This file records methods, counts, and honest
boundaries. Nothing here is self-reported by the project's authors.

## 1. Engine audit (cold, from source)

| Check | Method | Result |
|---|---|---|
| Unit suite | `cargo test --workspace` (fresh build) | **117 passed, 0 failed** |
| Behavioral suite | `tools/verify/verify-current.sh` (fresh release build + CLI scenarios) | **17 passed, 0 failed** |
| Determinism | `million_ticks_are_deterministic`, `life_30_days_is_deterministic` unit tests | PASS |
| Encryption | `full_roundtrip_with_passphrase`, `wrong_passphrase_fails` (Argon2id 64MiB/t3/p4 + XChaCha20-Poly1305) | PASS |
| Retrieval discriminates | Live probe: two events ("orange cat…" / "love drawing…"), then ranked retrieval | Query "cat on mat" → cat trace 0.576 vs 0.013; query "drawing ink" → 0.746 vs 0.024. Rankings flip correctly |
| Caution substrate | `tools/verify/verify-caution.sh` (seed 5, deterministic) | **4/4** — aversive trace binds at salience 0.876 (slow-decay tier), scores 0.074 vs a neutral query (specific, not generalized), repetitions habituate (0.876→0.552) |
| Adapter | `tools/adapter/verify_adapter.py` (live CLI, copy brains hash-pinned) | **13/13** — auto/manual attach, encoder gate both ways, aggregation math, twin determinism (bit-identical digests), failure honesty, car morphology, virtual vehicle |
| Virtual ride demo | fresh brain + `sim_car`, 2s session | motion percepts recorded (lin 4.84→5.91 m/s); vestibular cortex activation 0.69 |

## 2. "Is there an equivalent?" — GitHub landscape scan

14 targeted queries via the GitHub search API (2026-08-08), top hits by
stars. Caveats: GitHub ≠ all software; English-language queries; stars
measure adoption, not correctness.

- `"brain file" ai organism` → **0 repositories.**
- `neuromodulation agent llm` → **0 repositories.**
- Closest neighbors: `letta-ai/agent-file` (1,193★ — serializes LLM-agent
  memory blocks; no organs/drives/sleep/determinism/encryption);
  `openclaw-auto-dream` (548★ — batch memory consolidation, not a staged
  physiological sleep cycle); active-inference research code (~110 repos,
  top ~102★ — simulations, not ownable organisms); generative-agents
  clones (LLM-call-per-cognition); desktop-pet toys (0–28★).
- Commercial shallow analogs: NIO's NOMI (dashboard companion), Sony Aibo
  (persistent pet identity) — both bond with users; neither has
  autobiographical memory, homeostatic drives, or deterministic replay.

**Conclusion the data supports:** the *combination* — a portable,
encrypted, deterministic organism file with neuromodulator axes,
salience-decaying memory, staged sleep, organs, a teacher boundary,
chemical reproduction, and a universal body adapter — is unoccupied on
GitHub at any star count. Every individual mechanism has established
scientific lineage (ACT-R activation/decay, allostatic load, somatic
markers, homeostasis); the packaging is what is new.

## 3. Honest boundaries (what the review does NOT license)

- The system is **not sentient** and claims no experience; "feelings" are
  simulated states that modulate processing (standing notice, DESIGN.md
  §15.6).
- Text fluency comes from an **attached LLM teacher** through a metered,
  audited boundary; without one the file runs an honest degraded mode
  rather than fabricating.
- "No equivalent found" is a claim about public repositories on one day,
  not about all software everywhere.
- The §28–§38 body series is **design, not build** — withheld from this
  public copy until milestones pass their pre-registered bars.
