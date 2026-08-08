# M8 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M8 (intuitive physics learner) — per DESIGN.md §4.12, §20 and master prompt §11.

## Scope delivered

Blank-slate prediction-error learner (`packages/brain-core/src/physics.rs`):

- **Starts at maximum uncertainty**: every learned rate is 0.500 with 0 confidence. No physics is taught — no gravity constant, no Newton, no labels.
- **Learns from raw observation frames only**: entities with position/motion/support/containment/contact — features, not names.
- **Learned world model (all emergent)**:
  - fall-when-unsupported ("gravity"), stay-when-supported, stay-when-contained, inertia (keep moving), collision response, permanence (contained → still contained).
  - Beta-count style rates with confidence; deterministic.
- **Prediction error → surprise**: qualitative predictions (will it move?) vs. observed; violations produce surprise 0..1.
- **Surprise is adaptive**: high surprise binds an episodic percept (src=physics), nudges curiosity upward (violations draw attention — the file gets curious about what it can't predict), and lights the parietal cortex region (M8's addition to the cortex map).
- **PHYS shard** — 12th shard; validator + format tests updated (11→12).

## Verified (103 tests, suite 16/16)

- Blank slate: rates 0.5, confidence 0.0.
- Learns gravity from repeated unsupported falls (rate > 0.9, confidence > 0.8).
- Learns support + containment from repeated observation.
- Containment violation (object suddenly moving) → surprise > 0.3, rule logged.
- Familiar scenarios surprise less after learning (0 after 20 training rounds).
- Deterministic digest; persistence across save/load (PHYS shard).
- Live demo: 30 raw frames → fall 1.00 / support 1.00; permanence violation → surprise 0.70, curiosity nudge, percept retrievable (score 0.815, src=physics).

## Honest notes

- Qualitative only: it learns *whether* things move, not *how fast* (magnitude learning is out of scope; the surprise model penalizes motion mismatch only).
- The world model is experience-shaped: one containment violation made containment rate drop to 0 — the model mirrors what it has actually seen.
- Boundaries unchanged: learning machinery authored, rates emergent, no self-modification of the learning rules.

## Next

PFC executive milestone (§4.2) — emergent goals, hierarchical planning via sequence chunking, goal-biased attention, working-memory gate, inhibition, metacognition → audit; decision weighting over physics predictions. Then the final behavioral test (docs/TESTING.md, H1–H9) and the paper (docs/PAPER.md).
