# M2 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M2 (Sleep & dreams) — per DESIGN.md §20.

## Scope delivered

- **Sleep pressure** (`sleep.rs`): P ∈ [0,1] accumulator — baseline 0.05/h + memory-pressure term (capacity fullness) + emotional-load term (per-event |valence| registered in `process_event`) + interoception terms (low energy / high fatigue push pressure up). Trigger detection: pressure ≥ 0.8, memory-critical (fullness ≥ 0.95), emotional-load-high. Circadian schedule flag reserved (default off).
- **Four stages per cycle** (wind-down 5 min → light 20 min → deep 40 min → dream 30 min; full cycle 2 h sim = 72,000 ticks):
  - **Wind-down**: pending events flushed to traces; arousal/alertness glide down.
  - **Light**: salience-weighted replay (budget max(8, n/20) capped 200) — replayed traces strengthen ×1.03, reconsolidation_count++, marked replayed; 30% chance of a **recolored drift copy** (blend toward nearest semantic node + seeded jitter, weaker 0.6× strength, 0.7× salience) — memory changes over time, as real memory does (§10.3).
  - **Deep**: downscaling (strength ×0.97, salience ×0.99), pruning below the retention floor (score = salience×strength < 0.02), **gist clustering** (streaming k-means over trace embeddings; clusters ≥ 3 members with cohesion ≥ 0.6 distilled into semantic nodes with provenance `gist`, linked source episodes, episodes marked gist-extracted), emotional regulation (valence rebalanced toward baseline, arousal down, fatigue down, energy up, stress load ×0.8, saturation ×0.5).
  - **Dream**: associative walk over the semantic graph from residue (top-3 traces + a random node), similarity-weighted with temperature 0.55; fragments in modalities text/visual-motif/emotion/body, each **provenance-linked** (`node:<id>` / `interoception`) with per-fragment bizarreness (1 − jump cosine); **no external actions by construction** (the dream stage reads stores and writes the dream log only — verified by test: teacher untouched, zero tokens spent during sleep).
- **Sleep reports**: per-sleep stage work (replayed/recolored/pruned/gists/regulated), dream ids, modulator normalization, post-sleep audit alarms as bias actions. Reports + dream log persisted in the **DREAMS shard** (6th shard).
- **Persistence**: `sleep_pressure` + `sleep_emotional_load` added to the manifest (serde-defaulted for old files); dreams/reports restored on load; sleep included in the digest (determinism across save/load).
- **CLI**: `sleep <file> [--cycles N]`, `dreams <file> [--top N]`, `inspect` sleep line, `life --sleep-every N` (nightly sleep in the life harness).
- **Tests**: 46 total (was 40) — pressure accumulation/reset, **sleep ablation** (twin files, 15 days, sleep vs no-sleep: gist nodes ↑, pressure reset, affect variance ↓, trace growth bounded), dreams provenance + zero-external-actions, sleep determinism, dreams persist across save/load, deep-stage gist consolidation.

## Bugs caught by the test suite (all fixed)

1. **Sleep pressure not persisted** → digest mismatch after save/load (5 tests). Fixed: manifest fields `sleep_pressure`/`sleep_emotional_load`.
2. **Provenance attribution bug**: online semantic re-consolidation credited `gist` regardless of trace source, polluting the gist signal that the audit and consolidation metrics depend on. Fixed: attribution follows the trace's source; `gist` provenance is reserved for sleep-stage distillation.
3. **Ablation test expectation error** (not a code bug): replay's recolored drift copies are spec'd behavior (§10.3), so the sleeping file grows via reconsolidation; the honest assertions are bounded growth + distillation quality, not "fewer traces".

## Exit criteria status (M2)

- [x] Sleep pressure + triggers (user command, pressure ≥ 0.8, memory-critical, emotional-load-high; schedule flag reserved)
- [x] Four stages with real consolidation work (replay, downscaling, pruning, gist extraction, emotional regulation)
- [x] Dreams contain provenance-linked fragments (every fragment carries `node:<id>` / `interoception`)
- [x] Zero external actions from the dream stage (structural + test-verified: tokens untouched, teacher survives, pending flushed)
- [x] Sleep reports per cycle; dream logs persisted and inspectable
- [x] Sleep ablations show measurable consolidation effects (§22.2 directional: gist nodes ↑, affect variance ↓, pressure reset)
- [x] Post-sleep bias audit hook (`trigger: "post-sleep"` → bias actions in report)
- [ ] OS keychain key slot — still deferred (plain-dev + passphrase modes remain; scheduled with M3)
- [ ] HTTP teacher adapter — still deferred (MockTeacher implements the contract)

## Verified runs (real output, 2026-08-04)

```
$ cargo test --release
test result: ok. 46 passed; 0 failed

$ neuroform sleep m1.brain --cycles 1 --save
sleep #1 @ t=259510: 1 cycle(s) — pressure before 0.000, triggers: []
  wind-down 3000 ticks | replayed 0 recolored 0 pruned 0 gists 0 regulated false
  light     12000 ticks | replayed 8 recolored 3 pruned 0 gists 0 regulated false
  deep      24000 ticks | replayed 0 recolored 0 pruned 0 gists 6 regulated true
  dream     18000 ticks | replayed 0 recolored 0 pruned 0 gists 0 regulated false
  dreams: 4 | modulator normalized: true | bias actions: []
  pressure now 0.050

$ neuroform dreams m1.brain --top 2
dream #4 (sleep #1) bizarreness 0.28 — [text]stargazing [visual-motif]stargazing
  [emotion]emotion residue: +0.29 [body]weightless, load 0.11, saturation 0.36, fatigue 0.06

$ neuroform life m1.brain --days 6 --sleep-every 1
[day 5] sleep: replayed 8 recolored 0 pruned 0 gists 0 dreams 4
life complete: 131 traces (24 pruned), 61 nodes, 409 tokens

$ python tools/validator/validate_nf1.py m1.brain
PASS: 6 shards (STATE, MODULATORS, EPISODIC, SEMANTIC, HORMONE, DREAMS), checksums ok
```

## Notes

- The dream stage consumes the seeded RNG like everything else — sleep is fully deterministic per file (test-verified: same seed → same digest after 2 cycles).
- Recolored drift copies carry new trace ids and the `replayed` consolidation state; they decay like normal traces and are eligible for pruning — boundedness is preserved.
- Gist clustering is local-only (centroid + top keyword). LLM-assisted labeling of gists is the M3+ boundary enhancement (DESIGN.md §4.4 distillation pipeline).

## Next (M3 — Writing organ)

Document model + modes (prose/journal/worldbuilding/lorebook), version history, style analysis, continuity ledger, extraction pipeline into memory, brain-modulated assistance — plus the deferred OS keychain slot and HTTP teacher adapter.
