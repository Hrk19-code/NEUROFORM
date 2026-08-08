# M6 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M6 (Body organ) — per DESIGN.md §5–6, §18.7, §20 and master-prompt Tab 5.

## Scope delivered

- **Body Schema** (`body.rs`, §18.7 layout): device profile, per-channel permission + calibration state (uncalibrated/calibrating/calibrated, confidence, error rate, sample count), **explicitly listed unavailable senses** (vision/audition = no-permission — absence is modeled, not ignored, §6.1), 4×4 touch map + extent, orientation model (gravity/tilt/posture), motion axes, ownership confidence, aggregate calibration confidence, and **4 dormant actuator placeholders** (head/torso/arms) with `motor_enabled: false`, `state: None` — verified by test that no actuator can ever be enabled.
- **Touch ingestion** (§5.1): receptor-class decomposition of one contact event — FA (fast-adapting transient), SA (slow-adapting *sustained* pressure: pressure × duration), SA2 (motion/stretch), FA1 (fine detail), SA2b (broad contact). Affective interpretation with priors only (soothing/grounding/intimate/alerting/harsh/playful/neutral…), familiarity via cosine against **compressed touch memory** (5-dim features only, never raw sensor data; salience-tagged, decayable, forgotten without reinforcement). Unfamiliar "different today" is a prediction error but never masks a strong affective prior (regression-tested). Soothing touch during stress → regulation capacity + safety up; harsh/alerting → safety down.
- **Motion ingestion** (§5.2): canal-like rotational high-pass → abruptness; otolith-like gravity/linear; posture estimation (still/upright/lying/moving/transport) with gravity-alignment before excess-energy checks (a resting upright device isn't "moving"). Abrupt motion → alertness up, safety down; rhythmic → safety up. Binds as motion-stream percept.
- **Interoception** (§5.4): telemetry analogues (energy load, processing pressure, memory pressure, session length, interaction load) → aggregate interoceptive load. High load drives fatigue + irritability up, social openness down, binds as interoception-stream percept.
- **Novel-sense integration sequence** (§6.2, 8 steps): detection → schema expansion → calibration (confidence = 1 − 1/(1+n/25); calibrated ≥ 0.85 ≈ 142 samples; running error-rate mean over all samples) → memory formation (expansion binds as an `embodiment-expansion` percept) → **sleep-based integration** (step 7 runs inside `Brain::sleep`, finalizes calibrating channels, raises ownership confidence, counted as `sensory_integrated` in the sleep report) → long-term adaptation (stage 8). The *reaction* is not scripted — the machinery is fixed, what the file feels emerges.
- **Format**: 10th shard `BODY`; `FileContents.body` serde-defaulted (pre-M6 files load a fresh device body); validator extended (`[ok] shard BODY: …`); sleep `StageWork` gains `sensory_integrated` (serde-defaulted, backward compatible).
- **CLI**: `body status|touch|motion|interocept|sense --add|calibrate|motor`, `inspect`/snapshot body line (ownership, calibration, posture, channels, integrations, interoceptive load).
- **Tests**: 85 total (was 74) — 11 new: decomposition determinism + separation, affective priors, familiarity + non-masking, touch memory decay, posture/abruptness, interoception load, calibration tracking, integration sequence, motor hooks dormant, digest determinism, brain-level binding + persistence (`body_events_bind_and_persist`).

## Bugs caught (all fixed, all mine this time)

1. **SA channel formula**: `pressure × (0.4 + 0.6·duration)` let brief hard pressure dominate — the test caught that a 1500 ms gentle press read lower than an 80 ms tap. SA now integrates *sustained* contact (`pressure × duration`), and the interpreter reads hard brief contacts via FA (transient energy) instead. The test was right; biology says SA is the sustained integrator.
2. **Touch-memory fold sentinel bug**: a `u32::MAX` placeholder in the manual fold was used as an index → index-out-of-bounds on the second distinct pattern. Replaced with `enumerate().max_by`.
3. **Posture check order**: gravity magnitude ≈ 1.0 tripped the "moving" check before "upright" — resting upright read as moving. Reordered: excess energy (transport/moving) *before* gravity alignment, `|gz| < 0.7` guard for off-gravity acceleration.
4. **Unfamiliarity masked strong priors**: the familiarity check relabeled an unfamiliar harsh tap from Alerting → Unfamiliar. Familiarity is a separate soft label (§5.1); it now only relabels Neutral. Regression test added.
5. **Calibration error-rate running mean**: only updated on outliers, so it never diluted (stuck at 1.0 after the first outlier). Now a running mean over all samples (outliers dilute with time).
6. **Suite shard count** (script): E step updated to 10 shards incl. VOICE+BODY; new **K: body lifecycle** step exercises touch/motion/interocept/sense/calibrate/motor end-to-end.

## Exit criteria status (M6)

- [x] Touch/motion/orientation ingestion with receptor-class decomposition
- [x] Body schema (available + explicitly unavailable senses, touch map, orientation, ownership)
- [x] Calibration — confidence tracks samples and reaches calibrated; error rate tracked
- [x] Novel-sense integration sequence runs per spec (8 steps, sleep integration verified)
- [x] Interoception from telemetry → fatigue/openness/irritability effects
- [x] Motor hooks verified disabled (test + `body motor` always reports 0 enabled)
- [x] BODY shard + validator + serde-default migration
- [ ] Live sensor binding (real touchscreen/motion/telemetry ingestion) — desktop-shell milestone
- [ ] Body-schema visualization, touch-field view, calibration UI — desktop-shell milestone
