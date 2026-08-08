# UI-P0 NOTES — Eyes Backbone (encoder phase) complete

**Date:** 2026-08-05 · **Milestone:** BUILD-THE-BODY Phase 0 — all three encoders live.

## What was built

- **`handcrafted`** — original 16-dim extractor, untouched, default (backward compat: pre-encoder files load bit-identically).
- **`jepa`** — frozen V-JEPA 2 (facebook/vjepa2-vitl-fpc64-256, 326M) — merged 2026-08-05 (commit 662961a); ONNX backbone `models/vjepa2-vitl-fpc64-256/` (`.onnx` + `.onnx.data`, 1.22GB).
- **`onnx` (NEW this milestone)** — frozen DINOv2-small (onnx-community/dinov2-small, fp32, 88.5MB) → 384-dim embedding; downloaded to `models/dinov2-small/onnx/model.onnx` (gitignored; sha256 `f22797ea…c20a`).

## Design decision (honest deviation from the plan text)

The plan's P0-1 said "ONNX runtime crate wired in" (Rust ort). The jepa path (built first) proved the **Python sidecar architecture**: `media-extract.py` computes features outside Rust, the CLI receives `Vec<f32>` — encoder-agnostic, bit-exact (onnxruntime pinned to 1 thread), zero heavy Rust deps. The onnx option reuses that proven path instead of adding the ort crate + onnxruntime.dll vendoring on the mingw toolchain. Same determinism contract, same refusal semantics. If a Rust-native runtime is ever wanted (e.g., live preview at 60fps), it can be added later without touching the file format.

## Preprocessing (from official processor configs)

- jepa: shortest-edge 292 → center-crop 256² → /255 → ImageNet mean/std → tubelet ×2 (1,2,3,256,256). (VJEPA2VideoProcessor)
- onnx: shortest-edge **256** → center-crop 224² → /255 → ImageNet mean/std, **bicubic** resize (resample 3). (DINOv2 processor — caught the 256-not-224 detail from preprocessor_config.json)

## New machinery

- `create --encoder handcrafted|onnx|jepa` — model presence checked at creation; manifest records encoder + source model sha256.
- `verify` now includes **runtime model integrity**: streaming sha256 of the actual ONNX file the sidecar loads vs trusted constants (jepa `a08f2f68…`, onnx `f22797ea…`) → `runtime model: trusted` / `MISMATCH`. Load-time verification deliberately deferred (1.3GB hash per load too costly); verify-time covers it explicitly. Manifest records the model's *identity* (source checkpoint hash); the runtime check proves the machine's file is the known-good one.
- `expose` routes ANY non-handcrafted encoder through the encoder venv (resolved via `NEUROFORM_JEPA_PYTHON`, PYTHONPATH stripped).
- UI: encoder selector label updated (onnx now selectable).

## Acceptance evidence (real output)

```
=== ONNX SIDECAR DETERMINISM ===
a04c4d4d…  o1.json
a04c4d4d…  o2.json
onnx dims: 384 | identical: True

=== P0 E2E ===
created scratch-onnx.brain (tier standard, encoder onnx, ... seed 7, 14627 bytes)
exposed image "…jepa-test-img.ppm" ×1 (640x480, encoder onnx, 256 features — unlabeled)
  seed: 7   tier: standard   encoder: onnx   created: 1785931828
  encoder model sha256: f22797eabf810a75e41de68d378541ebea372122b25c4ce3ef25ff618250c20a
  runtime model: trusted (f22797eabf810a75)

=== JEPA REGRESSION ===
  encoder: jepa   encoder model sha256: 25466aef…  runtime model: trusted (a08f2f68…)

=== HANDCRAFTED CONTROL ===
  encoder: handcrafted   (no model, no runtime check — unchanged)
```

- Suite: `cargo test --release` → **116 passed, 0 failed** (unchanged — no engine regressions).
- Determinism: same seed 7 → identical brain_id + digest across all three encoders (`dbbc2ae7…`, `583f44d5…`) — encoder changes the eyes only.
- Refusal: missing model file → honest error at create (no file written); unknown encoder → honest error.

## Known gaps (honest)

- Runtime model integrity is verify-time only (not load-time) — documented above.
- DINOv2 embedding quality vs jepa vs handcrafted for the file's retrieval: unmeasured (Phase E experiment).
- Multi-thread ONNX speed-up untested (would need its own determinism verification).
