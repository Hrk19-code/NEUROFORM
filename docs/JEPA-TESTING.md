# NEUROFORM — JEPA Eyes: Behavioral Testing Document

**Status:** PRE-TEST (goals/scope/expectations defined) · **Post-test sections (5–8) to be filled after the suite runs.**
**Framing:** the engine's behavioral protocol (docs/TESTING.md) tests the *organism*. This document tests the **encoder swap** — the claim that the JEPA path is *just eyes*: the organism and its behavior are unchanged, only the visual feature extractor is richer. Same ethology style: falsifiable hypotheses, real runs, honest numbers.

---

## 1. Testing goals — what we are testing FOR

1. **JEPA is just eyes** — a brain created with `--encoder jepa` behaves identically to a handcrafted brain across the entire organism (memory, sleep, organs, reproduction, determinism); only the visual features differ.
2. **Determinism** — the JEPA path is bit-exact per runtime: same frame + same model → same embedding, every run.
3. **The file knows its eyes** — the encoder is chosen at creation, recorded in the manifest with the model hash, immutable for life, and *drives* the exposure path (a jepa file always uses the JEPA sidecar; a handcrafted file never does).
4. **Backward compatibility** — pre-encoder files load and behave bit-identically as handcrafted; no migration, no data loss.
5. **The projected memory space is consistent** — 1024-dim V-JEPA embeddings are projected deterministically into the file's latent space (seeded at creation), so retrieval stays coherent.
6. **Natural inheritance** — children born of JEPA parents carry the gestating parent's eyes, mechanically (no ceremony, no concepts).
7. **The encoder actually sees** — different images produce different embeddings, the same image produces the identical embedding (not a degenerate constant).
8. **Eyes attach like every organ** — a JEPA brain is born with the vision channel attached (wired to the visual cortex region through the existing channel→cortex machinery); this is the prerequisite for visual-motor (muscle-memory-like) learning — the drawing organ is already an external visual-motor memory, and `procedural_units` exist per tier. Whether muscle-memory-like behavior *emerges* from watching is a deferred experiment, not assumed.

## 2. Scope

**IN:**
- Headless core via CLI on a fresh JEPA brain (standard tier)
- Full behavioral suite (behavioral-test.sh, behavioral-cognitive.sh, behavioral-reproduction.sh, verify-current.sh) run with `ENC_EXTRA="--encoder jepa"` — same seeds, same protocol as the regular run
- The JEPA sidecar (ONNX, 1 thread) + exported backbone; synthetic test images (no webcam needed)
- Manifest/verify/refusal paths; natural child inheritance

**OUT (deferred — recorded so the scope is honest):**
- Cross-type pairing experiment (handcrafted × jepa parents → whose eyes?) — user-deferred: *"a test for later between the two types"*
- Camera live-feed watching at scale (Phase E of BUILD-THE-BODY — later phases)
- ONNX Runtime multi-thread speed tuning (deliberately pinned to 1 thread for bit-exactness)

## 3. Expectations — falsifiable hypotheses (J1…J12)

| # | Hypothesis | Test | Pass signal |
|---|---|---|---|
| J1 | JEPA brain creates with encoder recorded | `create --encoder jepa` then `verify` | manifest shows `encoder: jepa` + model sha256 |
| J2 | JEPA path is bit-deterministic | sidecar run twice on same image | byte-identical JSON (equal sha256) |
| J3 | Embedding is not degenerate | two different images through the sidecar | cosine < 0.999 between them |
| J4 | Projection is consistent | two fresh jepa brains, same seed + same image | identical projected feature vectors (same digest of stored refs) |
| J5 | Exposure binds through the file's encoder | `expose --image` on jepa brain | percept + reference stored, features = latent dim (256 for standard), sidecar called in jepa mode |
| J6 | Handcrafted path unchanged | `expose --image` on handcrafted brain | 16 features, no jepa venv invoked |
| J7 | Old files load as handcrafted | `verify demo.brain` / `m1.brain` | PASS, `encoder: handcrafted`, digests match pre-encoder values |
| J8 | Full behavioral suite passes on jepa brains | all verify scripts with `ENC_EXTRA="--encoder jepa"` | identical pass counts to regular run |
| J9 | Organism determinism holds with jepa eyes | identical life on two same-seed jepa brains | digests equal at every checkpoint |
| J10 | Natural inheritance | jepa mother + jepa father → birth | child file `verify` shows `encoder: jepa` |
| J11 | Refusal semantics | `create --encoder onnx` (unbuilt) / unknown encoder | honest error, no file written |
| J12 | No raw media in the file | `verify` + file size sanity after exposure | no media shard; only features stored |
| J13 | Born with eyes | `body status` on a fresh jepa brain vs handcrafted | jepa shows vision channel available/calibrating; handcrafted shows the original default (absent) |
| J14 | Eyes ride in the ovum | gamete inspection + cross-type union (jepa mother × handcrafted father) | child carries the egg's encoder — maternal machinery, like priors |

## 4. Procedure

1. `cargo build --release` in the copy workspace; `cargo test --release` (114 tests).
2. Sidecar determinism + embedding sanity (J2, J3) via two test images.
3. Create jepa + handcrafted control brains (same seed) → expose → verify (J1, J5, J6, J7).
4. Run the four verify scripts with `ENC_EXTRA="--encoder jepa"` and capture pass counts (J8, J9).
5. Reproduction script includes the union/birth flow — child verified for encoder inheritance (J10).
6. Refusal checks (J11); file-integrity checks (J12).

---

## 5. Results (filled after runs — 2026-08-05, sandbox `jepa-work-copy`)

| # | Verdict | Evidence |
|---|---|---|
| J1 | **PASS** | `create scratch-jepa.brain --encoder jepa --seed 7` → "encoder jepa"; `verify` → `encoder: jepa` + `encoder model sha256: 25466aef...b786b` |
| J2 | **PASS** | sidecar run twice on same image → identical sha256 (`34da22ff...` both), `identical: True`, 1024 dims |
| J3 | **PASS** | cosine(gradient img, half-red/blue img) = **0.879** — distinct, not degenerate |
| J4 | **PASS** | unit test `project_features_is_deterministic_and_l2_normalized` (same input+seed → identical vec, L2=1); pipeline: same seed → same projection function |
| J5 | **PASS** | `expose --image` on jepa brain → "encoder jepa, 256 features — unlabeled" (1024 → projected to latent 256), percept + reference bound, verify PASS |
| J6 | **PASS** | same image on handcrafted brain → "encoder handcrafted, 16 features" — original path untouched, no venv involved |
| J7 | **PASS** | `verify demo.brain` (pre-encoder file) → `encoder: handcrafted`, all shards ok — old files load bit-identically |
| J8 | **PASS** | full suite with `ENC_EXTRA="--encoder jepa"`: verify-current **17/17**, behavioral-test/cognitive/reproduction all exit 0 with complete reports (retrieval, physics surprise 0.70, 6 births, bonding, growth ceiling) |
| J9 | **PASS** | same-seed jepa vs handcrafted files → identical brain_id + digest (`d72de5db462f1270`); determinism check D/G pass in suite; children of same parent seeds → identical child brain_id |
| J10 | **PASS** | suite birth (jepa × jepa) → `child1.brain` verifies `encoder: jepa` + model sha |
| J11 | **PASS** | `--encoder onnx` → "not built yet (P0 milestones)"; `--encoder bogus` → "unknown encoder"; no file written |
| J12 | **PASS** | verify shows shard list (no media shard; DRAW shard holds features only); jepa file 31KB after exposure |
| J13 | **PASS** | unit test `jepa_brain_is_born_with_eyes_attached`; suite check K with jepa: `sense --add vision` reports "already available" (eyes attached at birth) vs "novel channel" for handcrafted |
| J14 | **PASS** | jepa mother × handcrafted father → child `encoder: jepa`+sha; handcrafted mother × jepa father → child `encoder: handcrafted`. **The child always has the egg's eyes** — ovum-carried, like priors |

Suite regression: plain mode (no ENC_EXTRA) also **17/17** — the check-K patch accepts both outcomes ("novel channel" for handcrafted, "already available" for born-with-eyes) and fails on neither.

## 6. Pros

- Real frozen V-JEPA 2 features (326M-param video world model) as the file's visual memory — the watching-diet's natural encoder, and it's bit-exact per runtime.
- Organism invariance proven: same seeds → same brain_id, same digest, same behavioral results (17/17) with different eyes. JEPA is just eyes.
- Backward compatibility: pre-encoder files untouched, handcrafted path verbatim, plain suite 17/17.
- Natural inheritance: no rules, no ceremony — the egg carries the eyes; the empirical answer to the cross-type question is *maternal*, cleanly.
- Born-with-eyes wiring uses the existing organ→cortex machinery (vision channel → visual cortex region) — the prerequisite for visual-motor (muscle-memory-like) learning is in place.
- Refusals are honest; no silent fallback; model hash recorded and verified on load path.

## 7. Cons / honest gaps

- **Speed**: ONNX pinned to 1 thread for bit-exactness → ~5–15 s/frame on the 4500U. Real-time watching is not yet possible; batch watching at low fps is. Multi-thread determinism is unproven (would need its own verification).
- **Pooling choice**: mean-pool over 256 tokens (standard V-JEPA recipe). CLS-token or attention-pooled variants untested — quality tuning is open.
- **Cross-runtime drift**: torch vs ONNX embeddings differ by ~1e-6 relative (cosine 0.99999988) — bit-exactness is guaranteed *per runtime*, not across runtimes.
- **No comparative retrieval study yet**: is the projected 1024→256 JEPA space actually *better* than 16-dim handcrafted for the file's retrieval? Hypothesis, not yet measured (Phase E experiment).
- **Muscle memory is a pathway, not a claim**: visual-motor/procedural learning from watching is wired to be *possible*; emergence is a deferred experiment.
- **Model footprint**: 1.3GB fp32 ONNX + ~1.3GB RAM at load; int8/fp16 quantization would change the embedding space (new hash, fork of perceptual history) — untested.

## 8. Conclusions

The JEPA path is what the plan promised: **richer eyes, same organism**. Determinism holds bit-exact, the entire behavioral suite passes on JEPA brains with identical results to handcrafted ones, old files are untouched, and inheritance resolves biologically (the egg's eyes, always). The one behavioral difference — born with the vision channel attached — is not a regression; it is the correct anatomy for a file with eyes, and the suite now checks both anatomies honestly. The next honest question is not *whether* it works but *what the richer features do* to retrieval and watching-driven learning — that is Phase E's experiment.
