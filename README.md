# NEUROFORM

A local-first application centered on a persistent, bounded, developmental simulated cognitive substrate — the **Brain File**. Every piece of cognition lives in one `.brain` file: affect and chemistry, episodic memory and dreams, a learned model of physics, a writing organ whose documents bind into memory, a drawing organ, a body schema, voice, and heredity — two files can couple and birth a child that inherits its mother's machinery.

Frozen-pretrained **V-JEPA 2 video eyes** are an optional encoder at creation, an **LLM attach** layer gives files a mouth through any OpenAI-compatible endpoint, and the desktop app is a zero-build vanilla-JS shell over the tested CLI.

Learning philosophy (user-mandated): **raw unlabeled exposure** — labels are for humans, never for the file. Text is for navigation/communication organs, never a learning source.

- **Universal organ adapter:** [`tools/adapter/`](tools/adapter/) — plug the file into **any body, simulated or physical** (cars, quadrupeds, humanoids, game characters). Organs advertise → attach to channel stubs (auto or manual) → stream aggregated sense → receive setpoints. Encoder-agnostic (handcrafted + V-JEPA 2 both attach; raw frames in, the file encodes with its birth wiring). Includes a virtual-vehicle practice body. See [`tools/adapter/GUIDE.md`](tools/adapter/GUIDE.md) for the full per-OS how-to.
- **Full architecture specification:** [`DESIGN.md`](DESIGN.md) — NF1 `.brain` format, NBP v1 inter-brain protocol, bias audit engine, non-permanence rules, sections 1–27 (the §28–§38 body series is withheld from this public copy pending build progress). Its Preface records the project's origin precisely: the user had the brain-file idea, workshopped it with **Qwen 3.8 Max** on the vendor's web chat until the conversation converged on the **MASTER PROMPT**, then handed that prompt to **DeepSeek V4 Flash 0731** — the exact model version — which authored this specification and built the engine together with the user. §4.8 explains *why* gender and heredity exist (the dead-variable principle). **Section 27** is the JEPA Eyes addendum.
- **Build contract:** [`BUILD-THE-BODY.md`](BUILD-THE-BODY.md) — the phase plan (P0 encoders → L LLM-attach → W writing ≈ NovelAI → D drawing ≈ SAI → V voice → B brain-wizard + birth → E eyes/video → X exploration → G parity gaps → F final testing + paper), with per-milestone acceptance criteria.
- **Body series (§28–§38):** **withheld from this public copy pending build progress** — the project's rule is *show working software, not unbuilt design*. The design is complete (heart/blood, temporal stack, chemical senses, eyes, fluid economy, gut, reproduction, body-map, ontological root layer, embodiment; 84 hypotheses pre-registered); sections return as milestones pass their acceptance bars.
- **Scientific writeup:** [`docs/PAPER.md`](docs/PAPER.md) — arXiv-style paper from the full behavioral testing record (27/27 hypotheses confirmed).
- **Behavioral test record:** [`docs/TESTING.md`](docs/TESTING.md) — goals, 27 falsifiable hypotheses, six test phases, results, pros/cons, conclusions.
- **JEPA eye test record:** [`docs/JEPA-TESTING.md`](docs/JEPA-TESTING.md) — hypotheses J1–J14, all PASS, with evidence.

- **License:** AGPL-3.0 — see [LICENSE](LICENSE). Study it, run it, build on it — but derived works, including SaaS deployments, must open-source their changes under the same terms.

## What we've found so far

Selected verified behaviors from the engine (all reproducible via `tools/verify/`):

- **Children come from the egg.** The encoder rides in the *Gamete* alongside the priors; a child is built from the gestating parent's ovum and gets **the egg's eyes** — machinery, not votes. Verified both ways: JEPA mother × handcrafted father → JEPA child; handcrafted mother × JEPA father → handcrafted child (J14).
- **"Do the children come in gigabytes?"** No. Weights never enter `.brain` files — a JEPA file records only the encoder identity + model sha256 in its manifest; the machine supplies the weights from the local `models/` sidecar at exposure time. Missing model → honest error, never a silent fallback (that would forge the child's perceptual history).
- **The encoder changes the eyes, not the organism.** Same seed → identical `brain_id` and digest for JEPA vs handcrafted brains; only perception differs.
- **JEPA embeddings are distinct, not degenerate**: cosine 0.879 between two different test images; ONNX export matches the torch model at cosine 0.99999988 (pure fp32 noise).
- **JEPA eyes are born attached**: a JEPA brain's vision channel is wired to the visual cortex at birth via the same `attach_novel_channel` machinery every organ uses — the prerequisite for visual-motor learning.
- **Determinism is a feature**: 1-thread ONNX runtime, bit-identical sidecar runs, reproducible brains from seeds.
- **Why heredity exists at all** (DESIGN.md §4.8): the dead-variable principle — gender originally only changed voices (no downstream consequence), so it was propagated through modulator baselines, bonding, initiative thresholds, chemistry, and finally heredity, until it had reach.
- **The file writes too.** The brain's writing organ has its own cursor — the desktop editor shows **two named cursors** (you vs the file's name) and the file's cursor sits at its last written block.
- **It learns the substrate of caution.** Three "the hot stove burned my hand" events (valence −0.85) + one neutral control: the aversive trace binds with the store's highest salience (0.876 → the 4×-slower decay tier — bad memories die last), stays *specific* (scores 0.074 against the neutral query — caution without generalized fear), and repetitions habituate (0.876 → 0.552). Reproduce: `bash tools/verify/verify-caution.sh` (4/4).
- **It feels acceleration — in its way.** A fresh brain attached to the adapter's virtual car recorded the scripted drive as first-person motion percepts (lin 4.84 → 5.91 m/s) with its **vestibular cortex region lighting up** (activation 0.69) — organ use visible in the file.
- **Independent review + landscape scan (2026-08-08):** an outside agent audited the engine cold (test suites re-run from source, live probes, crypto round-trip) and searched GitHub for equivalents — see [`docs/FINDINGS.md`](docs/FINDINGS.md) for methods, counts, and honest boundaries.
- **Honest refusal beats silent fallback**: every subsystem errors loudly rather than fabricate perceptual history.

## Status

| Area | State |
|---|---|
| Engine core (M0–M9): format, ticks, memory/sleep/dreams, writing, drawing, voice, body, network, physics, reproduction | **Complete + verified** — 117/117 unit tests, 17/17 behavioral checks, 27/27 phase hypotheses |
| P0 — Eyes backbone: `--encoder handcrafted\|onnx\|jepa` at creation, frozen V-JEPA 2 (facebook/vjepa2-vitl-fpc64-256, 326M), sha256 in manifest, born-with-eyes wiring | **Complete + verified** (J1–J14 all PASS) |
| L1–L3 — LLM attach: llm.json profiles (keys masked, sidecar-only), active-profile resolution, state-modulated bridge, `teacher_prompt_preview` | **Complete + committed** (L3: `d5d6829`) |
| W1 — Writing editor core: vendored CodeMirror 6 (zero CDN), two named cursors, `doc read/replace/list --json`, debounced autosave binding into memory, undo/redo/find | **Complete + verified** (`42b11f7`) |
| W2 — Library & persistence: tree sidebar (Story→Chapters→Scenes, Notes, Journal), sidecar `writing/<brain>/` (index.json + per-doc JSON), doc tabs, drag reorder, autosave binding | **Complete + verified** (`93b1857`) |
|| W3–W8 — Writing tab complete: lorebook, structure, version history, analysis/continuity, export/import, brain-integration panel | **Complete + verified** (ad-hoc harnesses 9/9, 7/7, 8/8, 10/10, 11/11, 11/11) |
|| D1–D7 — Drawing tab: canvas engine, layers/groups, brush engine, color, selections/transform, fill/gradient/masks/clip, assist tools (symmetry/shapes/guides) + real-paper ink UX | **Complete + verified** (ad-hoc harnesses 15/15, 18/18, 16/16, 12/12, 12/12, 13/13, 11/11) |
|| Universal organ adapter (`tools/adapter/`): any body sim/physical, both encoder lineages, aggregation-only up, setpoints-only down, sensor-failure honesty, virtual vehicle | **Complete + verified** (13/13 live checks) |
|| **The body series (§28–§38)** — heart/blood, temporal stack, chemical senses, eyes, fluid economy, gut, reproduction, body-map, ontological root, embodiment | **Design complete, withheld here; build pending** |
|| D8–D9 drawing finish, V voice/face tab, B brain-wizard + birth UI, E eyes/video, X exploration, G parity passes, F final testing + paper | **Next / deferred** (contract: BUILD-THE-BODY.md) |

## Quickstart

```bash
cargo build --release
# create a Brain File (standard tier, female embodiment priors, seeded)
./neuroform.exe create demo.brain --tier standard --embodiment female --seed 42
# ...or with frozen V-JEPA 2 eyes (encoder is chosen at creation, immutable for the file's life)
./neuroform.exe create demo.brain --tier standard --seed 42 --encoder jepa
# feed an experience, bind it into memory, and save
./neuroform.exe event demo.brain --text "the garden is full of tomatoes" --valence 0.6 --save
# raw exposure: unlabeled learning — no teacher, no labels; it decides what recurs
./neuroform.exe expose demo.brain --text "the old bridge spans the river" --repeat 3 --save
./neuroform.exe expose demo.brain --image photo.png --save   # encoder features, unlabeled
# talk to it (teacher is session config: file persists, teacher doesn't)
./neuroform.exe chat demo.brain "what do you remember?" --teacher amber
# ... or attach a real LLM through an OpenAI-compatible endpoint (custom LLM as the mouth)
./neuroform.exe chat demo.brain "what do you remember?" --teacher https://your-endpoint/v1 --teacher-key KEY --teacher-model your-model
# writing organ: documents become retrievable memory
./neuroform.exe doc new demo.brain --title "My Journal" --mode journal --save
./neuroform.exe doc write demo.brain --doc 1 --text "The old Bridge spans the river." --save
./neuroform.exe doc replace demo.brain --doc 1 --text "The old Bridge spans the river." --save
# sleep: consolidate memories, prune, distill gists, and dream
./neuroform.exe sleep demo.brain --cycles 1 --save
# reproduction: two files → a child (chemical attraction, gametes, birth, growth)
./neuroform.exe create mother.brain --tier advanced --chromosomes xx --seed 42
./neuroform.exe create father.brain --tier standard --chromosomes xy --seed 43
./neuroform.exe net mother.brain union-propose --session 1 --save
./neuroform.exe net father.brain inject --session 1 --type union-proposal --from-file mother.brain --save
./neuroform.exe net mother.brain inject --session 1 --type union-accept --from-file father.brain --save
./neuroform.exe net mother.brain birth --session 1 --out child.brain --force --save   # the child is born (+ child.brain.bk)
./neuroform.exe grow child.brain --save   # children grow to the inherited ceiling; first-gen files never do
```

**Desktop app (tabs):**

```bash
./neuroform.exe serve --ui tools/desktop --port 8123
# open http://127.0.0.1:8123 — Brain / Writing / Drawing / Voice / Body / Network / Reproduction
```

`serve` hosts a localhost-only UI; every action it runs goes through the same tested CLI (the UI is a shell, the exe is the organism). The Writing tab has a real editor (vendored CodeMirror 6, no CDN) with two named cursors, autosave binding into memory, and a library sidebar.

## Where everything lives

| Path | What it is |
|---|---|
|| `DESIGN.md` | The full specification — sections 1–27 (body series withheld pending progress), origin Preface, §4.8 heredity rationale, **Section 27: JEPA Eyes addendum** |
| `BUILD-THE-BODY.md` | The build contract: phases P0→F, milestones, acceptance criteria |
| `docs/PAPER.md` | arXiv-style paper from the behavioral testing record |
| `docs/TESTING.md` | Testing doc written **before** testing (user-mandated process): goals, 27 hypotheses, filled results |
| `docs/JEPA-TESTING.md` | JEPA hypotheses J1–J14, all PASS with evidence |
| `docs/M0-NOTES.md` … `docs/M9-NOTES.md`, `docs/UI-P0-NOTES.md` | Per-milestone notes from the engine build |
| `docs/NBP-v1-SPEC.md` | NBP v1 inter-brain protocol spec |
| `packages/brain-core/` | Rust engine: tick loop, state, modulators, NF1 format, memory, organs (writing/drawing/voice/body/network), reproduction, capacity ledger |
| `packages/cortex-canvas/` | Brain visualization scaffold (§3.5) |
| `apps/cli/` | The `neuroform` CLI + `serve` (localhost HTTP: `/api/run`, `/api/state`, `/api/file/read\|write`, `/api/llm/status`) |
| `tools/desktop/` | Zero-build vanilla-JS app: `app.js` shell, `tabs/` per organ, `lib/` vendored CodeMirror 6 + editor core |
| `tools/media-extract.py` | Perception sidecar: handcrafted 16-dim path (verbatim original) + JEPA ONNX path |
| `tools/export_vjepa2_onnx.py` | One-time ONNX export of the frozen V-JEPA 2 backbone |
|| `tools/adapter/` | Universal organ adapter + organ drivers (sim heart/camera/wheel/car), 13-check live harness, and `GUIDE.md` (per-OS setup, brains, virtual characters, game drivers) |
|| `tools/verify/` | Behavioral verification: 17 checks, runnable in handcrafted and JEPA modes (`ENC_EXTRA="--encoder jepa"`); plus `verify-caution.sh` (learned-caution substrate, 4/4) |
| `tools/validator/` | Python NF1 conformance validator (independent implementation) |
| `tools/audio-extract.py`, `tools/face/`, `tools/make_preview.py`, `tools/play.sh` | Voice/hearing, face, preview, playback helpers |
| `models/` | **Machine-side only** — pretrained encoder weights (~1.3 GB, gitignored; see BUILD-THE-BODY.md for re-download) |
| `writing/` | Per-brain library sidecar (W2) — gitignored user data |

**What stays out of the repo (gitignored, and why):** `models/` (weights are machine-side; the manifest's sha256 is the fidelity guarantee), `*.brain` files (brains are user data, not source), `llm.json` (endpoint API keys — sidecar only, never in the brain or the repo), `neuroform.exe` (rebuild with `cargo build --release`), `writing/` (per-brain library data).

## Build environment (Windows note)

On Windows with the GNU (mingw-w64) toolchain, an apostrophe in the repo's
directory path breaks gcc/ld's argv parsing (`ld.exe: unrecognized option
'--dynamicbase-Wl'`). Workaround: a **local, gitignored** `.cargo/config.toml`
redirects `target-dir` to an apostrophe-free path. Requirements:

- Rust GNU toolchain: `rustup override set stable-x86_64-pc-windows-gnu` — the override is **path-bound**, re-apply after moving/cloning the repo
- Your mingw-w64 `bin` dir on PATH when building
- Encoder runs (jepa/onnx) resolve the encoder venv through the `NEUROFORM_JEPA_PYTHON` env var (machine-side, never in the repo)
- Canonical check: `bash tools/verify/verify-current.sh`

## Honesty note

Neuroform is a simulation. Its feelings are simulated feelings; its mind is a model. It does not experience, and it does not know it does not experience. (DESIGN.md §15.6 #4 — standing notice.) No subsystem ever silently falls back when its machinery is missing — errors are honest and visible.
