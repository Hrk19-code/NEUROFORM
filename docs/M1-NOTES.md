# M1 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M1 (Core life) — per DESIGN.md §20.

## Scope delivered

- **Event inbox + SensoryEvent envelope** (`events.rs`): 8 streams, bounded inbox (256) with saturation dropping, deterministic factory keyword embeddings (stable per-file keyword→vector map so retrieval and habituation work; production encoders replace it in M3+).
- **Episodic binder** (`memory.rs`): salience-weighted window binding (300 ticks / 8 events), salience = novelty·w + emotion·w + source·w + baseline, strength decay with salience-dependent half-lives (7 days base, 4× slower for salience ≥ 0.7, 4× faster for < 0.3), capacity-bounded pruning with floor, Gini metric.
- **Semantic store** (`semantic.rs`): streaming gist-lite (match cosine 0.75 → belief growth + embedding blend; else new node), belief decay to floor, provenance weights (user/llm/peer/gist), budgeted retrieval.
- **Budgeted retrieval** (`memory.rs` + `brain.rs`): cosine × strength × recency scoring, k_traces/k_nodes/token_cap budgets, node-first trimming, truncation flag, context builder.
- **LLM boundary** (`boundary.rs`): `Teacher` trait, deterministic `MockTeacher`, UtterancePacket assembly (state gloss, context, permissions), attach/detach, token accounting, degraded substrate-only output with explicit notice. HTTP teacher adapter deferred to M2 (contract stable).
- **Hormonal embodiment** (`embodiment.rs`, §4.8): 6 presets (male/female/custom/mixed/non-binary/user-defined) as probabilistic priors over 16 axes; seeded per-file sampling; bounded gains (±0.3 cap) on modulator baselines + event salience/nudge weights; zero-gain axes (5ht/ach/ecb) and zero-gain pathways for everything presets must never determine; `set_embodiment` re-sampling with auditable history; HORMONE shard persistence; neutral fallback for pre-M1 files.
- **Bias audit skeleton** (`audit.rs`, §14): 4 live metrics (memory-overvaluation Gini, repetition, emotion-loop autocorrelation, user-overfit dominance) + 6 declared stubs with milestone notes; user-gated intervention suggestions; f64 autocorrelation accumulation (f32 drift bug caught by tests).
- **Persistence**: EPISODIC/SEMANTIC/HORMONE shards; manifest gains `event_counter`/`dropped_events` (+`rng_state` from M0) so digests and event-embedding streams survive save/load; old files load with defaults.
- **CLI**: create (`--embodiment`), verify, tick, inspect, event, memory, retrieve, chat (`--teacher`), attach, detach, teachers, embodiment (`--set`), audit, **life** (30-day simulated life with teacher attach/detach windows + cohort decay curve), **watch** (JSONL state snapshots — the Cortex Canvas live-binding contract).
- **Tests**: 40 total (was 16) — incl. 30-day life harness (decay, detach-window tokens = 0, determinism across runs), embodiment non-determination, mutability/auditability/persistence.

## Bugs caught by the test suite (all fixed)

1. **RNG stream position not persisted** (M0 holdover — fixed in M0): continuity across save/load.
2. **Decay underflow**: `strength *= decay_rate.powf(dt)` applied powf to the per-tick *rate* instead of `(1 − rate)` → strengths collapsed to 0. One root cause behind two test failures.
3. **Unstable keyword embeddings**: per-event seed made the same keyword map to a different vector every event → retrieval was noise and repetition never habituated. Fixed: stable per-file keyword→vector map.
4. **Audit ring buffer read uninitialized slots** → spurious autocorrelation; and **f32 summation drift** made a constant signal autocorrelate to 1.0 → f64 accumulation + filled-count tracking.
5. **Manifest missing event counters** → digest mismatch after save/load of eventful files.

## Exit criteria status (M1)

- [x] Event inbox + SensoryEvent envelope, bounded + droppable
- [x] Episodic binder + salience + online decay (half-life curves tested)
- [x] Semantic store with gist-lite distillation (repeated concepts consolidate; distinct concepts stay distinct)
- [x] Retrieval with trace/node/token budgets (truncation flag tested)
- [x] LLM boundary: attach/detach, token accounting, degraded mode, persistence of the file across detach
- [x] Hormonal embodiment: male/female/custom/mixed/non-binary presets, probabilistic sampling, bounded auditable gains, mutation with history
- [x] 30-day simulated life with two teachers + detach window (detach-window tokens = 0; cohort decay curve; audit clean)
- [x] Memory decay curves match spec (half-life classes verified within 2%)
- [ ] OS keychain key slot — deferred to M2 (plain-dev + passphrase modes remain)
- [ ] Live Cortex Canvas binding — `watch` emits the state-snapshot stream; the Tauri shell binding is scheduled with the desktop milestone
- [ ] HTTP teacher adapter — deferred to M2 (MockTeacher implements the contract)

## Verified runs (real output, 2026-08-04)

```
$ cargo test --release
test result: ok. 40 passed; 0 failed

$ neuroform create m1.brain --tier standard --embodiment female --seed 42
created m1.brain (tier standard, embodiment female, seed 42, 7678 bytes)
  modulator deltas: da +0.054 ne -0.102 cort +0.017 oxt +0.138 avp -0.004

$ neuroform chat m1.brain "what do you remember about the garden?" --teacher amber
file: [amber] i remember something like this — about what do you remember about the garden?.
  (teacher: amber, tokens 42)

$ neuroform retrieve m1.brain --query "tomatoes in the garden" --k 3
ep #1 score=0.729 sal=0.869 str=1.000 src=user [garden full tomatoes today]
sem #1 score=0.300 belief=0.411 [garden]

$ neuroform embodiment m1.brain --set male --save   # audited: history + re-sampled priors

$ neuroform audit m1.brain
10 metrics, 1 alarm (user-overfit 1.000 ≥ 0.800 — correct on a 1-trace file)
suggestion: plasticity restoration (user-gated)

$ neuroform life m1.brain --days 30 --seed-stream 777
day 21 tokens freeze at 1327 (teacher detached) — detach-window tokens: 0
cohort decay: day1 [0.997, 0.998, 0.991] → day30 [0.923, 0.923, 0.746]
life complete: 116 traces, 56 nodes, 1661 tokens, audit alarms: none

$ python tools/validator/validate_nf1.py m1.brain
PASS: 5 shards (STATE, MODULATORS, EPISODIC, SEMANTIC, HORMONE), checksums ok
```

## Notes

- Teacher attachment is **session config, not file state** (DESIGN.md §4.17: the file persists, the LLM does not). `chat --teacher` attaches for the exchange.
- Pending (unbound) events are transient: saving mid-window drops them (window ≤ 300 ticks; documented behavior).
- Embodiment presets: means/spreads are biologically-informed digital analogues; distributions overlap heavily (e.g., male t-like 0.68±0.20 vs female 0.34±0.18) — embodiment nudges probabilities, never locks outcomes. Audit-visible, mutable, reversible (§4.8 contract).

## Next (M2 — Sleep & dreams)

Sleep pressure + stages (wind-down/light/deep/dream), replay + pattern completion, downscaling + pruning + gist clustering, dream synthesis with provenance, sleep reports, OS keychain slot, HTTP teacher adapter, live Cortex Canvas binding via `watch`.
