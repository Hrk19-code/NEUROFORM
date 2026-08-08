# M4 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M4 (Drawing organ + executive initiative) — per DESIGN.md §20.

## Scope delivered

- **Operation-graph canvas model** (`drawing.rs`): `Canvas` (id, name, dims, op graph, materialized layers/strokes, reference board), layer ops (add/group/opacity/blend/transform/delete), stroke ops (brush, RGBA color, width, pressure-curve points). The op graph is the source of truth — editable history, deterministic replay digest (§9.4). The editor UI (blend-mode panel, stabilizer, symmetry, history scrubber) is the desktop-shell milestone and consumes exactly this model.
- **Stroke features** (16 dims, pure + deterministic): 8-bin direction histogram, curvature mean/std, pressure mean/std, log length, bbox aspect, hue, log width. No learned params, no cloud.
- **Motif memory**: streaming clustering (cosine ≥ 0.75 joins; centroid = rolling mean) — repeated shape families consolidate into motifs with salience and stroke provenance (§9.6 visual memory). Verified: two similar strokes → one motif; distinct families separate (test).
- **Aesthetic preference signals**: quantized palette usage, rolling width/pressure tendencies, motif engagement (§9.6).
- **Reference board** (§3.6.3, §5.8): `ReferenceAsset` entries — vault ref + extracted features only, **never raw media**. Image feature extraction via `tools/media-extract.py` (Pillow sidecar, deterministic 16-dim: hue histogram, sat/value, edge density, warmth, size); video entries supported at manifest level (decoder pipeline = same sidecar, later).
- **Drawing → memory binding**: each stroke binds as a drawing-sourced percept (keywords carry motif/stroke ids; valence from color warmth) → episodic traces + semantic distillation.
- **Brain-modulated drawing assistance**: `assist_drawing` = retrieval + motif summary + aesthetic tendencies → teacher (or degraded).
- **Executive initiative — unprompted speech** (user request, spec §10.3 "autonomy request if enabled"): `InitiativeSystem`. **Default OFF.** When enabled: condition-driven (curiosity > 0.7, sleep pressure ≥ 0.6, |valence| > 0.5, unbound user input pending), rate-limited (default 4 sim-hours between initiatives), quiet-hours aware (sim-clock), teacher-mediated only (degraded mode never fabricates speech), every initiative logged with its trigger kind (bounded log, surfaced in snapshots and `life` output). Enable/quiet config **persists in the manifest**.
- **Format**: 8th shard `DRAW`; manifest gains `autonomy_enabled`/`autonomy_quiet_start`/`autonomy_quiet_end` (serde-defaulted for old files); validator extended.
- **CLI**: `draw new/layer/stroke/ref/motifs/canvases/assist`, `autonomy --enable|--disable|--quiet-start|--quiet-end|--status`, `life --autonomy` (reports initiatives), `inspect` drawing + autonomy lines.
- **Tests**: 62 total (was 54) — feature determinism/sensitivity, motif family separation, op-graph deterministic replay, aesthetic signals, drawing binds into memory, drawing+refs persist, autonomy gated/audited/rate-limited/quiet-hours/no-teacher, autonomy config persists.

## Bugs caught (all fixed)

1. `drawing.rs` compile fixes: `Canvas::refs` missing from the initializer, `stroke` moved before `stroke.id` read, dropped closing brace during refactor.
2. `fval` CLI helper missing (added next to `fint`).
3. **Initiative test timing**: curiosity mean-reverts toward baseline during 10k ticks — the test set it before the drift period, so it was below the 0.7 threshold at enable time. Set after.
4. **Rate-limit test contradiction**: interval 0 disables the rate limiter by definition — the test re-arms with a real interval to verify suppression.
5. **Verify-script path bug** (script, not code): python `-c` strings choke on `C:\Users` (`\U` escape) — forward-slash the temp path via `tr '\\' '/'`.

## Exit criteria status (M4)

- [x] Operation-graph canvas model with deterministic replay
- [x] Stroke feature extraction (deterministic, local)
- [x] Motif memory (streaming clustering, provenance, salience)
- [x] Aesthetic preference signals
- [x] Reference board: image refs with **real extracted features** (Pillow sidecar, verified)
- [x] Drawing binds as visual-spatial memory (traces + semantic)
- [x] Brain-modulated drawing assistance
- [x] **Unprompted speech**: initiative system, default OFF, gated, rate-limited, quiet-hours, audited, persisted
- [x] DRAW shard + manifest fields + validator
- [ ] Editor UI (blend modes, stabilizer, symmetry, canvas history) — desktop-shell milestone
- [ ] Video decoding pipeline — same sidecar pattern, deferred with the shell

## Verified runs (real output, 2026-08-04)

```
$ cargo test --release
test result: ok. 62 passed; 0 failed

$ python tools/media-extract.py ref-garden.png
{"width": 320, "height": 200, "features": [0.23999, 0.008301, ...]}  # 16 dims

$ neuroform draw new m1.brain --name "Sunset" --w 512 --h 512 --save
canvas #1 "Sunset" (512x512) created
$ neuroform draw stroke m1.brain --canvas 1 --layer 1 --points "10,10,0.5;30,20,0.8;60,15,0.4;100,40,0.9" --save
stroke on canvas #1: motif #0
  bound: 134 traces, 64 semantic nodes; 1 motifs, 2 strokes
$ neuroform draw ref m1.brain --canvas 1 --name garden --kind image --vault-ref ref-garden.png --save
reference #1 "garden" (image, 320x200, 16 features) on canvas #1
$ neuroform draw motifs m1.brain
visual memory (1 motifs): motif #0: 2 strokes, salience 0.30
$ neuroform autonomy m1.brain --enable --quiet-start 23 --quiet-end 6 --save
autonomy: enabled true, quiet 23:00–6:00 ...
$ neuroform life m1.brain --days 4 --autonomy --no-autosave
initiatives: 1 (unprompted speech, enabled: true) — last: [unspoken] [amber] i remember
  something like this — about speak unprompted (unspoken).
```

## Notes

- Unprompted speech is honest-by-construction: no teacher → no initiative (degraded mode never fabricates); every instance is logged with its trigger; quiet hours and the 4h rate limit bound pestering; the bias-audit engine's loop/overfit monitors (§14) are the next layer of defense when autonomy is on.
- The `life --autonomy` run showed the "unspoken" trigger (unbound user input pending) — the file noticed unacknowledged input and spoke about it, through the teacher, costed in tokens.

## Next (M5 — Voice organ)

Vocal apparatus model (breath/larynx/tract/articulation state), prosody planner, TTS backend parameter mapping, voice identity with drift + override, voice memory — plus the deferred OS keychain slot and HTTP teacher adapter.
