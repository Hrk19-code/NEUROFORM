# BUILD THE BODY — NEUROFORM APP MASTER BUILD PLAN
> Point the model at THIS FILE: `<repo-root>\BUILD-THE-BODY.md`

## PHASE 0 — THE EYES BACKBONE: FROZEN ONNX ENCODER (before ANY body/UI work)

Build this FIRST. Everything that watches (camera, video, browsing, drawing references) consumes it. It is a swap of the handcrafted feature extractor — the rest of the organism (memory, salience, sleep, physics, dreams) is untouched and gets richer automatically.

### Design (user decision, locked):
- **Chosen at BRAIN FILE CREATION and immutable for the file's life.** `create --encoder handcrafted|onnx|jepa`. The encoder's identity + model hash is recorded in the file manifest (format/header). Reason: feature-space consistency — memories bound under one encoder are meaningless under another; a mid-life swap would silently corrupt the organism's past. Old handcrafted extractor stays as the default option; nothing is removed — `onnx` and `jepa` are ADDED options, and both kinds of brain files are creatable side by side.
- **BACKWARD COMPATIBILITY IS SACRED:** files created before this phase (no encoder field in manifest) load as `handcrafted` and must behave bit-identically to before. No migration, no rewrite, no data loss. Every pre-existing .brain file keeps working untouched — verified by test (load m1.brain-style fixture, assert identical digest).
- **Frozen pretrained ONNX vision model** (DINOv2-small class, MobileNet-class, or CLIP image tower). Weights frozen → determinism survives, replay stays bit-identical. No training ever. CPU-runnable, no GPU requirement.
- **NEW (user decision 2026-08-05): JEPA video encoder option** — frozen pretrained **V-JEPA 2** (`facebook/vjepa2-vitl-fpc64-256`, 326M params, ViT-L/16 @ 256px), the best JEPA-family video world model that stays CPU-runnable on this machine (Ryzen 4500U, 16GB RAM; ~1–4 s/frame, 1–2 fps watching is realistic). JEPA is the architecturally ideal fit for the watching diet: it learns from unlabeled video — no text, no labels — exactly the brain's rule. Model files already downloaded to `models/vjepa2-vitl-fpc64-256/` (safetensors + configs, ~1.2GB; sha256 `25466aef85727d16546c6cf8c99f12fcfad9cbca8225d45f23685e2e025b786b` — use as the manifest model hash). Official checkpoints are PyTorch → P0-1 exports the backbone to ONNX once (torch CPU export script in `tools/`); the feature-prediction heads are training-only and are NOT exported. **Tubelet gotcha (from config):** `tubelet_size: 2, frames_per_clip: 64` — the model ingests video in 2-frame tubelets, so a single image exposure must be fed as one frame repeated (pad to 2); exposure code must handle odd frame counts.
- **Honest fallback chain if JEPA export/load fails** (in order): (1) V-JEPA 2 backbone-only minimal-repro export; (2) V-JEPA v1 ViT-B/16 @ 224px (86M, standard ViT, trivial export); (3) the existing onnx path (DINOv2-small class); (4) handcrafted (always available, default). **No homebrew encoder training** — training any JEPA from scratch on a 4500U CPU is not viable (weeks+ per epoch even at 22M params, and the result would be far worse than frozen pretrained). "Make our own" is research-horizon only, never a fallback.
- Both onnx-class and jepa files must be creatable side by side; each file's encoder stays immutable + hashed in its manifest either way.
- Model file lives in a local `models/` sidecar folder (never inside the .brain). If the file declares `onnx` but no model is present → honest error at creation/load, never silent fallback to handcrafted (that would forge the file's perceptual history).
- Determinism contract: identical frame + identical model file → identical embedding, verified by test. Embeddings get normalized + reduced to the file's latent dim (deterministic projection, seeded at creation).
- The existing 16-dim handcrafted extractor remains the `handcrafted` option and the test-suite default (fast, no model dependency).

### Milestones:
- **P0-1:** ~~ONNX runtime crate wired in~~ **DONE via sidecar (see docs/UI-P0-NOTES.md — honest deviation):** the jepa path proved the Python-sidecar architecture (media-extract.py + onnxruntime 1-thread, bit-exact); the onnx option reuses it. Model loader + `models/` folder convention; embeddings verified byte-identical across two runs for BOTH jepa and onnx (new tests in JEPA-TESTING/UI-P0). JEPA 2 backbone exported (tools/export_vjepa2_onnx.py, cosine 1.0 vs torch). Fallback chain recorded in the plan; no homebrew training.
- **P0-2:** `create --encoder handcrafted|onnx|jepa` flag DONE — manifest records encoder identity + model hash, verify reports runtime model integrity (trusted sha check; load-time check deliberately deferred — 1.3GB hash per load too costly), mismatch = honest error.
- **P0-3:** expose path DONE — consumes embeddings for image, camera, and (jepa) video frames at ≤2 fps; downstream binding unchanged; end-to-end verified for all three encoders; suite stays 100% green (116/116).
- ACCEPTANCE: **PASSED** — same image exposed twice under onnx AND under jepa → identical embedding hash each; a file created `handcrafted` rejects an onnx/jepa model and vice versa (manifest-bound routing); no model present → clean error message.

### Encoder heritage (reproduction — ANSWERED 2026-08-05, verified)
- The gamete carries the encoder like the priors: the ovum provides the machinery, the child is born from the egg → **the child always gets the MOTHER's eyes** (jepa mother × handcrafted father → jepa child; handcrafted mother × jepa father → handcrafted child; verified in docs/JEPA-TESTING.md J14). Jepa children are also born with the vision channel attached (net_birth).
- Remaining open question (deferred experiment): does the *maternal line* of eyes matter behaviorally across generations (grandchildren etc.)? Observation only — nothing is coded for it.

---

> Hand this file to the working model at session start. It is the contract.
> The engine (brain-core + CLI) is REAL and VERIFIED: 110 tests pass, behavior confirmed.
> The UI is a shell. This plan builds the real app on top, milestone by milestone.
> Do NOT rebuild the engine. Do NOT rewrite working Rust. Extend, don't replace.

---

## 0. RULES OF WORK (non-negotiable)

1. **One milestone at a time.** Finish, verify, commit, write notes, THEN move on.
2. **"Done" means verified.** Every milestone has an ACCEPTANCE section. Run it, paste real output into the milestone notes. Never claim a feature without running it.
3. **Notes per milestone:** `docs/UI-MX-NOTES.md` — what was built, acceptance output, known gaps, honest failures. No gap is shameful; hidden gaps are.
4. **Tests stay green.** `cargo test --workspace` = 110+ passing before AND after every Rust change. Add new tests for new Rust.
5. **Git commit per milestone** with a descriptive message in the existing style.
6. **No fake UI.** A button that shows a panel but does nothing is forbidden. Every control must hit the real engine or a real browser API. If something is deferred, label it visibly `[deferred]`.
7. **Honest stubs only.** If a feature must be stubbed, comment it `// HONEST STUB:` with a note, and list it in the milestone notes.
8. **Do not break determinism.** brain-core replay must stay bit-identical. Any Rust change touching brain-core gets the determinism tests re-run.
9. **THE 310-TICK RULE:** any NEW ingest path (any code that adds percepts/events and saves) MUST run `run_ticks(310)` before save, or percepts die at exit. This bug happened TWICE already (voice hear; body touch/motion/interocept). Check every new ingest path against it.
10. **Exe discipline:** after ANY Rust change, rebuild and recopy the exe to the repo ROOT (`<repo-root>\neuroform.exe`). The app serves from the repo root exe.
11. **Environment traps (this machine):**
    - Shell is git-bash. POSIX syntax only. No PowerShell.
    - The exe CANNOT write to `/tmp/...` — use repo-relative paths.
    - Toolchain is pinned (rustup override in repo, mingw at `<tools-dir>\winlibs-mingw64`). Do not touch it.
    - Target dir is `<tools-dir>\neuroform-target` (.cargo/config.toml). Leave it.
12. **Compute respect:** no retraining, no heavy model downloads, no giant asset packs. edge-tts + ffmpeg already installed; use them.

---

## 1. ARCHITECTURE (what exists, what gets added)

EXISTS:
- `packages/brain-core` — the organism (memory, modulators, embodiment, voice plan, drawing motifs, physics, network, reproduction).
- `apps/cli` — CLI + `neuroform serve` (localhost HTTP bridge that spawns the CLI per request).
- `tools/desktop/index.html` — 363-line shell, 7 tabs, live dashboard + face. KEEP the dashboard/face logic; grow the tabs around it.

TARGET ARCHITECTURE:
- Same serve bridge, extended with a small set of new endpoints (below). Keep it localhost-only.
- **LLM attach:** the engine already supports attach-by-name (mock) or an OpenAI-compatible endpoint URL (`attach` in main.rs). Build Phase L (below) FIRST — writing/drawing/voice conversation all consume it. Credentials (API key) stored in a local `llm.json` sidecar (never inside the .brain file, never logged), endpoint + model + temperature + max tokens configurable from the UI. A connection test button that makes one tiny real call and shows latency + honest error on failure. NEVER fake an LLM response; no endpoint attached → honest empty state everywhere.
- UI stays **zero-build vanilla JS**, but split into files: `tools/desktop/app.js` (shell), `tools/desktop/tabs/writing.js`, `tabs/drawing.js`, `tabs/voice.js`, `tabs/brain.js`, `tabs/body.js`, `tabs/network.js`, `tabs/repro.js`, `tools/desktop/lib/` (editor, canvas engine, widgets). Serve these from the exe (static file serving from tools/desktop). No bundler, no npm if avoidable; if a library is truly needed (see W1), vendor the file locally — no CDN at runtime.
- New engine endpoints needed (add as milestones require): `/api/state` (JSON without spawning), `/api/file/read`, `/api/file/write` (for editor autosave of sidecar doc metadata), everything else stays `argv` via `/api/run`.

GLOBAL UI REQUIREMENTS:
- Dark theme consistent with current palette. Everything animates subtly (no static dead panels): bars ease, values tween, face breathes.
- Keyboard shortcuts in editor and canvas (listed per phase).
- Every tab shows live brain state where relevant (it's one organism, not six apps).
- **TWO NAMED CURSORS (user mandate 2026-08-05):** the Writing editor and Drawing canvas each carry TWO cursors — the user's and the brain's — visually distinct and NAME-LABELED (user cursor = "you", brain cursor = its file name), so it is always visible whose hand is where. The brain cursor moves to the file's last action position (last written block in the doc, last stroke endpoint on the canvas; surfaced via the state snapshot). This is a requirement of W1 and D1, not an afterthought.
- **GLOBAL SPEECH (user mandate 2026-08-05):** voice output is NOT tab-bound. When the file speaks — prompted OR autonomously (initiative system, default OFF, audited) — the audio plays and the face animates regardless of the active tab; the player lives in the shell (app.js), not the Voice tab. Unprompted speech is an observation experiment: the app surfaces initiatives honestly (activity log + global audio) when they fire; nothing scripts them.

---

## PHASE L — LLM ATTACH & ENDPOINT MANAGEMENT (build FIRST)

The engine already supports LLM attachment; this phase gives it a real home in the UI and a clean contract every organ uses. Nothing in W/D/V may call an LLM except through this layer.

### L1 — Endpoint Manager
- LLM panel (lives in Brain tab): endpoint URL, model name, API key (password field, stored in local `llm.json` sidecar — never in the .brain, never logged, never echoed to the activity log), temperature, max tokens, optional system preamble.
- Multiple named profiles (e.g. "local llama", "openai", "cheap model") with one active at a time; switch without restart.
- ACCEPTANCE: save a profile, reload app, profile persists; key never appears in plaintext in any log or file dump of the brain.

### L2 — Connection Test & Health
- "Test" button: one tiny real completion call. Show success + latency, or the honest error (auth, network, model name) verbatim. Status pill on every tab that uses LLM (attached / detached / error).
- Graceful offline: if the endpoint is down mid-feature, the organ shows honest failure, never a fabricated answer.
- ACCEPTANCE: point at a valid endpoint → green + latency; point at a bad key/URL → the real error text, no fake success.

### L3 — The Shared LLM Bridge
- One code path: `llm_call(prompt, context, budget)` used by writing/drawing/voice. It automatically injects the brain's current state per master spec §8 — relevant retrieved memories, affect, embodiment, attention — and applies the brain's permissions, so every organ's LLM use is modulated by the organism, not raw.
- Budget + timeout controls; every call logged to the activity log (tokens in/out, no secrets).
- ACCEPTANCE: two different organs produce LLM output; both show brain-state modulation (change affect → output tone shifts); a call appears in the log.

### L4 — Writing Integration (the main use)
Wire the bridge into the Writing tab as assist actions on the current selection/doc:
- **Continue** from cursor, **Rewrite selection** (with tone intent), **Summarize doc**, **Expand scene**, **Critique** (continuity + style), **Generate lorebook entry draft** from a passage, **Name suggestions**.
- Every result is a *suggestion diff* — user accepts/rejects per change; nothing auto-overwrites prose. Rate-limit + undo.
- Style guard: assistance is modulated by the doc's style fingerprint so it drifts toward the author's voice, not the model's.
- ACCEPTANCE: each action returns real streamed output into a reviewable diff; reject restores exact prior text; no endpoint → honest "no LLM attached" state.

### L5 — Drawing Integration (reference aid ONLY)
Per master spec: the Drawing tab must NOT become a flat image generator. LLM/image help is planning/reference only; the artifact stays editable strokes.
- **Composition suggestion:** given canvas + intent, return editable guide overlays (rule-of-thirds, focal point, horizon) as a toggleable non-destructive layer.
- **Reference aid:** if an image model endpoint is configured, generate a reference image into the reference board only (never merged into paint layers), clearly labeled "reference, not yours".
- **Motif prompt:** turn current motifs into a text prompt the user can copy.
- No image-to-strokes flattening into the art layer.
- ACCEPTANCE: composition guides render as a separate toggleable overlay; reference appears only in the reference board; no generated pixels land in editable layers.

### L6 — Voice Conversation Wiring
- V4's conversation mode routes replies through this bridge (modulated per §8), then through the voice plan. If no endpoint → honest fallback (the brain's own initiative/reflective text), still spoken.
- ACCEPTANCE: with endpoint, conversation replies are LLM-sourced and state-modulated; without, honest fallback — both spoken.

---

## PHASE W — WRITING TAB (target grade: NovelAI / NovelCrafter)

The writing organ's backend exists (docs bind to memory, src=writing, style fingerprint, continuity conflicts). Build the workspace.

### W1 — The Editor Core
- Real text editor. RECOMMENDED: vendor **CodeMirror 6** (local files, no build needed via prebuilt ESM bundle) OR write a solid `contenteditable` layer — decision recorded in notes with a working spike proving undo/redo + IME + selection survive.
- Prose mode (no markdown rendering) + markdown mode toggle.
- Caret memory per document, smooth scroll, typewriter scrolling option, focus mode (dim all but current paragraph).
- Word/char counts live; session stats; writing goal bar.
- Shortcuts: bold/italic, headings in md mode, find/replace (Ctrl+F), undo/redo.
- ACCEPTANCE: type 2000+ words smoothly; paste rich text; undo 50 steps; IME input works; find/replace works across doc.

### W2 — Library & Persistence
- Sidebar file tree: **Story → {Chapters → Scenes}, Notes, Journal**. Create/rename/reorder by drag.
- Storage: sidecar folder `writing/<brain>/` with one JSON per doc + index.json; saved through serve file endpoints. Every doc write also binds into the brain (`doc write` already exists) — autosave with debounce + visible "bound to memory" pulse.
- Tabs: multiple open docs, switch without losing caret.
- ACCEPTANCE: create 3 chapters × 3 scenes + 2 notes; close app; reopen; tree, order, contents, carets intact; brain shows src=writing traces.

### W3 — Lorebooks & Entity Sheets
- Lorebook entries: title, keywords/triggers, body, enabled toggle, tags. Insertion preview: given current paragraph, show which entries would fire (keyword match).
- Entity sheets (characters/places/items): free-form fields, portrait slot (can point at a drawing canvas export), relationship lines to other entities (simple list first, map view deferred).
- ACCEPTANCE: 5 lorebook entries fire correctly on matching text; entity sheet CRUD persists.

### W4 — Structure Tools
- Outline view (beat sheet): cards per scene, drag-reorder reorders the tree; each card shows synopsis + word count + status (draft/revised/final).
- Timeline view: scenes placed on a horizontal timeline (scene has optional in-story date/order field); zoomable.
- ACCEPTANCE: reorder beats → tree updates; timeline renders and persists.

### W5 — Version History
- Snapshot per document: manual "save version" + automatic snapshot every N words (configurable).
- Version list with word count + time; view any version read-only; restore; diff view (simple word-level highlight diff is enough).
- ACCEPTANCE: 5 versions of one doc; restore an old one; diff shows changed spans.

### W6 — Analysis & Continuity
- Style analysis panel: sentence length histogram, repeated-word cloud, adverb density, dialogue ratio, readability score.
- Continuity: engine already detects property conflicts (`writing.rs`) — surface it: entity properties list, conflict warnings inline.
- ACCEPTANCE: write a deliberately repetitive passage → analysis flags it; contradict an entity property → warning appears.

### W7 — Export / Import
- Export: .md, .txt, single-file HTML (styled), full project JSON. Import: .md/.txt → new doc; project JSON restore.
- ACCEPTANCE: round-trip a project through JSON; export md opens clean.

### W8 — Brain Integration Panel
- "What the brain remembers" side panel for the open doc: retrieved traces (cue = doc text), style fingerprint radar vs previous docs, semantic nodes extracted from this doc.
- Writing mode selector that modulates binding salience (journal vs prose vs worldbuilding).
- ACCEPTANCE: panel shows live retrieval for open doc; different modes produce different binding (visible in brain inspect).

### W9 — LLM Assistance
MOVED to Phase L (L3/L4). Do not build LLM anything here; consume the shared bridge. If L is not built yet, this milestone is: honest "no LLM attached" states only.

---

## PHASE D — DRAWING TAB (target grade: Paint Tool SAI class)

Backend exists: strokes bind to motifs/shape memory. Build the painter.

### D1 — Canvas Engine & Stroke
- Viewport: zoom (wheel, 1%–3200%), pan (space/middle-drag), fit, 100%. Crisp pixel handling at high zoom.
- Pointer Events with **pressure** (pen-aware; mouse = constant pressure).
- Stroke pipeline: raw points → **stabilizer** (delayed-pull like SAI's S levels 0–7) → smoothed path → stamped brush dabs → layer canvas.
- Undo/redo from day one (command pattern, ≥30 steps).
- ACCEPTANCE: draw with tablet pressure visible in width; stabilizer smooths a shaky line; zoom 800% shows crisp dabs; undo restores.

### D2 — Layers
- Layer stack UI: add/delete/duplicate/reorder (drag), visibility, lock, opacity slider, blend modes: normal, multiply, screen, overlay, luminosity, add(glow) — composite via offscreen canvases.
- Layer groups (folder) with pass-through + group opacity.
- Merge down / flatten visible.
- ACCEPTANCE: 8 layers + 2 groups render correctly in all listed blend modes; reorder live.

### D3 — Brush Engine
- Brush parameters: size, min-size (pressure), opacity, flow (buildup), hardness/soft edge, spacing; eraser modes (hard/soft, affect alpha only).
- Brush presets panel: ≥10 built-in presets (pen, hard round, soft airbrush, marker, watercolor-ish blend, eraser variants…) + save/load custom presets.
- Stabilizer control in toolbar.
- ACCEPTANCE: 10 presets visibly distinct; custom preset persists across reload; flow builds up on held stroke.

### D4 — Color
- HSV wheel + SV square picker, hex input, sliders; swatches/palettes (save/load, default palette); eyedropper (reads composite); foreground/background swap, X shortcut.
- ACCEPTANCE: pick, swatch CRUD, eyedrop matches composite pixel.

### D5 — Selections & Transform
- Tools: rect, ellipse, lasso, magic wand (tolerance slider, contiguous option), select-all, deselect, invert; feather; move-selection content.
- Transform on selected layer/selection: move, scale, rotate, flip H/V, with handles + Enter/Escape.
- ACCEPTANCE: wand selects a flat color region with tolerance; transform with rotation commits cleanly; undo reverses.

### D6 — Fill, Gradient, Masks, Clipping
- Bucket fill (tolerance + contiguous), linear/radial gradient tool, gradient map adjustment over a layer.
- Layer masks (paint black/white), clipping layers to the layer below (SAI behavior).
- ACCEPTANCE: masked layer erases non-destructively; clipped layer shows only where base layer has pixels.

### D7 — Assist Tools
- Symmetry drawing (vertical/horizontal, toggle), straight-line + shape tools (line/ellipse/rect with outline or fill), perspective guide overlay (1/2-point, non-destructive rulers), reference board (side panel, load an image, opacity slider).
- ACCEPTANCE: symmetric stroke mirrors live; reference image sits beside canvas.

### D8 — History, Autosave, Export
- Canvas document format: `drawing/<brain>/<name>.ndraw.json` (layers as PNG data + meta + motif bindings). Autosave.
- Export PNG (composite or single layer), import PNG as new layer. New canvas dialog (W×H presets + custom).
- ACCEPTANCE: kill app mid-painting → reopen, strokes intact; export matches screen.

### D9 — Brain Integration
- Every committed stroke set binds to the brain (existing `draw stroke` path + 310-tick rule); Motif Memory panel: shape families the file learned, most-used colors/sizes; "aesthetic memory" readout from brain inspect.
- ACCEPTANCE: draw → motifs panel updates from real brain state.

---

## PHASE V — VOICE / MOUTH TAB (make it an organ, not a dropdown)

Backend exists: voice plan (pitch/prosody/affect-sensitive), maturity, mimicry gate, heard voices, TTS via edge-tts.

### V1 — Speak & Prosody View
- Speak panel: text → engine voice plan → rendered audio (edge-tts with rate/pitch mapped from the plan: pitch→pitch, arousal→rate, fatigue→rate-down + breathier voice choice). Audio plays inline with waveform.
- **Prosody contour view:** draw the plan's pitch/pause structure over the sentence (from the plan data, not guessed).
- Breath view: breath-group chunking with pause markers.
- Voice history log (last N utterances with affect at time of speaking).
- ACCEPTANCE: speak 3 lines with different induced affect (feed events first) → visibly different contours + audibly different delivery; log shows them.

### V2 — The Face, Upgraded
- Keep current reactive face as base; upgrade to full parametric model: brows (valence+dominance), eyelids (arousal/fatigue), mouth shape set (visemes), gaze (follows cursor by default), micro-expressions (occasional, seeded — deterministic), breathing idle (chest/shoulder sway or scale oscillation), blink dynamics with arousal-linked rate.
- **Lip-sync:** drive mouth openness from the playing TTS (word-boundary timings from edge-tts SubMaker if available; else amplitude envelope of the audio). Face speaks while audio plays.
- Emotion blending: affect vector (valence/arousal/dominance/warmth) → blended expression, smooth tweens, no snaps.
- ACCEPTANCE: feed positive/negative events → face shifts expression with easing; speak a line → mouth moves in rough sync; eyes follow cursor; blinks vary with arousal.

### V3 — Vocal Tract & Development
- 2D sagittal vocal-tract diagram: tongue position, lip rounding, jaw from articulation params of the voice plan (schematic but live-bound).
- Voice development timeline: pitch/maturity/mimicry history chart (engine keeps voice memory — expose it).
- Voice identity panel: current params + drift history; override controls (existing override + audit); mute.
- ACCEPTANCE: diagram moves with different phoneme-ish plans; timeline shows real history from the file.

### V4 — Conversation Mode
- Chat panel: talk WITH the brain. Each user line is ingested (hear path + 310 rule), response via attached LLM if present (modulated per §8), else the brain's own initiative/reflective text; every reply goes through the voice plan → spoken + face animates. Snappy: stream text, start audio fast.
- ACCEPTANCE: 5-turn conversation; each turn shows ingest trace in brain + spoken reply + face animation; no LLM → honest fallback text, still spoken.

---

## PHASE B — BRAIN TAB (create, inspect, couple, birth)

### B1 — Creation Wizard
- New Brain dialog: name, **tier picker with real capacity numbers** (prototype/standard/advanced/experimental — show episodic slots, latent dim, file cap), **embodiment preset** (female / male / custom / mixed) → shows resulting karyotype + priors preview (hormone axes as bars, clearly "priors not destiny"), seed input + reroll, passphrase optional (with plain-dev warning shown), create.
- Brain gallery: list `.brain` files in folder with tier/seed/karyotype/age-in-ticks at a glance.
- ACCEPTANCE: create one of each tier + male + female; gallery lists them with correct metadata.

### B2 — Inspection Depth
- Memory inspector: browse episodic traces (filter by source/modality/time), search by cue, inspect a trace's full binding; semantic graph view (nodes/edges, top-N by belief); procedural list.
- Dream log viewer (from sleep cycles); sleep controls with stage progress (wind-down → light → deep → dream) and sleep-pressure gauge.
- Modulator history: engine needs a small ring buffer (add to brain-core, deterministic, persisted shard or STATE) → time-series charts for 8 modulators + affect.
- Bias audit panel: surface existing `audit.rs` output with suggested actions.
- ACCEPTANCE: every panel shows real file data for m1.brain; history chart grows over ticks.

### B3 — Coupling & Birth Wizard (the engine ALREADY does this — build the ceremony)
- Two-pane pairing: pick file A and file B (or open a second serve port for true two-instance). Guided flow: keys → pair → establish session → **propose union** (shows pheromone/hormone profile being sent) → peer chemistry response (complementarity score shown honestly: attracted or not) → accept → **gamete view** (ovum X from mother; sperm X/Y revealed at birth — child's sex is chance) → birth → child card (tier, karyotype, lineage, backup flag) → growth monitor with age-gated tier progress toward inherited ceiling.
- **Lineage tree:** multi-generation family tree view (parents, children, karyotypes, tiers), rendered from lineage records.
- Kin-recognition readout: shared chemistry between child and parents (the real 0/+ signal).
- ACCEPTANCE: run the full ceremony twice producing ≥2 generations; tree renders grandparents→parents→children; two-ova pairing correctly refuses (shown honestly).

### B4 — Life Controls
- Day-cycle runner: run N days with visible sleep/consolidation passes; autonomy toggle with initiative feed; camera/mic exposure buttons (existing expose --camera) with consent dialog each time and visible "sensor live" indicator.
- ACCEPTANCE: run 30 days; watch sleep pressure cycle and dream log grow; camera exposure binds a src=visual-exposure trace.

---

## PHASE S — BODY & NETWORK TABS (polish, smaller)

### S1 — Body Tab
- Real browser sensors where hardware allows: pointer/pressure → touch channel (live touch-field heatmap), DeviceMotion/orientation if available → vestibular, mic level + camera thumbnail (with per-session consent) → auditory/visual exposure. Each ingest obeys the 310-tick rule.
- Body schema panel: channel list with reliability + calibration confidence, calibration flow (hold still / draw a circle), body ownership confidence from the file.
- ACCEPTANCE: painting in Drawing tab produces touch percepts visible in Body tab; status reflects calibration.

### S2 — Network Tab
- Session console between two files: message log with emotional tone per message, bond/familiarity/trust bars live, teaching packets, shared canvas mode (strokes from file B appear on file A's canvas via inject) and shared doc mode.
- ACCEPTANCE: two files exchange 10 messages → social memory visible on both; one draws, other's canvas receives.

---

## PHASE E — EYES: VIDEO & LIVE-FEED LEARNING (LeCun-grade perception, unlabeled)

The engine ALREADY has the perception entry path: `expose --camera` (ffmpeg dshow → 16-dim features → unlabeled binding, src=visual-exposure) and the autonomy/organ-control machinery (`life --autonomy`, initiatives, teacher). This phase extends that exact pathway to time — videos and live feeds. Core rule from the user: the brain learns from WATCHING, not from text labels. Features ARE the memory; raw media is never stored (per master spec §5 Vision).

### E1 — Live Camera Feed Mode
- Persistent watching mode: while enabled, sample the webcam every N seconds (configurable), extract features, bind through the existing exposure path (310-tick rule applies — run the bind window before save).
- Consent gate: visible "EYES LIVE" indicator with pulse while sampling; one-click mute; auto-stop when the app loses focus (configurable).
- ACCEPTANCE: enable feed for 2 minutes → N percepts bound, retrievable with src=visual-exposure, cortex visual region active; disable → sampling stops verifiably.

### E2 — Video Watching
- Watch panel: user picks a video file → ffmpeg extracts frames at a fixed interval (e.g. 1–2 fps, configurable) → features per frame → sequential unlabeled exposure into the brain.
- Temporal features: extend the feature extractor with frame-to-frame difference/motion signals so the predictive world model and physics module get temporal structure (object motion, appearance/disappearance → permanence violations → surprise). No labels anywhere.
- Watch log: sessions recorded (file name, duration, frames exposed, surprise peaks, cortex activity) — inspectable in Brain tab.
- Budget guard: max frames per session + sleep pressure rises with watching (sensory load, per interoception spec).
- ACCEPTANCE: watch a 1-minute clip → features bound in order; a clip with sudden appearances produces measurable surprise peaks vs a static clip; watch log shows both.

### E3 — Attention During Watching
- The brain does not drink every frame equally: salience-gated exposure — frames whose feature novelty/change exceeds the current attention threshold bind strongly; boring frames bind weakly or skip. Modulated by curiosity, arousal, and fatigue (tired brain watches worse).
- ACCEPTANCE: same clip watched at high vs low arousal → measurably different bound-trace counts/salience; numbers recorded.

### E4 — Consolidation of the Watched
- Sleep replays watch sessions: replay of bound visual percepts during consolidation, gist extraction into semantic memory (unlabeled clusters — the file groups what it saw without being told what things are), dream content influenced by watched material.
- ACCEPTANCE: watch a clip → sleep → dream log or semantic clusters show influence from that session's percepts; verifiable in inspect.

---

## PHASE X — EXPLORATION: REFERENCE IMAGES, INTERNET, LOCAL COMPUTER (permission-gated)

Built on the SAME perception pipeline as Phase E (features → unlabeled binding → consolidation). Governing spec: master prompt §11 — allowlists, blocklists, domain permissions, content safety filters, time budgets, action budgets, user approval, session/exposure/preference logs. The user rule holds: no text learning, no labels. The brain learns from what it SEES. Raw media never enters the .brain file; a display cache sidecar folder may hold reference images for the UI, clearly flagged and wipeable.

### X1 — Reference Image Collection
- Feed the Drawing tab's reference board (D7) from sources: local folders and allowed domains. Each collected image gets features extracted + bound as visual exposure (aesthetic-salience tagged) AND appears on the reference board for the human.
- ACCEPTANCE: collect 10 images → reference board displays them, brain shows 10 bound visual percepts with aesthetic tags; wipe cache → display gone, memory traces remain (features only).

### X2 — Internet Wandering
- Browsing panel: allowlist/blocklist manager, per-session user approval, time + action budget sliders, visible "BROWSING" indicator, content safety filter (configurable), kill switch.
- Mechanism: fetch page → extract images → features → exposure pipeline. Text on pages is NOT learned (user rule) — it may only be used for navigation.
- Preference learning per §11: distinguish dwell, revisit, rejected, ignored → weighted preferences that DECAY and are auditable. Bias audit flags echo-chamber browsing (repetitive domains/styles) and suggests diversification (§13).
- ACCEPTANCE: one approved session on an allowed domain → exposure log shows what was seen; a rejection visibly lowers a preference weight; audit panel lists learned preferences with sources.

### X3 — Local Computer Wandering
- The user grants SPECIFIC folders (never whole-disk by default). The brain wanders them: finds images, watches them through the feature pipeline, binds percepts, logs what it saw where.
- Permission manifest persisted in the file (master spec §15); every grant is revocable; full exposure log reviewable + selectively deletable.
- ACCEPTANCE: grant one folder → wander → exposure log lists files seen + bound trace count; revoke → no further access; delete a memory set → traces gone.

### X4 — Exploration Memory & Audit
- Exploration history view: timeline of watching/browsing/wandering sessions with what shifted in the brain (new semantic clusters, preference changes, surprise peaks).
- ACCEPTANCE: history renders real session data and links each session to measurable brain changes.

---

## PARITY GAPS — what separates "features exist" from NovelAI/SAI grade

Feature coverage is not parity. These three areas are mandatory before any "it's done" claim, and each has its own acceptance bar:

### G1 — Performance at real scale
- Writing: a 200k+ word novel must open, scroll, and search smoothly. If the editor slows, virtualize rendering (render visible range only), debounce analysis off the main thread, and prove it: open a generated 200k-word doc, measure scroll FPS and keystroke latency, record numbers in notes.
- Drawing: large canvases (≥4000×4000) with many layers must stay interactive. Technique: tile-based layer bitmaps (render visible tiles only), and composite via WebGL if 2D canvas can't hold 60fps during stroke preview. Prove it: 4000×4000 canvas, 12 layers, stroke latency measured and recorded.
- No milestone is done if it only works on a small demo doc/canvas.

### G2 — Feel calibration (the SAI magic)
- Brush feel needs a dedicated tuning pass, not just parameters existing. Compare side-by-side with reference behavior: stabilizer at every level, opacity/flow buildup curves, blend smoothness. Record before/after notes.
- Editor feel: caret movement, selection behavior, IME, paste-of-rich-content must feel native, not webby. Test with actual long-form typing sessions and record issues found.
- This pass happens AFTER features exist (after D9 / W8), as its own dedicated effort.

### G3 — Long-tail niceties (pros notice these)
- Drawing: canvas rotation (R key), mirrored/flipped view toggle, nudge selection with arrow keys, color picker from any screen position.
- Writing: series/multi-book shelf, per-book stats, distraction-free fullscreen.
- These are small individually; do them as one sweep milestone before Phase F.

---

## PHASE F — FINAL TESTING & PAPER (user-mandated process, do not skip)

1. **All features complete first.** Nothing left half-done.
2. Write `docs/UI-TESTING.md` BEFORE testing: goals, scope, falsifiable expectations per tab (e.g., "editor undo survives 50 steps", "wand tolerance monotonic", "face expression tracks induced affect", "birth refuses two ova").
3. Run the tests ethology-style — actually exercise every organ, record results, pros/cons, conclusions in the same file.
4. Addendum to `docs/PAPER.md`: what the embodiment of organs (writing/drawing/voice UI bound into the file) changed vs headless CLI, honestly.
5. Final README quickstart + exe recopied to repo root, git tree clean.

---

## ORDER OF WORK

**P0-1 → P0-2 → P0-3** → L1 → L2 → L3 → W1 → W2 → D1 → D2 → W3 → L4 → D3 → D4 → W4 → W5 → D5 → V1 → V2 → D6 → W6 → B1 → W7 → D7 → L5 → V3 → B2 → W8 → D8 → V4 (uses L6) → D9 → B3 → B4 → S1 → S2 → **E1 → E2 → E3 → E4 → X1 → X2 → X3 → X4** → G1 → G2 → G3 → F.

> **⚠ DELAY (2026-08-06):** W3 onwards (and all of D/V/B/S/E/X/G/F) are **deferred** while the
> body build (`VISCERA.md`, §28–§38) proceeds. This contract builds organs/UI; VISCERA builds
> the organism's body. Rule of the two contracts: a milestone in either may not break the
> other. When the body build reaches a point where an organ's UI is the right next step,
> resume this list from where it left off (W3 next).

(Interleaved deliberately: LLM bridge first so organs consume it as they're born; writing and drawing momentum alternate so neither stalls; face early because it's cheap morale; coupling late because it's pure UI over existing engine. The G-passes come after all features exist and before final testing — that's where "features exist" becomes "feels like the real thing.")

## SIZE HONESTY (read once, then work)

- Writing phase ≈ NovelAI-class editor is large. The plan breaks it so each milestone is individually verifiable. Do not attempt W1–W9 in one session.
- Drawing phase: D1–D3 are the hard spine; get them excellent before anything else.
- If a milestone is too big, split it and SAY SO in notes — never silently shrink scope.
- The engine is the soul; the tabs are the body. Build the body so the soul is visible.

BEGIN WITH W1.
