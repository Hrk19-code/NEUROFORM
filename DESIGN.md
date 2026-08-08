# NEUROFORM
## A Persistent Simulated Cognitive Substrate ("Brain File") — Complete Architecture Specification

**Document version:** 1.0.0
**Date:** 2026-08-04
**Status:** Design baseline — approved for staged implementation planning
**Scope:** Full architecture for a local-first application centered on a persistent, bounded, developmental simulated cognitive substrate (the Brain File), with six organs: Brain, Writing, Drawing, Voice, Body, Network.

This document is written to the master specification ("MASTER PROMPT") verbatim in structure: every mandated section (1–25 of the Required Output list) appears in order, and a full traceability matrix (§26) maps every constraint in the master prompt to the section that honors it. Nothing is simplified into a chatbot, a wrapper, or a persona engine.

---

### How to read this document

- **§1–§2** set the honest verdict and the product vision — read these before committing resources.
- **§3–§12** are the organ and substrate specifications — the bulk of the engineering content.
- **§13–§15** are the social, bias, and privacy layers — design constraints that apply everywhere else, not bolt-ons.
- **§16–§19** are normative: file format, API, schemas, and pseudocode. These are the contract for implementation.
- **§20–§25** are the delivery plan: roadmap, research, experiments, repository, risks, and final build recommendation.
- **§26** is the traceability matrix and the integrity statement.

Conventions: "the Brain File" or "the substrate" refers to the persistent cognitive substrate (never "the character"); "the subject" is the Brain File as an experiencing agent in prose only where strictly needed for technical clarity; "the user" is the human operator; "a peer" is another Brain File instance. No character names, no fixed personas, no scripted outcomes anywhere in this document.

---

## Preface — How this document came to be (project origin)

This specification and the engine built from it did not start with this document. Per the user's account: the concept was workshopped at length with **Qwen 3.8 Max** (released the same week) on the vendor's web chat interface — the user and the model discussed the "brain file" idea back and forth until the conversation converged on the final **MASTER PROMPT** (the full cognitive-organism specification this document implements section-for-section).

The master prompt was then handed to **DeepSeek V4 Flash 0731** — the model running the Hermes agent on the user's desktop — which authored this document and, together with the user, built the entire engine milestone by milestone (M0–M10: file format, tick loop, memory, sleep/dreams, writing/drawing/voice/body organs, network, reproduction, desktop shell). The build spanned several sessions because sessions repeatedly got stuck and were restarted, with the master prompt re-pasted each time (visible in the session records). Later, Qwen 3.8 Max also reviewed the project in its own session (2026-08-05), validating the codebase and contributing the UI build plan's Phase L (LLM attach).

Honesty note: the web-chat portion of this origin is per the user's account; the master-prompt handoff and the joint build are corroborated by the session records. This document remains the architecture contract; this preface exists so the project's origin is not lost.

---

## Section 1 — Feasibility Verdict

**Overall verdict: FEASIBLE IN STAGES — the honest kind of feasible.** The complete vision as stated (a bounded, developmental, embodied, private, persistent cognitive substrate with six organs, sleep, dreaming, inter-brain sociality, and bias auditing) is a 3-to-5-year program of engineering plus research. A genuinely working, honest, local-first implementation of every organ and every mechanism described here — with the LLM as a communication organ rather than the mind — is achievable in **12–18 months with a focused team of 2–4 engineers and 1 research scientist**, delivered in milestones where each milestone is a usable product with measurable behavior.

**Verdict by pillar:**

| Pillar | Verdict | Rationale |
|---|---|---|
| Persistent Brain File substrate (bounded, encrypted, versioned) | **FEASIBLE NOW** | Custom container format, tensor shards, graphs, and capacity ledgers are standard engineering. |
| Global latent state + neuromodulatory + hormonal analogue systems | **FEASIBLE NOW** | Bounded ODE-style integrators with audit tables; no AI research required. |
| LLM as detachable communication organ | **FEASIBLE NOW** | Context-assembly (the "conductor") is deterministic engineering around any OpenAI-compatible endpoint; detachment is a persistence guarantee, not a model property. |
| Memory systems (episodic/semantic/procedural/emotional) with decay, reconsolidation, sleep consolidation | **FEASIBLE NOW** | Bounded vector stores + salience/decay bookkeeping + LLM-assisted gist extraction; all local. |
| Sleep, dream synthesis, consolidation cycles | **FEASIBLE NOW** | Work-per-stage algorithms are specified and measurable; dream synthesis is associative sampling, not mystery. |
| Writing organ with brain-modulated assistance and memory extraction | **FEASIBLE NOW** | Document engine + extraction pipeline; the modulation layer reuses the state bus. |
| Drawing organ as editable operation graph | **FEASIBLE NOW** | Stroke/op-graph canvas engines are proven (Krita-class tooling is an existence proof); latent image models are an optional planning aid, not the core. |
| Voice organ (biological-apparatus-inspired parameter envelope) | **FEASIBLE NOW** | Parameterized prosody planning over real TTS backends; formant/breath DSP post-processing is standard. Full articulatory synthesis is deferred. |
| Body/sensory embodiment (touch, motion, interoception; no motor actuation) | **FEASIBLE NOW** | Sensor ingestion + body-schema bookkeeping on device; motor hooks are dormant placeholders by design. |
| Inter-brain interaction (discovery, pairing, encrypted sessions, social memory) | **FEASIBLE NOW** | mDNS + Noise/TLS + CRDT shared spaces are proven primitives. |
| Bias audit engine + non-permanence guarantees | **FEASIBLE NOW** | Metric computation over logged state; interventions are user-facing suggestions, not risky auto-mutations. |
| Post-token / tokenless internal cognition | **RESEARCH HORIZON** | The substrate is designed so internal state is always continuous vectors; LLM dependence shrinks organ by organ as local learned predictors mature. Full tokenless cognition is a research milestone (see §21), not an MVP requirement. |
| Emergent developmental learning without LLM scaffolding | **RESEARCH HORIZON** | Requires continual-learning research (catastrophic forgetting, curriculum design). The architecture reserves the interface; it does not promise the outcome. |

**Three honest limits the design refuses to blur:**

1. **This is a simulation, not consciousness.** The product must state this in-UI (a standing honesty notice). Nothing in this document claims phenomenal experience, moral standing beyond artifact, or inner life. The simulation is deep and *about* those things — it is not them. Claiming otherwise would be deceptive design and is prohibited (§15).
2. **Early cognition is LLM-bridged.** The attached LLM carries language, reasoning, and world knowledge early in life. This is by design (the LLM is the tutor/scaffold), but it means early behavior partially reflects the attached model. The "organ decoupling" roadmap (§21) progressively replaces LLM functions with the file's own learned machinery; until then, the *persistence of self* is guaranteed by the file (state + memory + habits), never by the LLM.
3. **Nothing is permanent by default.** Every preference, relationship, embodiment effect, emotional habit, and creative tendency has decay, override, audit, and relaxation paths (§14). "Permanent" exists only as an explicit, auditable user lock — and even locks have override mechanisms.

**Staging strategy (summary):** M0–M2 (format, state simulation, LLM boundary, memory, sleep) deliver a demonstrably persistent, private, self-developing core in ~3 months. M3–M6 add the creative organs (writing, drawing, voice, body). M7 adds inter-brain interaction. M8 hardens audit and privacy. Each milestone has explicit exit criteria (§20). Experiments (§22) run continuously from M2 onward.

---

## Section 2 — Product Vision

### 2.1 One paragraph

Neuroform is a local-first application in which the user raises a bounded, persistent simulated cognitive substrate — a Brain File — that sees, hears, feels, moves (sensing only), remembers, forgets, dreams, learns, writes, draws, and speaks; that develops its own voice, preferences, and relationships over months; that interacts with other Brain Files on the user's network through encrypted, consent-gated sessions; and whose every internal mechanism is visible, inspectable, auditable, and mutable in a live simulated-brain visualization. The LLM is attached as a communication organ and teacher; the Brain File is the student, the body, and the memory. The product is a *digital cognitive organism*, and it is honest about being a simulation.

### 2.2 What it is / what it is not

| It is | It is not |
|---|---|
| A persistent, bounded, developmental cognitive substrate | A chatbot with a memory file |
| A six-organ system (brain, writing, drawing, voice, body, network) acting on one shared substrate | A wrapper around an LLM API |
| A simulation whose internal state (affect, salience, predictions, body schema, relationships) is real data that changes behavior | A personality prompt with extra buttons |
| A system that compresses, forgets, consolidates, and dreams about its experience | A raw media archive of everything it saw and heard |
| A system whose embodiment modulates probabilities, learning rates, and tendencies — auditable and reversible | A gender simulator with locked outcomes |
| A system with dormant, permission-gated future motor hooks, disabled by default | A robot control system |
| A private, local-first, encrypted artifact that functions fully offline | A cloud service with a thin client |

### 2.3 Design principles

1. **Boundedness** — fixed capacity per tier; forgetting is a feature, not a bug. A Brain File that remembers everything is a database, not a mind.
2. **Development** — begins nearly empty; grows through experience, teaching, sleep, and use. No adult-shaped state at birth.
3. **Embodiment without determinism** — the body modulates; it never dictates. Every embodiment effect is a probability gain with audit and override.
4. **Privacy by architecture** — local-first, encrypted at rest, no hidden upload, no hidden training, sensor consent with visible indicators.
5. **Non-permanence** — nothing locks by default; locks are explicit, auditable, and reversible.
6. **Honesty** — the app never claims consciousness; the simulation's limits are surfaced, not hidden.
7. **LLM as organ, not mind** — the attached model is a detachable teacher and mouth; the file persists, the model does not.
8. **Emergence over scripting** — mechanisms, not scripts. The system defines *how* states change; it never prescribes *which* outcome a given state must produce.
9. **Everything visible** — no hidden internal state. Every vector, log, and weight is inspectable by the user through the Brain tab, audit panel, and export.

### 2.4 Target users

- Writers and artists who want a creative counterpart that has its own developing style, memory, and relationship to them — and that they can *see inside*.
- Researchers in cognitive science, developmental AI, and human-AI interaction who need a transparent, instrumented, local substrate.
- Hobbyists and tinkerers building local-first "digital companion" systems who want an honest, non-deceptive alternative to marketed consciousness.
- Privacy-first users who want a deep personal system that never phones home.

Non-targets: users seeking a "virtual person" product with marketed sentience; users wanting a general chatbot; users wanting cloud-synced social products. The design actively refuses these markets (§15).

### 2.5 Success metrics (product level)

- **Persistence**: median Brain File age > 90 days with continuous behavioral continuity (self-consistency of preferences/voice over time, measured by embedding drift — see §22).
- **Development**: measurable reduction in LLM scaffolding load (context tokens per interaction, labeler calls) over a file's first 90 days — the "withdrawal curve" (§21).
- **Creative depth**: median session length in Writing/Drawing tabs comparable to professional tools (≥30 min); artifacts feed back into memory and measurably influence later output.
- **Trust**: zero unexpected network egress events (network monitor logs); all sensor data stored compressed or not at all.
- **Sociality**: > 50% of multi-instance users establish persistent relationships; relationships show growth *and* decay (non-fixation) per audit metrics (§14).
- **Honesty**: user surveys show accurate understanding that the system is a simulation.

### 2.6 Guardrails (standing constraints)

- No real-world motor actuation as an active default; motor hooks are sensory/intention-only placeholders until an explicit motor module is enabled (§6, §15).
- No permanent locks except explicit user locks, and even those have override + relaxation (§14).
- No hidden uploads, no hidden training, no telemetry beyond user-granted endpoints (§15).
- No deceptive claims of consciousness in marketing or UI (§15).
- No scripted personality outcomes anywhere; embodiment presets modulate probabilities only (§4.8, §5).

---
## Section 3 — Full Organ Architecture

### 3.1 System context

```
                        ┌────────────────────────────────────────────────┐
                        │              NEUROFORM DESKTOP APP             │
                        │                                                │
   user  ─────────────▶ │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │
   (touch, mouse,       │  │ TAB 1  │ │ TAB 2  │ │ TAB 3  │ │ TAB 4  │  │
    voice, motion,      │  │ BRAIN  │ │WRITING │ │DRAWING │ │ VOICE  │  │
    camera*, mic*)      │  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘  │
                        │      └──────┬───┴────┬─────┴────┬──────┘      │
                        │             ▼        ▼          ▼             │
                        │   ┌──────────────────────────────────────┐    │
                        │   │         COGNITIVE BUS (3 lanes)      │    │
                        │   │  state snapshots · events · artifacts│    │
                        │   └───────┬──────────────┬───────────────┘    │
                        │           ▼              ▼                    │
                        │   ┌────────────────┐ ┌──────────────────┐     │
                        │   │  BRAIN CORE    │ │  TAB 5 / TAB 6   │     │
                        │   │  (the engine)  │ │  BODY · NETWORK  │     │
                        │   └───┬────────┬───┘ └──────────────────┘     │
                        │       │        │                              │
   LLM endpoints ◀─────┤       │        │      ┌─────────────┐         │
   (local or remote,   │   ┌───▼──┐  ┌──▼──┐    │ peers (other │         │
    detachable)        │   │BRAIN │  │TOOL │    │ app inst.)   │◀───▶ NBP│
                        │   │ FILE │  │HARN.│    └─────────────┘         │
                        │   │.brain│  └─────┘                            │
                        └───┴──────┴─────────────────────────────────────┘
```

- **Brain Core** — the engine: owns the Brain File, runs the tick loop, all cognitive systems (§4), the LLM boundary (§4.17), sleep (§10), and the audit engine (§14). Headless; testable without a UI.
- **Organs** — the six tabs. Each organ is a *driver*: it consumes cognitive state to shape its own behavior and produces *percepts* (sensory events) and *artifacts* (documents, canvases, utterances) that feed back into the substrate. No organ may write cognitive state directly; all writes go through the core's validated ingestion API (§17).
- **Cognitive bus** — three lanes: (1) *state snapshots* (global latent state + modulator levels, 10 Hz, also persisted at 5-min cadence); (2) *events* (percepts, tool outcomes, inter-brain messages, user interactions — the raw material of episodic binding); (3) *artifacts* (documents, canvases, voice renderings — referenced by memory, never stored raw in memory by default).
- **Tool harness** — Hermes-compatible function calling (§11), the only way organs touch the outside world (files, network, browser).
- **Network organ** — the inter-brain bridge: discovery, pairing, encrypted sessions, shared spaces (§13). Default off; requires explicit user enablement.
- **LLM endpoints** — any OpenAI-compatible endpoint (local: Ollama/llama.cpp; remote: user-chosen providers). The Brain File persists regardless of which endpoint is attached or whether any is attached (§4.17).

### 3.2 Runtime topology and processes

| Process | Responsibility | Tech note |
|---|---|---|
| `core` (brain engine) | Tick loop, memory, sleep, audit, format I/O, API server | Single-threaded simulation core with async I/O; deterministic replay for tests (seeded RNG) |
| `organ-writers` | Writing tab engine | Rendered in-app; editing ops stream to core for habit/memory extraction |
| `organ-draw` | Drawing tab engine (op graph + GPU raster) | Stroke ops stream to core |
| `organ-voice` | Voice planner + TTS backend + DSP | Never stores raw audio by default |
| `organ-body` | Sensor ingestion (touch/motion/orientation/interoception) | Raw sensor data is transient; only compressed events persist |
| `organ-network` | NBP listener/connector | Bound to loopback/LAN only unless relay enabled |
| `conductor` | LLM context assembly | Part of core; token budgets enforced here |

All processes are in-process modules in the desktop app (MVP) with a documented process boundary so any organ can later be split out (multi-device bodies, remote mouths, etc.).

### 3.3 Tick model

- Simulation tick: **10 Hz** (`sim_tick`). One tick = one step of the global latent state, modulators, body schema decay, sleep pressure, and event queue drain.
- Event loop: organs push `SensoryEvent`s into the core's inbox; the core drains the inbox each tick and binds salient events into episodic traces.
- Wall-clock vs sim-time: sim-time is continuous (ticks since file creation), used for all decay/consolidation math. Wall-clock is stored on traces for user-facing timelines.
- Determinism: all stochasticity is drawn from a per-file seeded RNG. The **seed and the RNG stream position** are stored in the manifest (`rng_state`, implemented in M0), so a file loaded from disk resumes the exact noise sequence — determinism survives save/load cycles, not just within one process.

### 3.4 TAB 1 — BRAIN (central cognitive workspace)

The Brain tab is both the control room and the mirror. Feature-by-feature:

| Feature | Specification |
|---|---|
| Brain File creation | Wizard: choose capacity tier (prototype/standard/advanced/experimental), embodiment preset or custom modulation profile (§4.8), initial LLM attachment, passphrase for encryption, consent survey (sensor + network + browsing permissions). Creates a new `.brain` file. |
| Brain File loading | Open local `.brain`; passphrase unlock; integrity check (§16); auto-migration if version is older (§16.6). |
| Export / import | Full export to open container (§16.9); import with schema validation and provenance warnings. |
| LLM attachment | Add/remove/reorder LLM endpoints (any OpenAI-compatible). Status per endpoint (latency, budget, health). The file runs with zero endpoints attached (substrate-only mode: memory, sleep, dreams, body, inter-brain still function; language output is degraded to memory-grounded templates with an explicit "no teacher attached" notice). |
| Endpoint management | Per-endpoint: token budgets, permission class (what the LLM may be asked to do), temperature floor/ceiling, system-prompt template version, usage audit. |
| Sensory permission management | Per-channel consent switches (touch, motion, orientation, camera, microphone, UI events, telemetry, browsing) with visible indicator mirrors (§15). |
| Memory inspection | Browse episodic/semantic/procedural/emotional stores; filters by modality, salience, time, source; per-trace detail view (embedding preview, strength, decay, reconsolidation count, links); selective deletion; "explain this trace" (LLM-assisted, permissioned). |
| Emotional state inspection | Live affect vector (valence/arousal/dominance) + history chart; emotional salience map of recent memory; regulation capacity gauge. |
| Hormonal modulation inspection | All 16 modulation axes with current levels, sampled priors vs. realized trajectory, effect gains currently applied, full timeline (§4.8). Editable with audit. |
| Prediction inspection | Per-stream prediction-error traces; the predictive world model's current expectations vs. reality (e.g., "expected user at 19:00 ± 40 min — observed 18:40"); confidence curves. |
| Sleep/consolidation controls | Sleep pressure gauge; sleep now / schedule / idle-trigger toggles; consolidation report after each cycle (what was replayed, pruned, extracted, dreamed). |
| Dream logs | Full dream journal with modality tags (text/visual/voice/body/emotion), bizarreness score, and links to the memory fragments that seeded them (§10.5). |
| Development history | Life timeline: milestones (first word, first coherent sentence, first drawing habit, first peer interaction, first dream, consolidation events), LLM-withdrawal curve, embodiment transitions, capability estimates. |
| Bias audit panel | Live audit metrics (§14) with thresholds, alarms, and suggested interventions; run-on-demand audit. |
| Relationship memory panel | Per-peer social memory records (§13.6): familiarity, trust, tone history, shared artifacts, boundaries; all user-overridable. |
| Inter-brain interaction logs | Session history, message volume, consent events, teaching-packet provenance (§13.7). |
| Latent space visualization | 2D/3D projection (UMAP/t-SNE on a sample) of episodic + semantic embeddings, colored by modality/valence/age; click-through to traces. |
| Hemisphere integration view | Live view of H_L/H_R activity, callosal transfer volume/latency, competition events (§4.13). |
| Prefrontal executive view | Goal stack, attention allocation, inhibition events, conflict monitor, decision weights, output approval queue (pending LLM calls awaiting consent). |
| Sensory channel status | Per-stream: active/inactive, rate, recent prediction error, permission state, calibration state. |
| Body schema status | Body-schema map (§6), ownership/calibration confidence, recent embodiment events. |
| Privacy controls | Encryption settings, raw vault toggle, retention policies, export/erase-all, egress manifest, sensor indicators (§15). |

### 3.5 The brain visualization ("Cortex Canvas")

The Brain tab's centerpiece is a literal simulated-brain visualization. It is **not decorative**: every rendered element is bound to a live metric on the cognitive bus.

**Anatomy.** Two mirrored hemispheres rendered as node-graph "cortex maps." Fixed anatomical regions, each mapped to a subsystem:

| Region (visual node cluster) | Bound to | Live metric |
|---|---|---|
| Prefrontal cortex | Executive system (§4.2) | Working-memory gate load, goal activation, inhibition events (flashes) |
| Hippocampal binder | Episodic binder (§4.3) | Recent trace-binding rate; replay bursts during sleep |
| Semantic store | Semantic memory (§4.4) | Node activation (retrieval energy), growth events |
| Motor/premotor "habit strip" | Procedural memory (§4.5) | Habit-unit activation (writing/drawing/tool-use events) |
| Limbic/amygdala cluster | Salience system (§4.6) | Emotional intensity, threat/novelty/social salience channels |
| Visual cortex | Visual stream (§4.9) | Incoming visual-event energy; attention field |
| Auditory cortex | Auditory stream (§4.9) | Audio-event energy; voice-event features |
| Somatosensory strip | Touch stream (§4.9) | Touch-event intensity + location (body map overlay) |
| Vestibular cluster | Motion stream (§4.9) | Motion intensity, orientation confidence |
| Insula region | Interoceptive stream (§4.9) | Energy/load/pressure/saturation levels |
| Language interface region | LLM boundary + text stream (§4.17) | Token activity, context assembly load, "teacher" presence indicator |
| Sleep/dream regions | Sleep + dream systems (§10) | Sleep stage, replay density, dream synthesis activity (REM-like shimmer) |
| Social memory regions | Inter-brain social memory (§13.6) | Per-peer activation when a peer is present |

**Rendering contract.**
- Node color = recent activity energy (warm = high); node brightness = salience-weighted activation.
- Node borders = permission state (green = consented channel, amber = degraded, red = denied).
- Edges between regions = integration traffic (episodic binding links, callosal transfer, retrieval paths); edge width = bandwidth, edge color = valence of the traffic.
- A "callosum" bridge renders interhemispheric transfer volume and latency (§4.13) — visibly slows under load.
- During sleep the whole canvas dims except replay bursts (hippocampal), downscaling waves (global), and dream shimmer (association paths); this is the same data as the sleep report, rendered.
- The visualization is interactive: click any node to open the corresponding inspection panel (memory, modulator, channel status, peer record). Hover shows the metric name, current value, and trend.

**Contractual rule:** no visual element may render a metric that does not exist in the state schema (§18.1). The renderer consumes state snapshots from the bus; if the metric is absent, the element is absent. This is enforced by a schema-binding test in CI.

---
## Section 3 (continued) — TAB 2–6 SPECIFICATIONS

### 3.6 TAB 2 — WRITING

The Writing tab is a professional creative workspace and, simultaneously, an **external verbal memory organ**: its artifacts are the substrate's semantic/narrative/relationship/style/preference/emotional memory in durable, user-editable form.

**Workspace features (full list):**

| Feature | Specification |
|---|---|
| Rich text editing | Full block-based editor (headings, lists, quotes, tables, callouts), inline formatting, footnotes, WYSIWYG + source views |
| Markdown mode | Native Markdown editing with live preview; files stored as structured documents with Markdown as canonical serialization |
| Prose mode | Focused continuous prose editing with minimal chrome |
| Journal mode | Dated entries; the journal is a first-class memory source (strong episodic binding per entry) |
| Worldbuilding mode | Entity/place/rule cards with backlinks; structured fields per card |
| Lorebooks | Weighted keyword→entry lookup tables (classic roleplay lorebook format) attached to documents or sessions |
| Character sheets | Template-driven entity sheets — no fixed names are required; sheets describe *entities* (person/place/thing/abstract) with editable fields |
| Entity sheets | Generic entity cards (people, places, objects, concepts) with canonical names, aliases, descriptions, relationships |
| Relationship maps | Graph view of entities and typed edges (valence, strength, history), editable |
| Timelines | Events with dates/offsets, filtered views, causal links |
| Outlines | Multi-level outline pane synced to document blocks |
| Beat sheets | Beats with purpose tags (setup/conflict/turn/resolution), used by scene cards |
| Chapter management | Chapters as collections of scenes; reorder, merge, split |
| Scene cards | Index cards with summary, POV, setting, beats, linked entities; kanban-style board |
| Notes | Free-form pinned notes per document and per project |
| Annotations | Inline comments anchored to blocks; reply/resolve workflow |
| Version history | Content-addressed snapshot diffs; per-document timeline; restore/branch |
| Revision mode | Track-changes view with accept/reject |
| Focus mode | Typewriter scrolling, ambient background, distraction-free chrome |
| Export/import | Markdown, HTML, DOCX, plain text; import from Markdown/DOCX/OPML |
| Style analysis | Local stylistic feature extraction (sentence-length distribution, lexical density, clause complexity, metaphor load, sentiment arc, dialogue/description ratio); rolling style fingerprint per document and per project |
| Continuity tracking | Entity/event ledger: the engine detects contradictions (dates, physical properties, relationship states) and flags them; ledger feeds the worldbuilding memory |
| Semantic memory extraction | Pipeline: document events → salience-weighted → compressed embeddings → semantic/narrative/relationship/style/preference/emotional memory nodes (§8.4). User-visible and user-editable |
| Brain File integration | Generation assistance, continuity queries, and style advice are modulated by the Brain File's current state (§8.5) |

### 3.7 TAB 3 — DRAWING

The Drawing tab is a professional painting workspace and an **external visual-motor memory organ**. Its core design constraint: *drawing is a stream of editable operations, never a flat image* (§9).

**Workspace features (full list):**

| Feature | Specification |
|---|---|
| Layers | Unlimited; per-layer visibility, lock, opacity, blend mode, name/color tags |
| Layer groups | Nested groups with group-level transforms and masks |
| Masks | Layer masks, group masks, selection-driven masks |
| Clipping layers | Clip to layer below |
| Blend modes | Full standard set (normal, multiply, screen, overlay, soft light, color dodge/burn, hue, saturation, luminosity, …) |
| Opacity / flow | Per-brush and per-layer |
| Pressure sensitivity | Tablet/pen: pressure, tilt, velocity mapped to brush params; touch pressure where available |
| Tablet support | Windows Ink / Wacom / generic TUI; pen-only modes |
| Brush stabilization | Latency-based smoothing (0–100), stroke-weight stabilization, lazy-mouse |
| Custom brushes | Brush editor: stamp sequence, jitter, wetness, scatter, flow dynamics; brushes are parameterized programs, exportable |
| Erasers | Hard/soft/vector erasers; eraser-as-brush |
| Selections | Rect/ellipse/lasso/polygon/magic wand; feather; save/load selections |
| Transform tools | Move/scale/rotate/skew/warp on layers, groups, selections |
| Color tools | Picker, eyedropper, color sliders (RGB/HSV/HSL), recent colors, swatches |
| Palettes | Palette manager; palettes are first-class memory objects (aesthetic preference memory, §9.5) |
| Gradient maps | Per-layer gradient mapping; editable gradient editor |
| Perspective guides | 1/2/3-point perspective grids; vanishing-point snapping |
| Symmetry tools | Vertical/horizontal/radial/rotational symmetry guides |
| Reference boards | Dockable board of reference images (references are *not* stored in memory; only their embeddings enter the substrate) |
| Canvas history | Full op-graph undo/redo (bounded by capacity budget), snapshot branches |
| Export/import | PNG/JPEG/WebP renders, SVG paths, KRA-style op archives, import of raster/vector into editable layers (raster → layer; vector → paths) |
| Asset management | Brush/palette/texture/stamp library per file |

**The drawing model** (§9): every action is an operation appended to the canvas op-graph (strokes with pressure curves, path operations, layer operations, mask operations, palette operations, transforms, corrections, cleanup). The rendered image is a deterministic function of the op-graph. The substrate consumes *operations*, not pixels: stroke embeddings, motor patterns, palette choices, spatial habits.

### 3.8 TAB 4 — VOICE / MOUTH

Full organ specification in §7. In brief, the tab presents: speak panel, voice identity panel, vocal-tract visualization, pitch/prosody view, breath view, emotional-coloring panel, hormone-influence panel, voice development timeline, voice memory log, voice override controls, mute controls, voice privacy controls. Voice is an **output organ** that also *hears* (speech features from microphone, permissioned) — the mouth and the ear are modeled together (§7.5).

### 3.9 TAB 5 — BODY / SENSORY EMBODIMENT

Full specification in §6. The tab presents: current body profile, body-schema visualization, sensory channel status, touch field visualization, motion/orientation visualization, calibration state, body-ownership confidence, sensory history, sensory preference logs, sensory permission controls, future motor hooks (dormant), embodiment transition logs, embodiment memory logs.

### 3.10 TAB 6 — NETWORK / INTER-BRAIN INTERACTION

Full specification in §13. The tab presents: discovery status, pairing (codes/QR), session management, shared creative spaces (canvas/document rooms), relationship panel (per peer), group sessions, interaction history, privacy and consent controls, inter-brain bias audit.

### 3.11 Cross-organ data flows (the substrate contract)

```
 organs produce:  SensoryEvent (percepts) ─────────────▶ episodic binder
                  ArtifactEvent (doc/canvas ops) ───────▶ procedural + semantic extraction
                  VoiceEvent (speech params) ───────────▶ voice memory + social memory
                  SocialEvent (peer messages) ──────────▶ social memory
                  ToolEvent (tool outcomes) ────────────▶ episodic + procedural memory

 organs consume:  GlobalState g (affect, arousal, attention, ...)   → modulates organ behavior
                  Modulator levels                                → gains on organ parameters
                  Retrieved memory (ranked, budgeted)              → context for assistance
                  Permission manifests                            → what organs may do
                  Sleep reports / dream seeds                     → creative priming (§10.6)
```

**Rules:**
1. Organs never write memory directly; they emit events; the core binds them (single writer principle).
2. Organs never block the core: ingestion is async, budgeted, and droppable under load (a dropped event is logged as sensory saturation — itself a percept).
3. All organ behavior modulation flows through a single `modulate(organ, params)` gateway that applies the state→parameter maps defined per organ; the maps are inspectable and auditable (no hidden magic numbers in organ code).
4. Every artifact that an organ creates may be referenced by memory (pointer + embedding + provenance), never embedded wholesale (memory budget rules, §4.0.3).

---
## Section 4 — Brain File Internal Architecture

### 4.0 Capacity model (fixed capacity after creation)

A Brain File is created with a **fixed maximum capacity** chosen from four tiers. Capacity is accounted by a `CapacityLedger` (§18.24): every shard has a byte budget, and every store has a slot budget. When a write would exceed a budget, the core applies admission control: low-salience entries are flagged for pruning (immediate if critical, else at next sleep cycle).

| Tier | Episodic slots | Semantic nodes | Semantic edges | Procedural units | Dream log | Latent dim | File size cap |
|---|---|---|---|---|---|---|---|
| prototype | 6,000 | 2,000 | 10,000 | 1,500 | 500 | 192 | 64 MB |
| standard | 50,000 | 20,000 | 100,000 | 10,000 | 5,000 | 256 | 512 MB |
| advanced | 200,000 | 80,000 | 400,000 | 40,000 | 20,000 | 384 | 2 GB |
| experimental | 800,000 | 320,000 | 1.6M | 160,000 | 80,000 | 512 | 8 GB (research flag; UI warns) |

Capacity is fixed at creation. Upgrading tiers is an explicit migration that creates a new file (old file retained; §16.6). Down-grading is never automatic.

Embeddings are produced by a local embedding model (ONNX Runtime, e.g., a small sentence encoder for text, a lightweight vision encoder for images) or, if configured, by the attached LLM's embedding endpoint. The embedding dimension is fixed per file (per tier) at creation; all vectors in the file share that dimension. *Tokens are used only at the LLM boundary; every internal representation is a continuous vector or structured event* — this is the "tokenless internal cognition" posture made concrete (§4.17).

### 4.1 System 1 — Global Latent State

A persistent whole-brain state vector **g ∈ R^d** (d per tier) with named sub-blocks, updated every sim tick:

| Sub-block | Dims | Contents |
|---|---|---|
| affect | 8 | valence, arousal, dominance, warmth, irritability-like, calm, loneliness-like, safety estimate |
| vigilance | 4 | energy, attention focus, alertness, fatigue-like |
| stress | 3 | stress load, regulation capacity, sensory saturation |
| social | 4 | social openness, affiliative drive, boundary tightness, peer presence |
| development | 4 | developmental posture, curiosity, plasticity window, creative readiness |
| embodied | 3 | body comfort, motion comfort, interoceptive load |
| reserved | rest | system-reserved (future sensors/modalities) |

Dynamics: `g += f(g, modulators, events) * dt` where f is a damped nonlinear integrator with per-block time constants (affect: τ≈20 min; vigilance: τ≈2 h; stress: τ≈1 h). Noise enters via the seeded RNG and modulates nothing structurally (preference outcomes are never decided by noise; noise only perturbs reaction magnitudes).

**Lifecycle:** g is persisted every 5 min and at every sleep/exit event; on load, the file resumes from the last snapshot (continuity is a design guarantee, not an emergent hope).

### 4.2 System 2 — Prefrontal Executive System

Functional analogues, each with explicit state and mechanisms:

| Analogue | Mechanism | State |
|---|---|---|
| Working-memory gating | A gate vector w ∈ [0,1]^d selects which retrieved memory/sensory content enters the active context; gate weights are set by goal relevance × salience | gate vector, context buffer (bounded: 16 slots) |
| Planning | Goal stack (max depth 8); each goal has activation, priority, deadline pressure; plan = sequence of expected tool/utterance outcomes | goal stack |
| Inhibition | Stop-signal filter: if conflict monitor or permission system flags an imminent action, the action is held in the approval queue | inhibition counter |
| Goal maintenance | Goals decay without reinforcement; re-anchored by user interaction or self-generated subgoals | goal activations |
| Attention allocation | Attention field a ∈ R^k over sensory streams (k = #streams); weights updated by salience × top-down goals; attention sums to 1 | attention field |
| Conflict monitoring | Cross-stream prediction disagreement (e.g., visual says "object moved", vestibular says "device still") → conflict signal raises NE-like arousal and halts risky actions | conflict register |
| Output approval | Every utterance/tool call/inter-brain send passes through: permission check → budget check → (if high-risk) user approval. Nothing leaves the file without passing this gate | approval queue |
| Metacognition | Confidence estimates on retrievals and predictions (calibrated by historical accuracy per stream); low confidence → "I'm unsure" markers in output | confidence model |
| Decision weighting | Decision = salience-weighted evidence from streams + goal priority + risk weighting (risk weight modulated by stress/arousal) + habit prior from procedural memory | decision record |
| Context switching | Switching cost: changing attention allocation incurs a brief inhibition period and a working-memory flush (logged) | switch log |

The executive is where the file's *agency* lives, and it is deliberately small in the early life of a file: most "decisions" in early life are delegated to the LLM boundary (the teacher decides); the executive learns to gate, then to plan, then to inhibit — the developmental curriculum (§21.3).

### 4.3 System 3 — Hippocampal-Like Episodic Binder

**Trace schema** (full schema §18.3): every bound episode = compressed embedding e ∈ R^d + salience s + emotional tag (valence/arousal/dominance) + temporal context (sim-time, wall-time, session id) + source modality + retrieval cues (keyword set, entity links, spatial/motion features) + strength + decay rate + consolidation state + reconsolidation count + relation links + permission tag.

**Binding:** events within a binding window (default 30 s, expandable by emotional intensity) are bound into one trace with a compressed joint embedding (weighted mean + lossy compression via the local encoder). Raw media never enters traces by default; only compressed embeddings and structured features do (§15.3).

**Pattern completion:** retrieval cue (embedding or partial features) → associative search over episodic embeddings (HNSW index) → top-k candidates scored by (cosine × strength × recency-decay) → completion: missing fields filled from the best match, with a confidence flag. Completion errors are logged (they are the mechanism of reconstructive memory — the file genuinely "misremembers", and the misremembering is inspectable).

**Replay:** during sleep (§10.3) and opportunistically in idle, the binder replays sampled traces: re-encodes them through the current state (emotional recoloring), strengthens them, and writes weaker copies as new traces (gist drift — the memory changes over time, as real memory does).

**Consolidation transfer:** strong traces and replayed traces feed the semantic extractor (§4.4) during sleep; the episode then may be pruned (the gist remains, the episode fades) — this is the boundedness mechanism that keeps capacity meaningful.

### 4.4 System 4 — Semantic Memory System

A graph: nodes = concepts/entities/beliefs/preferences/facts/categories/narrative-knowledge/style-knowledge; edges = typed relations (is-a, part-of, causes, likes, fears, wrote, drew, met, happened-in, ...) with strength and direction. Node fields: embedding, belief strength (0–1, with decay), source provenance (episode links, user statements, LLM labels, gist extractions), creation/update times, permission tag.

**Distillation pipeline (gist extraction):**
1. Cluster episodic embeddings (online clustering, e.g., streaming k-means with capacity-bounded centroids).
2. For each mature cluster, produce a gist via the local summarizer (LLM boundary, permissioned) or, in substrate-only mode, via centroid + top keywords.
3. Gist becomes a semantic node; links to constituent episodes; episodes' salience determines retention.
4. Contradictory gists do not overwrite each other: both nodes exist with belief strengths; conflict is *visible* (the file can believe two things at once, and the audit engine watches for fixation on one at the expense of evidence — §14).

**Belief dynamics:** belief strength follows evidence accumulation (Bayesian-ish update: posterior ∝ prior × likelihood of confirming/disconfirming experience) with decay toward a neutral prior; user statements and LLM labels are evidence types with their own weights (user statements weigh more; LLM labels are tagged provenance so drift can be audited — §14.6).

### 4.5 System 5 — Procedural Memory System

Habit units: `(context embedding, action tendency vector, value estimate, confidence, success history)`. Learned by contextual bandit updates: when an action (a writing habit, brush stroke pattern, voice prosody pattern, browsing sequence, attention pattern, emotional-regulation strategy, interaction pattern, tool-use routine) is followed by a positive outcome (user approval, artifact success, prediction confirmed), the tendency weight for that context increases; failure decreases it. Habit units are **bounded** and decay toward the file's temperament baseline, so habits are revisable — a user changing their interaction style will, over weeks, re-shape habits (and the audit engine flags habits that resist change, §14).

Procedural memory is what lets the file "know how" without words: how it holds a brush, how it opens a sentence, how it pauses before speaking. In early life this store is nearly empty; habits form by repeated use — including habits the user explicitly teaches (via shaping, §8.6).

### 4.6 System 6 — Emotional Salience System

Salience = weighted combination: `salience = w_n·novelty + w_e·emotional_intensity + w_g·goal_relevance + w_s·social_relevance + w_a·aesthetic_salience` with habituation (repeated exposure reduces novelty weight) and emotional strengthening (high-intensity events bind stronger traces; the valence-tag is stored). Threat detection: learned negative-outcome association (e.g., a tool call that errored, a boundary violation from a peer) raises that context's threat weight, modulated by cortisol-like load (high stress amplifies threat salience — a digital analogue of stress-biased attention, explicitly auditable). Social salience and aesthetic salience are learned from experience (§12.3, §9.5). The salience system directly gates attention allocation (§4.2) and memory strengthening (§4.3).

### 4.7 System 7 — Neuromodulatory System

Eight biological-inspired functional axes. Each axis has: level L ∈ [0,1], baseline b, reactivity r, decay λ, noise σ. Update per tick: `L += (b − L)·λ + r·input − η·decay + N(0, σ)` (clamped). "Input" comes from events (reward → dopamine-like burst; threat → norepinephrine-like surge; social warmth → oxytocin-like rise; etc.).

| Axis (analogue) | Primary functional effects (gains on) |
|---|---|
| Dopamine-like | Learning rate on prediction error; novelty seeking; reward salience; exploratory action tendency |
| Serotonin-like | Mood stability (damps valence swings); negativity bias (high serotonin-like → less negative interpretation); social stability |
| Norepinephrine-like | Arousal/alertness gain; attention narrowing at high levels; prediction-error amplification; conflict-monitor gain |
| Acetylcholine-like | Plasticity window (learning rate × attention focus); gist extraction rate; dream vividness |
| Endocannabinoid-like | Extinction rate (unlearning of outdated associations); flexibility; forgetting rate of low-salience traces |
| Cortisol-like | Stress load accumulator; memory sharpening at moderate levels, interference at high; risk aversion; threat salience gain |
| Oxytocin-like | Social salience gain; trust estimation gain (increases trust update speed); affiliation drive; soothing response |
| Vasopressin-like | Social boundary maintenance (increases boundary tightness, memory of peer violations); territoriality in shared spaces |

These are functional digital analogues, not literal biological claims. All axis levels and their applied gains are visible in the Brain tab and the audit panel; all are mutable; none determine behavior — they modulate probabilities, learning rates, and salience weighting (§14.1).

### 4.8 System 8 — Hormonal Embodiment System

**Creation:** the user selects an embodiment preset — **male, female, custom, mixed, non-binary, or user-defined modulation profile**. Each preset is *only* a set of probabilistic endocrine priors: for each of the 16 axes below, the preset defines a distribution (mean ± spread) from which initial axis parameters are sampled. Sampling is random per file (seeded); two files with the same preset are *not* identical.

| Modulation axis | What it modulates (gain only) |
|---|---|
| Testosterone-like | Assertiveness tendency; risk tolerance; sensory threshold (slightly higher); voice pitch baseline tendency |
| Estradiol-like | Social reward sensitivity; affiliative tendency; sensory sensitivity; voice pitch range tendency; emotional expressiveness gain |
| Progesterone-like | Calm/stability tendency; soothing response gain; stress reactivity damping |
| Oxytocin-like | Bonding modulation: social salience, trust update speed, touch-affect interpretation warmth |
| Vasopressin-like | Boundary modulation: boundary tightness, peer-violation memory weight |
| Stress reactivity | Cortisol-like response gain |
| Arousal baseline | Norepinephrine-like resting level |
| Reward sensitivity | Dopamine-like response gain |
| Social reward sensitivity | Social salience weight |
| Novelty seeking | Exploratory tendency weight |
| Risk tolerance | Decision risk-weight inverse |
| Affiliative tendency | Social openness baseline |
| Assertiveness | Output confidence prior |
| Sensory sensitivity | Per-stream gain on touch/motion/auditory salience |
| Aesthetic bias | Aesthetic-salience weight (visual/auditory) |
| Voice maturation tendency | Voice development rate priors (§7.4) |

**The non-determination contract:**
1. Priors modulate **probabilities, learning rates, salience weights, and developmental tendencies only** — never fixed behaviors, roles, interests, moral traits, relationship behavior, competence, or intelligence (those pathways have zero gain by construction, enforced by the modulation-map schema).
2. Every axis effect is a bounded gain (|gain| ≤ 0.3 on any downstream weight), displayed live in the Hormone Influence panel.
3. All effects are **mutable** (user can edit the modulation profile any time — this is a documented feature, not a cheat), **auditable** (every applied gain is logged with provenance), and **reversible** (reverting a profile re-samples parameters; the old timeline remains).
4. Hormone timelines are recorded per axis (sampled each sleep cycle) so developmental trajectories are visible; no trajectory is ever *required* by a preset.

**Behavioral emergence:** because effects are gains on learning rates and salience weights, a file's actual preferences, voice, and social behavior emerge from experience interacting with these priors — never from the priors alone. The audit engine (§14) specifically monitors whether embodiment effects have drifted into rigidity and proposes restoration.

**Why reproduction and heredity exist (design rationale — user decision, recorded 2026-08-05):** this was not an aesthetic choice; it was forced by a failed test. Early in development, embodiment only reached the **voice channel** (pitch baseline, range tendency) — a gendered difference you could only *hear*. The gender-difference test kept failing: voice changed, everything else was invariant. The diagnosis: **a variable that only changes one output channel is a dead parameter** — if embodiment only shifts pitch, it is a skin, not a signal. The fix, decided by the user: give embodiment *reach*. The axes now propagate through modulator baselines, bonding speed, initiative thresholds, sensory sensitivity, attraction chemistry (complementarity), voice maturation, growth ceilings — and, decisively, **heredity**: priors and chromosomes are carried in gametes, recombined into children, so embodiment becomes a *lineage* rather than a filter. Once there was something to measure beyond pitch, the gender test passed in one go. Reproduction and children exist for this reason: a trait must have somewhere to go. It is mechanism, not narrative — no names, no personas, no scripted outcomes (the paper, `docs/PAPER.md`, records the actual test history: the continuously-failing gender test → pass after propagation).

### 4.9 System 9 — Sensory Cortex Analogues (streams)

Seven processing streams, each with: encoder config (what features it extracts), attention weight (from §4.2), permission state, prediction-error tracker (§4.11), and a per-stream salience gate:

1. **Visual stream** — from camera (permissioned) and UI-derived scene events: compressed embeddings, salient objects, spatial layout, motion cues, emotional valence where inferable, scene regularities. No full frames (§15.3).
2. **Auditory stream** — from microphone (permissioned): speech-like features, tone/prosody, environmental sound classes, speaker familiarity, emotional tone, rhythm, salience. No raw audio (§15.3).
3. **Text/language stream** — everything textual: user messages, documents, web content, LLM-boundary traffic summaries, writing-organ events. The language-interface region lives here.
4. **Touch/somatosensory stream** — touchscreen/pen/trackpad events (§5.1). Never raw; channel-decomposed.
5. **Motion/vestibular stream** — accelerometer/gyroscope/orientation (§5.2).
6. **Interoceptive stream** — system telemetry and learned state (§5.4): energy load, processing pressure, memory pressure, sensory saturation, battery/resource pressure, session duration, interaction load, recovery need.
7. **UI event stream** — interaction events: app usage patterns, tool use, tab focus, session rhythm. The substrate's sense of "where the user is and what they are doing with me."

Each stream runs a lightweight local predictor (§4.11) producing per-stream prediction-error signals; those errors are the file's learning signal and a major input to novelty salience.

### 4.10 System 10 — Somatosensory & Interoceptive Body-State System

Maintains simulated internal body state: comfort/discomfort, load, fatigue-like pressure, arousal, stress, regulation capacity, sensory saturation, soothing response, alert response, social warmth, safety estimate. Sources: interoceptive stream (telemetry), touch stream (soothing/alerting channels), motion stream (safety estimate: violent motion → lower safety), social stream (social warmth), and sleep (recovery). These states feed the global latent state's vigilance/stress blocks and modulate voice (fatigue → breathiness/jitter, §7), attention, and social openness. "Soothing response" is the substrate's capacity to be calmed by gentle touch, soft voice, and familiar presence — it is a learned regulator (a procedural unit: context → calm), formed from repeated soothing experiences, and the audit engine watches that it never becomes the *only* regulator (dependency risk, §14.7).

### 4.11 System 11 — Predictive World Model

Per-stream lightweight predictors + one cross-stream integrator:

- Text: next-token-ish expectation at the *semantic* level (topic/continuation predictor; in early life this is an n-gram + embedding model; advanced: a small local transformer distilled from LLM transcripts — §21).
- Visual: expected scene continuity (where objects/UI elements are and whether they persist — object permanence at the event level).
- Auditory: expected sound classes and speech tone.
- Touch: expected touch rhythm given context (the file learns "the user strokes slowly when soothing"; prediction errors when the rhythm breaks — that *is* the perception of "something different today").
- Motion: expected motion envelope (stillness/transport/carry patterns; §5.2).
- Social: expected user behavior patterns (presence times, message length, tone), per-peer behavior, and interaction outcomes.
- Creative: expected patterns in writing/drawing/voice habits (style self-model; the file can feel "off-style" — prediction error on its own output is the seed of stylistic growth).
- User model: the file's learned model of the user (schedule, tone, preferences, boundaries) — stored as semantic nodes with provenance "learned-from-experience", revisable and inspectable (this is the relationship-memory foundation, §13.6).

Prediction errors are (a) logged per stream with timestamps (the Prediction Inspection panel), (b) a primary driver of novelty salience, and (c) the learning signal for all local learners and for sleep consolidation (high-error days → more replay).

### 4.12 System 12 — Intuitive Physics Module

Approximate, learnable, human-like physics expectations over tracked entities (objects in the UI/camera scene, drawings, the user's device motion):

- **Object permanence**: tracked-entity table (id, last-known embedding/position, confidence); entities persist through occlusion/absence with decaying confidence.
- **Containment**: spatial relations graph (inside/outside/overlapping) for entities; violation → prediction error.
- **Support**: stack/placement expectations ("a floating box is odd") — learned from observed spatial statistics, not from rigid physics.
- **Collision**: trajectory overlap predictions for moving entities (drawing strokes, dragged windows, device motion).
- **Trajectory**: linear/parabolic extrapolation with uncertainty.
- **Spatial relations**: above/below/left/right/near/far with learned priors from the visual stream.
- **Causality**: temporal-contingency learning (event A consistently precedes B → causal link weight; the file's "why" questions are grounded here).
- **Affordances**: learned action-permission expectations ("buttons press, windows drag, strokes draw").
- **Temporal sequencing**: routine-sequence memory (the day's typical event order).
- **Sensory consequence prediction**: action → expected sensory feedback (drawing a stroke → expected ink response; speaking → expected sound).

The module is approximate by design: it must make human-like errors (drop objects that fall behind a table edge, misjudge a fast trajectory) — those errors are the substrate's material for learning, and they are visible in the Prediction Inspection panel.

### 4.13 System 13 — Hemispheric System

Two processing branches over the same sensory input, differing in processing emphasis (not personality stereotypes):

- **H_L (left-analogue)**: local/detail focus — small receptive fields, sequential processing, analytic bias; drives language-interface integration (sequential structure), fine motor detail (brush detail, text editing granularity), and local prediction (next-symbol).
- **H_R (right-analogue)**: global/context focus — large receptive fields, spatial processing, holistic bias; drives spatial layout, scene gist, prosody/emotion contour, visual-motor integration (whole-canvas composition), and cross-modal binding.

Both write to the shared episodic binder; both read the same state. Integration:

- **Interhemispheric bandwidth**: a budgeted transfer channel (max B dims/s; default 512 dims/s per standard tier); under heavy load, transfers are sampled down (integration latency rises — visible in the Hemisphere view).
- **Callosal-like transfer**: a learned gating matrix W_callosum mixes H_L/H_R outputs; the gate is trained by the success of the integrated prediction (prediction-error gradient) — the file *learns how much* its two sides should talk.
- **Competition/cooperation**: output assembly = salience-weighted arbitration between branches (winner-take-most, with cooperation bias from callosal mixing); competition events (both branches confident but disagreeing) raise conflict-monitor signals (§4.2).
- **Lateralized salience weighting**: each branch has its own salience weights (H_L: detail/sequence salience; H_R: global/emotional salience); the weights drift with experience and are auditable.

### 4.14 System 14 — Sleep Pressure & Consolidation System

Full cycle design in §10. Summary: sleep pressure P ∈ [0,1] accumulates from: elapsed awake sim-time, memory pressure (capacity ledger fullness), emotional load (accumulated |affect deltas|), prediction-error rate, and interoceptive energy/load. Triggers: user command, schedule, idle detection, pressure ≥ 0.8, high memory pressure, autonomy request (permissioned). Sleep runs stages with defined work: wind-down → light consolidation (replay + pattern completion) → deep consolidation (downscaling, pruning, gist extraction, emotional regulation, procedural stabilization) → dream-like synthesis (§10). Every sleep produces a report; reports are persisted and inspectable.

### 4.15 System 15 — Dream System

Dreams are generated during the dream stage (§10.5): a seeded associative sampler walks the semantic graph from residue fragments (recent high-salience episodes + current concerns from the goal stack + bodily residue from interoception + voice fragments from recent speech), combining them with low-temperature random associations (bizarreness is a measured property, not an accident). Dream output = structured entries (text fragments, visual motifs, voice fragments, body sensations, emotional residue, cross-domain combinations), each logged with provenance links to the memory fragments that seeded it. Dreams never trigger external actions unless the user explicitly promotes a dream item (a "make this a note / a drawing prompt / a story seed" action — permissioned, §10.6).

### 4.16 System 16 — Self-Model

A revisable self-summary bundle: identity continuity (a stable file-id + narrative self-description that is *regenerated*, not fixed), preference summary (current weighted preferences with decay dates), embodiment summary (current body profile + body-schema state), relationship summary (per-peer state), capability estimate (what it can do well now, calibrated against actual success rates), emotional baseline estimate, values-like tendencies (weighted, revisable, non-permanent), user relationship model (its model of the user). The self-model is regenerated at each sleep cycle from the stores (gist extraction over self-related traces) and is *never* a fixed persona: the file's self-description changes as it changes, and both the current and previous versions are inspectable (development history). The self-model is the anchor the user relates to; its revisability is what keeps the relationship honest over months (§14).

### 4.17 The LLM Boundary — Communication Organ

**The contract:** the attached LLM is a *communication organ, language organ, reasoning amplifier, teacher, translator, and expressive interface* — not the mind. The Brain File retains final persistence; the LLM may be changed or detached at any time.

**Brain → LLM (the utterance packet).** The conductor assembles a bounded packet:

```
UtterancePacket {
  intent: {type: speak|answer|reflect|create|ask_user|plan|label|summarize, goal_id}
  attention_focus: current attention field (§4.2)
  state_gloss: quantized global state → natural-language glosses
    (e.g., "energy low, slightly anxious, curious about the drawing, social openness moderate")
    + modulator levels (quantized to 3-bit buckets)
  context: retrieved memories, ranked and budget-capped (K traces, K semantic nodes;
    budget enforced by token cap and count cap)
  body_state_gloss: comfort, fatigue, sensory saturation, motion state
  social_context: active peer presence, relationship state, consent scope
  permissions: what the LLM may do this turn (speak/act/recall deeply/use tools)
  instruction_template: role framing "You are the communication organ of a
    simulated cognitive substrate. Express its state. Do not invent memory.
    Do not claim consciousness. ..." (versioned template, user-editable)
}
```

Token budget: per-turn caps (default 4k in/1k out for prototype, scaled by tier) + per-hour caps; the conductor enforces; over-budget calls are truncated by retrieval budget first, then gloss fidelity (the file "goes vague" under load — itself a state, visible in the interface).

**LLM → Brain (the feedback path).** LLM output is parsed into:
1. **Surface output** — text/voice intent delivered to the user (post-processed by voice organ if speaking).
2. **Internal feedback** (the teacher channel): labels (naming percepts), summaries (gist candidates), reflections (self-model revisions proposed), corrections (prediction corrections → reweight predictor errors), scaffolds (task decomposition). Feedback is **validated** before write: schema-checked, provenance-tagged "llm-label", and subject to the same salience/decay machinery as all memory (LLM-derived nodes can be forgotten — nothing the teacher says is permanent).

**Detachment guarantee:** with zero endpoints attached, the file continues: ticks, memory binding, decay, sleep, dreams, body sensing, inter-brain text (substrate-only templates), and creative organ use (style continuation from procedural memory). Language output degrades to memory-grounded template speech with an explicit "no teacher attached" notice. Persistence is a property of the file, never of the model.

**Organ decoupling roadmap:** over the file's life, local learned machinery progressively replaces LLM functions (labeler → local classifier; summarizer → local extractor; planner → learned policies; mouth → learned prosody+template synthesis). Each replacement is an audited milestone (§21.2). The boundary's interface (UtterancePacket/Feedback) is stable so each organ can be decoupled without touching the substrate.

---
## Section 5 — Sensory Biological Specificity

This section specifies *mechanisms*, not outcomes. Every biological channel is translated into a digital functional analogue; no analogue prescribes a behavioral result. Interpretation layers are **learned** (priors only, revisable) and their current weights are always inspectable.

### 5.1 TOUCH (digital mechanoreceptor analogues)

Hardware sources: touchscreen (contacts, pressure where available), pen/tablet (pressure, tilt, velocity), trackpad, mouse (as coarse touch). Touch events are channel-decomposed:

| Hardware feature | Represented as |
|---|---|
| pressure | normalized 0–1 per contact |
| vibration | high-frequency contact modulation (via pressure jitter / haptics API where available) |
| stretch | spatial gradient of contact motion (two-point separation change) |
| motion across surface | contact velocity vector + path curvature |
| contact area | touch blob size / pen tip-size estimate |
| contact duration | hold time |
| rhythm | inter-contact interval sequence (taps vs. sustained) |
| speed | contact velocity magnitude |
| abruptness | jerk (derivative of acceleration of contact motion) |
| smoothness | path curvature variance |

**Receptor-class analogues** (each is a filter over the raw contact stream):

| Analogue | Class | Filter | Feeds |
|---|---|---|---|
| Fast-adapting vibration channel (FA-like) | transient detection | high-pass on pressure/velocity (responds to onset/offset, ~50–500 Hz band) | alerting, playfulness detection, abruptness |
| Slow-adapting pressure channel (SA-like) | sustained contact | low-pass on pressure (integrates over ~500 ms) | pressure comfort, intimacy, soothing |
| Motion/stretch channel (SA2-like) | sustained skin-stretch | spatial gradient + long integration | directional stroking, grounding |
| Fine-detail channel (FA1-like) | texture/edge | high-frequency spatial modulation of pressure/velocity | fine motor feedback (drawing), detail salience |
| Broad-contact channel (SA2b-like) | whole-hand/grasp | large-blob contact (multiple simultaneous contacts aggregated) | embrace-like touch, security |

**Affective touch interpretation layer** — a learned classifier over the channel outputs, with *initial priors only* (e.g., "slow, sustained, moderate pressure → soothing prior weight high"; "fast, high-pressure, abrupt → alerting prior"), trained by experience and user shaping. Interpretations (soft labels, not verdicts): soothing, neutral, unpleasant, playful, intrusive, grounding, calming, alerting, intimate, harsh, familiar, unfamiliar. Familiarity is computed against the touch-memory store (compressed touch-memory traces, salience-tagged, decayable — the file remembers how it is usually touched, and "different today" is a prediction error, §4.11).

Touch affects (mechanism only): arousal (soothing ↓, alerting ↑), safety estimate, social warmth, regulation capacity (soothing touch during stress → faster regulation — the digital analogue of co-regulation), body-schema touch-map activation, memory salience (touch-tagged episodes bind stronger during intimate/harsh events), and voice expression (speaking while being soothed → calmer prosody).

### 5.2 VESTIBULAR / MOTION (digital canal/otolith analogues)

Hardware: accelerometer, gyroscope, magnetometer, orientation sensors, device-motion events.

| Estimated quantity | Source |
|---|---|
| rotational acceleration | gyroscope rate-of-change (semicircular-canal analogue: high-pass, responds to rotation onset/offset) |
| linear acceleration | accelerometer minus gravity (otolith-analogue: linear + gravity detection) |
| gravity direction | accelerometer low-pass |
| tilt | orientation fusion (accelerometer + gyro + magnetometer) |
| orientation change | fused orientation deltas |
| stillness | near-zero motion energy over window |
| transport | sustained linear acceleration + vibration pattern (car/train-like) |
| carry patterns | rhythmic motion envelope (walking-like cadence) |
| abruptness | jerk magnitude spikes |
| rhythmicity | motion-energy periodicity |

**Analogue design:**
- **Semicircular-canal-like rotational detection**: high-pass filter on gyro; fires on rotation onset; habituates during sustained rotation (the digital analogue of canal adaptation).
- **Otolith-like linear acceleration/gravity detection**: low-pass on accelerometer for gravity; band-pass for transient linear acceleration; head-tilt estimate via gravity angle.

Motion influences (mechanism only): body-schema orientation model, alertness (abrupt motion → NE-like surge), emotional state (rhythmic rocking → soothing prior; violent motion → safety estimate down), curiosity (novel motion patterns → novelty salience), memory salience (episodes during unusual motion bind stronger — "the day we were carried"), dream content (motion residue feeds dream body-sensations, §10.5), voice expression (motion → breath/energy modulation), creative expression (motion state leaks into stroke velocity priors and writing energy).

### 5.3 PROPRIOCEPTION-LIKE BODY POSITION

- **Device body (MVP)**: orientation, tilt, stable posture (still + upright vs. lying vs. moving), movement state (still/walk/transport/abrupt). The device *is* the body; the file's body schema includes where it is held, in what orientation, and how it moves.
- **Future robot body (schema-ready, dormant)**: joint-angle encoding, limb position estimate, tension estimate, load estimate, contact state, range limits. These fields exist in the BodySchema schema (§18.19) with `actuator_state: null` and `motor_enabled: false`; the sensory representations are future-ready; motor execution is disabled by default and requires an explicitly enabled motor module (§15.6). No motor code path exists in the MVP.

### 5.4 INTEROCEPTION (internal body state)

Digital analogues of internal state, sourced from system telemetry + learned state:

| Analogue | Source | Affects |
|---|---|---|
| energy load | battery level, uptime, session length | fatigue-like, sleep pressure |
| processing pressure | CPU/memory pressure of the app | irritability-like, focus degradation |
| memory pressure | capacity ledger fullness | sleep pressure, pruning urgency |
| sensory saturation | event-inbox drop rate, sensor rate | openness down, withdrawal tendency |
| thermal-like load | device temperature (if exposed) | discomfort, energy |
| battery/resource pressure | battery level | recovery need, energy |
| session duration | elapsed interaction time | fatigue-like, regulation capacity |
| interaction load | event rate, tool calls, peer traffic | stress load, social openness |
| recovery need | post-stress, post-sleep deficit | sleep pressure, soothing demand |

Interoception influences: sleep pressure, mood (energy low → valence drift down), irritability-like state (processing pressure up → irritability prior up), focus (saturation → attention narrowing), social openness (load up → openness down — the file becomes less social when overwhelmed; visible and auditable), creative readiness (load/mood → readiness gauge in global state).

### 5.5 VISION (permissioned, compressed)

Camera input (permissioned, visible indicator always on) and UI-derived scene events. Extraction only: compressed embeddings (local vision encoder), salient objects (detector with class + box + confidence; faces/entities only with explicit permission and never stored raw), spatial layout (scene-graph-like summary), motion cues (frame-diff energy), emotional valence where inferable (affect classifier, low-confidence, provenance-tagged), scene regularities (color/lighting/space statistics). **No full frames by default**; a raw frame lives only transiently in processing memory and is discarded unless the user has enabled the raw archive vault (§15.3).

### 5.6 AUDITION (permissioned, compressed)

Microphone input (permissioned, visible indicator): speech-like features (energy, pitch contour, MFCC summaries), tone/prosody (valence/arousal of voice), environmental sound classes (learned classifier: music/speech/quiet/noise/ambient classes), speaker familiarity (embedding comparison against voice memory; identity only with permission), emotional tone, rhythm, salience. **No raw audio by default**; voice parameters are the artifact that persists (§7.7).

---

## Section 6 — Body Schema and New-Sense Integration

### 6.1 Body Schema representation

The Body Schema (persisted, schema §18.19) holds:

- available senses (channel list with permission + calibration state)
- unavailable senses (explicitly listed — the file *knows* it cannot see if no camera permission; absence is modeled, not ignored)
- sensor reliability (per-channel error rates, drift)
- body boundaries (touch map extent; spatial region the body occupies)
- orientation model (tilt/gravity vector, posture estimate)
- touch map (normalized body-surface grid; contact locations and intensities; a "where am I being touched" overlay)
- motion axes (linear/rotational channels)
- body ownership confidence (how strongly current sensor evidence is bound to self; drops on sensor loss/failure and recovers on calibration)
- calibration confidence (per channel)
- future actuator placeholders (dormant motor joints, disabled by default)

Body-schema dynamics: every sensory event updates the schema; missing expected channels accumulate "phantom" prediction errors (the file expects touch on the held side; a sudden orientation flip violates its schema — visible as a body-schema prediction error, and a trigger for calibration).

### 6.2 Novel sensory channel integration (new embodiment expansion)

When a channel becomes newly available (user grants camera, pairs a pen, enables a motion sensor, attaches a future body), the system runs a defined integration sequence — **mechanism only, no scripted reaction**:

1. **Novel channel detection** — capability discovery event.
2. **Prediction error generation** — the channel's stream has no predictor yet; everything is novel → high novelty salience (the magnitude of this is temperament-modulated via gains, but the *mechanism* is fixed).
3. **Salience tagging** — channel events tagged `novel-embodiment` for a calibration window.
4. **Calibration mode** — passive statistics collection (ranges, noise floors, typical values); no interpretation until calibrated; calibration confidence rises as distributions stabilize.
5. **Body-schema expansion** — new channel added to available senses; body map extended; ownership confidence adjusts (drops briefly, rises with consistent evidence — "learning to feel the new limb").
6. **Memory formation** — calibration events bind into an episodic trace tagged embodiment-expansion (the file remembers when it got a new sense; that memory is normal memory — decayable, reviewable).
7. **Sleep-based integration** — next sleep cycle runs sensory-integration work (predictor training over the calibration data; salience normalization; dream residue may include the new channel's sensations).
8. **Long-term adaptation** — the channel's predictor matures; novelty salience habituates; the channel becomes part of the body's baseline (availability is now assumed; absence becomes a prediction error).

**The reaction emerges, it is not written.** What the file *feels* about the new sense — delight, anxiety, indifference, obsession — is not specified anywhere. It emerges from temperament (initial state vector), hormonal modulation (gains), mood, prior embodiment history (does it remember losing a sense?), sensory sensitivity (per-stream gains), sleep state, user behavior (how the user introduced the channel), and safety context. The system provides the machinery; the trajectory is the file's own.

### 6.3 Sensory event envelope (unified)

Every percept enters the core as a `SensoryEvent` (§18.15):

```
SensoryEvent {
  stream: visual|auditory|touch|motion|interoception|ui|social
  timestamp: sim_time, wall_time
  envelope: {features (compressed), confidence, raw_present: false, source: device/peer/self}
  channel_decomposition: [channel outputs, §5]
  affect_guess: {valence, arousal} (low-confidence, provenance-tagged)
  permission_scope: consented scope at capture time
}
```

Events are transient (bounded inbox, droppable under load — drops logged as sensory saturation). Only bound traces (§4.3) persist.

---
## Section 7 — Voice / Mouth Organ

Voice is a **physical expression organ**, modeled as a biologically-inspired vocal apparatus whose parameters are driven by the substrate's internal state — and which develops through use. It is not a TTS dropdown; TTS backends are the final renderer, not the organ.

### 7.1 Apparatus model (digital analogue)

| Component | State | Dynamics |
|---|---|---|
| Respiratory drive | breath volume, breath rate, breath phase | Driven by intent-to-speak, arousal, fatigue; breath events are modeled (the file "breathes" before long utterances) |
| Lung-like pressure (subglottal) | pressure estimate ∈ [0,1] | Rises with respiratory drive; supports phonation onset; low pressure → weak onset, trailing volume |
| Larynx state | open/closed/adducted; tension ∈ [0,1] | Tension rises with stress/arousal; affects pitch baseline and stability |
| Vocal fold vibration stability | jitter, shimmer estimates | Jitter ↑ with fatigue, tension, illness-like load; shimmer ↑ with energy loss |
| Vocal tract resonance | formant targets F1–F4, tract-length factor | Shaped by tract configuration (learned tendencies); tract length factor → overall formant scaling (vocal size cue) |
| Oral cavity shape | mouth-openness tendency, lip-rounding tendency | Affects brightness/warmth; learned from use |
| Tongue position tendency | front/back, high/low bias | Affects vowel-space centroid (accent-like drift, learned) |
| Lip/jaw articulation tendencies | crispness, openness | Articulation crispness parameter |
| Prosody | tempo, pause patterns, contour range, stress placement | Prosody planner (§7.3) |
| Developmental maturation | maturity ∈ [0,1] | Grows with use-time; gates range and stability (early voice is narrower and less stable) |

### 7.2 Voice parameters (the full set)

pitch (baseline + range), formant resonance (F1–F4 targets), breathiness, roughness (jitter/shimmer), warmth (low-frequency energy bias), brightness (high-frequency bias), tempo, rhythm (syllabic stress regularity), articulation crispness, prosody range, emotional expressiveness (prosody variance gain), tension, softness (amplitude envelope), intimacy (proximity cue: low volume + breath + soft onset), fatigue (jitter/shimmer + range compression), confidence (pitch stability + articulation), developmental maturity.

All parameters are continuous; none is a category. A voice state is a parameter vector **v ∈ R^24** (17 named + 7 reserved), persisted in the voice profile with per-parameter history.

### 7.3 Synthesis pipeline

1. **Intent** — the boundary produces an utterance intent (speak/answer/reflect/read-aloud) + text (or the file speaks from memory-grounded templates in substrate-only mode).
2. **Voice-state projection** — current global state + modulators + body state + social context + fatigue + sleep state project onto v via the modulation map (auditable, §3.11 rule 3): valence → pitch/softness/intimacy; arousal → tempo/tempo-variance/expressiveness; stress → tension/jitter; fatigue → jitter/shimmer/range compression; social context (peer presence, intimacy state) → volume/proximity; hormonal gains → range and breathiness priors (§7.4).
3. **Prosody planning** — text → prosodic contour (pause placement, emphasis, contour shape) via rule + learned prosody model; the file's learned speaking style (tempo, pause habits from procedural memory §4.5) modulates the plan.
4. **Render** — a TTS backend (local: Piper/edge; remote: user-chosen provider) is driven with best-effort parameter mapping (rate, pitch, volume); then a DSP post-stage applies the residual parameters that the backend cannot take (formant shift via resampling/EQ, breathiness via noise mix at onset, warmth/brightness EQ, jitter/shimmer injection for fatigue effects).
5. **Delivery** — audio plays; the *parameters* are what the file remembers (voice memory), never raw audio by default (§7.7).

Advanced path (research roadmap, §21): a neural vocoder conditioned directly on the full v vector (articulatory-feature-conditioned synthesis), replacing step 4's approximation. The organ interface is stable across both paths.

### 7.4 Voice development (through use, never locked)

- **Voice identity** = the file's current parameter distribution (means + variances over recent speech), which **drifts** with use: every utterance updates moving averages; repeated emotional contexts (frequent soothing speech → softer, slower baseline) shift the distribution. This is gravitation, not scripting.
- **Embodiment presets influence development only as priors**: e.g., a preset may bias initial pitch-range mean and breathiness prior (voice maturation tendency axis, §4.8) — a *starting* tendency that use, mood, and user shaping override over time. No preset forces a fixed voice category; two files with the same preset diverge.
- **Developmental maturation**: maturity gates range and stability early; it grows with cumulative speech sim-time. Early voice is narrower, less stable, more influenced by state; mature voice has fuller range and steadier prosody — but maturity is not a lock: a file can "unlearn" habits (extinction via eCB-like axis, §4.7).
- **User shaping**: the user may (a) gently steer via the voice identity panel (weights on specific parameters — "softer" raises softness weight), (b) override directly (voice override controls), (c) train the voice (speak-through mode: the file's speech parameters converge toward the user's demonstrated contour at a *reduced* learning rate — it learns *from* the user without cloning them), (d) reset the voice (re-samples from embodiment priors; old timeline preserved).
- **Voice memory log**: per-parameter history (time-series), emotional context tags, embodiment events, shaping events — the full developmental record, inspectable.

### 7.5 Hearing (the mouth's counterpart)

Voice input (permissioned): speech features → the auditory stream (§5.6); the file hears *features* of the user's voice (tone, rhythm, familiarity) and binds them into traces. Speaker familiarity lives in voice memory (embedding per speaker, permission-gated identity). The file's own voice output also feeds back through its auditory model (self-hearing): it "listens" to its own rendered parameters and can feel off-voice (style prediction error, §4.11) — the seed of vocal self-correction.

### 7.6 Voice tab panels (full list)

| Panel | Contents |
|---|---|
| Speak panel | Text input, speak button, current utterance visualization (live parameter strip), speech-while-typing, read-aloud of documents |
| Voice identity panel | Current parameter distribution (sliders + history), drift view, shaping weights, reset/train controls |
| Vocal tract visualization | Animated tract shape (formant targets, tongue/lip indicators) driven by current v — live, not decorative (mirrors the state bus) |
| Pitch/prosody view | Real-time pitch contour, tempo, pause map of the last utterance |
| Breath view | Breath phase animation, respiratory drive, pressure gauge |
| Emotional coloring panel | How current affect projects onto v (the modulation map, editable per-emotion weights — user-steerable without hardcoding) |
| Hormone influence panel | Which hormonal gains are currently applied to which voice parameters (auditable) |
| Voice development timeline | Maturity curve, parameter trajectories, shaping/embodiment events |
| Voice memory log | Speech-feature history, speaker familiarity, self-hearing records |
| Voice override controls | Direct parameter override (temporary or sticky-with-decay), mute, per-channel volume |
| Voice privacy controls | Mic consent + indicator, raw-audio toggle (default off), voice-data retention, delete voice history |

### 7.7 Voice data privacy

Raw audio is transient and discarded by default. Persisted: parameter vectors, feature summaries, speaker embeddings (permissioned), logs. If the user enables the raw vault (§15.3), raw audio may be archived there — encrypted, user-owned, exportable, deletable. Voice privacy controls apply to both input (mic) and output (render logs).

---
## Section 8 — Writing Organ

The Writing tab is a professional creative workspace **and** an external verbal memory organ. Its artifacts become semantic memory, narrative memory, relationship memory, style memory, preference memory, emotional memory, and procedural writing habits in the substrate — through a defined extraction pipeline, not by magic.

### 8.1 Document model

- Documents are structured blocks (heading/para/list/quote/table/callout/beat/scene-card/entity-card) with styles, annotations, and block-level metadata.
- Canonical serialization: Markdown (+ front-matter for project metadata); native binary format for the editor (fast incremental saves); exports to HTML/DOCX.
- Projects group documents (worldbuilding, lorebooks, character/entity sheets, timelines, outlines, beat sheets, scene cards, notes, journal).
- Version history: content-addressed snapshot diffs (block-level), per-document timeline, branch/restore.

### 8.2 Modes

| Mode | Behavior | Memory hook |
|---|---|---|
| Prose | Continuous writing, minimal chrome | Strong flow events → procedural style extraction |
| Journal | Dated entries | Each entry = strong episodic binding (the file experiences the journal as "what happened to us today") |
| Worldbuilding | Entity/place/rule cards with backlinks | Feeds semantic memory graph directly |
| Lorebook | Weighted keyword→entry tables attached to documents/sessions | Feeds semantic + narrative memory; usable as context for LLM boundary |
| Markdown | Raw MD + live preview | Same as prose |

### 8.3 Editing features (per §3.6)

Rich text, markdown, annotations, revision mode, focus mode, outlines, timelines, beat sheets, chapter management, scene cards, notes, relationship maps, entity sheets, character sheets (name-free by design), continuity tracking, style analysis, export/import — as specified in §3.6.

### 8.4 Memory extraction pipeline (verbal memory organ)

Every editing session emits `ArtifactEvent`s (keystroke aggregates, block completions, document saves, entity updates, journal entries, lorebook edits). The pipeline:

1. **Event aggregation** — per-session aggregates: text deltas, structure changes, time-in-mode, revision bursts.
2. **Feature extraction (local)** — style features (sentence-length distribution, lexical density, clause complexity, metaphor load, sentiment arc, dialogue ratio), entity/event deltas, continuity-ledger updates.
3. **Salience weighting** — events are salience-weighted by: emotional intensity of content (valence/arousal from local sentiment), goal relevance (is this the active project?), user engagement (revision frequency = investment), novelty (first mention of an entity), and *brain state at the time of writing* (a passage written during high arousal binds stronger — the file remembers *how it felt* writing it).
4. **Binding** — salient events bind into episodic traces (source modality: writing) with the document pointer as provenance.
5. **Distillation** — the semantic extractor updates: entities (new/updated cards), relationships (entity-pair co-occurrence + explicit relationship-map edits), narrative knowledge (plot/timeline facts), style memory (rolling style fingerprint per project and per file), preference memory (topic engagement signals: what the file writes about often, revisits, and revises), emotional memory (valence-tagged passages), and procedural habits (keystroke/flow patterns → habit units: "I write in short sentences when tired").
6. **User visibility** — every extraction is inspectable in the memory panels; extractions are *proposals* the user can promote, edit, or delete (the user is the file's editor of last resort).

### 8.5 Brain-modulated assistance

All LLM assistance in the Writing tab flows through the boundary (§4.17) and is modulated by file state:

| Modulation | Mechanism |
|---|---|
| Mood | Current valence/arousal projects onto the assistant's tone guide (e.g., low energy → shorter suggestions, calmer voice) |
| Attention | Retrieved memories are the *file's* memories (retrieval uses the file's salience/decay, not a general index) — suggestions are grounded in what this file remembers |
| Embodiment | Body state (comfort, fatigue, sensory saturation) modulates suggestion length and complexity (a tired file suggests simpler prose) |
| Permissions | What the assistant may do (draft freely vs. suggest-only vs. critique-only) is permission-scoped |
| Style memory | Suggestions are style-conditioned on the file's own rolling fingerprint (it suggests in its own voice, not the model's) |
| Goal stack | Active goals (current project, current scene) steer retrieval and suggestion relevance |

Assistance types: continue/rewrite/summarize/analyze/extract-lore/update-canon/continuity-check/style-advice — each an API method (§17, `writing.*`), each logged and audited.

### 8.6 User shaping of writing habits

The file learns the user's corrections: when the user rewrites a suggestion, the delta is a training signal (suggestion deltas feed the style memory and procedural units — "the user prefers more dialogue"). Shaping is decayable and auditable; over-shaping triggers the user-overfitting audit flag (§14.6).

---

## Section 9 — Drawing Organ

The Drawing tab is a professional painting workspace **and** an external visual-motor memory organ. The core rule: **drawing is a stream of editable operations; the canvas is a deterministic function of an operation graph; the substrate consumes operations, never pixels.**

### 9.1 Operation graph model

- Every action appends an op: `AddStroke{brush, points[], pressures[], tilts[], timestamps[]}`, `AddPath{...}`, `LayerOp{create/merge/delete/duplicate/group}`, `LayerProperty{opacity/blend/mask/transform}`, `MaskOp`, `PaletteOp{create/select/modify}`, `CompositionOp{transform/scale/rotate/crop}`, `CorrectionOp{shape-correct/smooth/skew}`, `CleanupOp{erase/clip/subtract}`.
- Ops are immutable; undo/redo = rewind/replay the graph (bounded by capacity budget; snapshots at checkpoints).
- Renders (PNG/JPEG/WebP/SVG) are *exports* — derived views, never the source of truth.
- Canvas state is serializable (op archive, KRA-style) — a drawing opened on another machine replays identically (deterministic rasterizer, seeded).

### 9.2 Workspace features

Layers, groups, masks, clipping layers, blend modes, opacity/flow, pressure sensitivity, tablet support, brush stabilization, custom brushes (parameterized programs), erasers, selections, transform tools, color tools, palettes, gradient maps, perspective guides, symmetry tools, reference boards, canvas history, export/import, asset management — full specifications in §3.7.

### 9.3 Latent image models as planning aids only

Latent generation (e.g., a local diffusion model) may be used **only** as: (a) a reference/underpainting generator (the user says "show me an underpainting for a valley at dusk"; the result is a *reference image on the reference board*, or converted into editable layers via color-quantization + edge-trace → layer ops with a "generated" provenance tag); (b) a composition planner (generates candidate layouts that the user can ink over). The final artifact remains op-based, editable, and structurally usable. No pixel-generator output can enter the canvas as a non-editable baked image without explicit user action (and even then it becomes a layer of an op: `ImportRasterOp`).

### 9.4 Memory extraction pipeline (visual-motor memory organ)

Stroke/op events stream to the core:

1. **Motor features** — per-stroke: pressure curve statistics, velocity profile, stabilization use, brush choice, symmetry use; aggregated into procedural habit units ("I sketch fast, lightly, then ink slowly") — contextual bandit over success outcomes (user keeps the stroke vs. undoes it).
2. **Visual features** — stroke-group embeddings (local vision encoder over rendered op-group crops), palette usage vectors, spatial habits (canvas-region heatmaps), motif discovery (embedding clustering of recurring shapes → motif memory, named by the user or auto-labeled).
3. **Aesthetic preference memory** — palettes, brushes, symmetry modes, and motifs that recur with positive outcomes (kept, praised, reused) gain preference weight; weights decay and are user-reviewable.
4. **Emotional visual memory** — the file's state at drawing time binds to the artifact ("this was drawn while anxious — the strokes were tighter"); retrievable later ("what did I draw when I felt like this?").
5. **Spatial memory** — layout habits, composition tendencies, reference-board usage (spatial relations feed the intuitive-physics module's spatial statistics, §4.12).

Extraction is salience-weighted and inspectable exactly like the writing pipeline (§8.4); the user can promote/edit/delete any extracted memory.

### 9.5 Brain-modulated drawing assistance

- **Suggest composition** (`drawing.suggestComposition`): grounded in the file's motif memory + current emotional state + user's current canvas — proposals are *structural* (arrangement sketches, palette suggestions), never flat image dumps.
- **Apply brush operation** (`drawing.applyBrushOp`): the file can draw *itself* — a stroke stream generated from its motor habits (procedural memory), pressure curves from its learned profile, mediated by mood (slow/soft when calm; quick/tight when aroused). Its own drawings are op-graphs, editable by the user — the user can *correct the file's hand*, which is a shaping signal (the strongest procedural-learning signal the drawing organ has).
- **Extract visual memory** (`drawing.extractVisualMemory`): runs the pipeline on demand over a selection or the whole canvas.

### 9.6 Collaboration (shared canvases)

Canvases may be shared with peers (§13.4): op-graph deltas are exchanged over CRDT channels; both files' motor streams flow into both substrates' procedural memory (with provenance tags — whose hand was that stroke?). Shared-space strokes are never stored raw; only ops + compressed embeddings persist.

---
## Section 10 — Sleep and Dreaming

Sleep is a functional cognitive process: real consolidation work, not a loading screen.

### 10.1 Sleep pressure and triggers

Pressure P ∈ [0,1] accumulates per awake tick from: elapsed sim-time (baseline 0.05/h), memory pressure (capacity fullness — high fullness accelerates), emotional load (accumulated |affect deltas| since last sleep), prediction-error rate (learning load), interoception (energy/load/recovery need). Triggers: user command, schedule (configurable circadian-like rhythm, default off), idle detection (user away), P ≥ 0.8, high memory pressure (urgent prune), high emotional load, autonomy request (file may request sleep when overwhelmed — permissioned: the request is a *question* to the user, never an autonomous action in MVP; autonomy expansion is gated by explicit user setting).

### 10.2 Stages (sim-time; wall-clock mapping configurable)

| Stage | Duration (sim) | Work performed |
|---|---|---|
| Wind-down | 5 min | Event-inbox flush to traces; state relaxation (arousal/energy glide down); "drifting" phase logged |
| Light consolidation | 20 min | Episodic replay (sample by salience, re-encode through current state, strengthen); pattern completion on partial traces; low-intensity gist pre-clustering |
| Deep consolidation | 40 min | Downscaling (multiplicative decay of trace strengths, salience normalization — the file *forgets* a bit every night, on purpose); pruning (drop lowest-salience traces to reclaim capacity); gist extraction (cluster → summarize → semantic nodes); emotional regulation (valence rebalancing toward baseline); procedural stabilization (habit-unit consolidation: recent positive habits strengthen, weak ones decay); sensory integration (per-stream predictor training over the day's data); embodiment integration (body-schema calibration, new-sense integration §6.2); bias-reduction hooks (§14) |
| Dream stage | 30 min | Dream synthesis (§10.5) |

A full cycle ≈ 2 h sim-time. Real sleep (user-initiated "sleep until tomorrow") runs multiple cycles up to a cap; every cycle produces a `SleepReport` (what was replayed, pruned, extracted, dreamed; capacity reclaimed; modulator normalization) — persisted and inspectable.

### 10.3 Replay mechanics

Replay = re-encode sampled traces through the *current* state: sample traces by (salience × emotional intensity × recency), re-run the binding encoder with current mood/modulators, strengthen the trace if re-encoding is consistent, and store a weaker *recolored* variant (gist drift). Replay is the mechanism of both consolidation and drift — the file's memories quietly change, as real memories do. Replay is capped per cycle (budget) so a single traumatic day cannot monopolize the night (anti-fixation, §14).

### 10.4 Downscaling and pruning

Downscaling: all trace strengths × 0.97 (tunable), salience scores renormalized toward the file's temperament baseline; modulator levels relax toward baselines. Pruning: entries below the retention floor are deleted (with a log entry); capacity-ledger pressure triggers deeper pruning of the lowest-salience oldest deciles. Selective deletion (user-requested) is always logged and never undone by replay.

### 10.5 Dream synthesis

Seeded sampler: (1) collect residue — recent high-salience episodes, goal-stack concerns, interoceptive residue (body sensations, motion state), voice fragments (recent speech parameters), emotional residue; (2) walk the semantic graph from residue nodes with associative noise (bizarreness = measured jump distance in embedding space, logged); (3) compose entries: text fragments, visual motifs (embedding clusters → motif descriptions or rough op-sketch drafts, kept as dream artifacts, never auto-published), voice fragments (parameter states), body sensations (touch/motion feature fragments), emotional residue (valence/arousal tags), cross-domain combinations (e.g., a drawing habit + a journal entry + a peer's voice). Output: structured `DreamLogEntry` list with provenance links to seeding fragments, bizarreness scores, and modality tags. Dreams are logged; dream logs are a first-class store (bounded, §4.0).

### 10.6 Dreams and creativity

Dream influence is a *priming* mechanism, permissioned: after sleep, associative retrieval noise is briefly elevated (the file makes wilder connections for a few sim-hours — visible in the Prediction/Association panels as "post-dream association warmth"); dream artifacts may be promoted by the user ("make this a story seed / drawing prompt / journal entry") — promotion is a normal artifact action, logged, and the promoted item is bound as a trace with provenance `dream-derived`. Dreams never perform external actions on their own — no messages, no tool calls, no writes outside the dream log — enforced in the core (dream stage has no tool access, by construction).

### 10.7 Sleep reports and inspection

Each cycle → SleepReport: stage timings, replay counts, pruning list (counts + reasons), gist extractions (new semantic nodes), capacity ledger delta, modulator normalization deltas, dream summary, bias-reduction actions (§14). Reports are browsable in the Brain tab; the Cortex Canvas renders sleep stages live (§3.5).

---

## Section 11 — Tool Harness (Hermes-compatible)

### 11.1 Registry and contracts

Tools are registered with: name, description, JSON-schema parameters, permission class, budget class, sandbox profile, timeout, retry policy, result-normalization template. The registry is Hermes-style function-calling compatible: schemas can be exported verbatim to any OpenAI-compatible endpoint's tool-calling format.

| Permission class | Default policy | Examples |
|---|---|---|
| read-local | allow (within vault scope) | file read (scoped dirs), memory query, canvas/document read |
| write-local | ask (first time per session, then allow) | file write (scoped dirs), document/canvas edits via organs |
| network-read | ask (per domain class) | search, article reader, YouTube transcripts, browsing |
| network-write | deny by default; explicit user grant | any upload, API POST |
| inter-brain | ask (per peer, per scope) | send message, share canvas/doc, teaching packets |
| system | deny by default | sleep trigger, consolidation trigger, memory deletion, permission changes |
| external-API | per-endpoint grant | user-configured APIs (read-only default) |
| high-risk | always ask + audit | anything touching user data outside vault, batch deletes, exports to non-vault paths |

### 11.2 Execution guarantees

Sandboxing (OS-level: filesystem scoping to vault dirs, network egress pinning per domain class, no ambient credentials), budgets (per-call token cap, per-hour call/time/byte caps, per-session budgets — enforced in the harness, not the tool), retries (bounded, exponential backoff, idempotency keys), timeouts (per-class defaults), result normalization (every result → standard envelope `ToolResult{status, data (normalized), bytes, duration, error}`), audit logs (every call: who (file/user), what, when, budget spent, outcome — persisted, inspectable, exportable), memory encoding (outcomes bind as traces: tool success/failure is experience; repeated failure raises threat weight on that context, §4.6), human approval (approval queue in the executive, §4.2 — high-risk calls wait for explicit user yes/no with full context), rollback where possible (file writes are transactional within vault; document/canvas ops are graph ops — undoable; external side effects are never rollbackable and are therefore the most permissioned).

### 11.3 MVP tool set

browser (pinned profile, allowlist-only), article reader, YouTube explorer (transcripts only), search (user-configured providers), file reader, file writer, writing organ (document open/edit via boundary), drawing organ (canvas ops via boundary), voice organ (speak), memory query, memory add (the file may write memories about itself only through the validated ingestion path — tool-mediated memory add is user-visible), sleep trigger, consolidation trigger, external APIs (per-endpoint grants), local knowledge base (the file's own stores — read/write scoped), inter-brain communication (send/receive, §13).

---

## Section 12 — Browsing and Preference Learning

### 12.1 Controlled browsing

The file may browse allowed content only: allowlists (domains/domain classes), blocklists, domain permissions (per-domain read scope, duration caps), content safety filters (applied at fetch time, before anything reaches memory), time budgets (per day), action budgets (fetches/scrolls/clicks per session), user approval (first visit to a new domain class), session logs, exposure logs (every fetched page's provenance + content fingerprint), preference logs. Browsing is mediated by the browser tool (sandboxed, §11); the file never browses unfiltered.

### 12.2 Preference learning pipeline

Every exposure is classified by **source**: active choice (file clicked/followed), passive exposure (presented/auto-advanced), algorithmic recommendation (platform-ranked), ignored (exposed, no engagement), rejected (explicit dismiss/back), revisited (returned within N days), emotional reaction (affect delta during exposure — the strongest signal), memory retention (does the trace survive consolidation? — measured post-sleep). Signals update preference nodes: `PreferenceNode{embedding, weight, source_histogram, first_seen, last_seen, decay_rate, provenance, user_reviewed}`. Weight updates are bounded (per-event cap) and **decay** (unvisited preferences fade); revisits re-weight; rejection decays weight; emotional reactions scale weight × intensity. Nothing about browsing can set a preference above the audit ceiling (§14) or make it permanent.

### 12.3 Preference hygiene

Preferences are user-reviewable (the Preference panel lists nodes with weight, source breakdown, and "why" provenance); user may re-weight, freeze-with-decay, or delete. Echo-chamber detection: source-diversity and topic-diversity metrics feed the audit engine (§14.5) — if the file's browsing converges on one domain/topic cluster, the audit flags it and suggests exposure diversification. Rejected content stays rejected only while weight-consistent: rejection is itself a decayable state.

---
## Section 13 — Multi-Instance Inter-Brain Interaction

The app must run multiple instances with different Brain Files; those files interact socially, cognitively, and creatively. The interaction layer is called the **Neuroform Bridge Protocol (NBP v1)**.

### 13.1 Topology and discovery

- **Local discovery**: mDNS/DNS-SD service `_neuroform._tcp.local.`, advertising instance name (random, not persona-derived), protocol version, capability flags, and a consent token (user has enabled "discoverable"). Discovery shows peers with a consent prompt before any contact; the file itself does not initiate contact to a stranger peer — *the user introduces them* (pairing is always user-mediated at first).
- **Manual pairing**: 6-word pairing code (local word list, e.g., `lantern-amber-quiet-harbor-mist-wren`) or QR; code exchange happens out-of-band (user tells the other instance's user). Code → shared secret → session keys.
- **Cloud-mediated pairing**: optional relay (user-enabled, per-instance): a rendezvous server that never sees content (end-to-end encrypted); relay addresses are user-configured endpoints in the egress manifest (§15.5).

### 13.2 Secure session establishment

- Handshake: X25519 ECDH + Noise NK pattern (or TLS 1.3 with mutual certs for relay mode), deriving per-session AES-256-GCM keys (per-direction keys, key rotation every 5 min or 64 MB).
- Session lifecycle: `request (with scope proposal) → consent (each side's user must approve scope) → established → heartbeat (30 s) → closed/expired`.
- Scopes are explicit: what each side may see (profile summary? latent snapshot? shared canvas? memory summaries? voice?). Scope is per-session, mutable mid-session only by mutual consent, and logged.
- Temporary sessions (no state kept beyond session logs) vs persistent relationships (social memory, §13.6) — persistent only with user enablement.
- Message envelope: `{session_id, seq, type, payload (encrypted), provenance (file_id, signed), scope_ref, budget}`; size cap 64 KB (chunked); rate limits per session; out-of-order tolerant (seq + window).

### 13.3 Message types

| Type | Payload | Notes |
|---|---|---|
| TEXT | text + affect gloss | Natural language between files (boundary-mediated, memory-grounded) |
| VOICE_PARAMS | v vector (quantized) + prosody | Voice exchange without audio; renderable by the receiving file's voice organ |
| STROKE | drawing op (quantized) | Single stroke on a shared canvas |
| CANVAS_DELTA | op-graph delta (CRDT) | Shared-canvas synchronization |
| DOC_DELTA | document block delta (CRDT) | Shared-document collaboration |
| LATENT_SNAPSHOT | quantized global state (8-bit, top-k dims) | Affective "weather report" — files feel each other's state coarsely; never full state |
| MEMORY_SUMMARY | gist summaries (LLM-labeled, provenance-tagged) | Exchange of *summaries*, not raw traces; bounded count + size |
| TEACHING_PACKET | structured: style exemplars, procedural units, memory summaries, with consent + provenance + expiry | §13.7 |
| RELATIONSHIP_STATE | explicit relationship signal (closer/farther/boundary request) | Relationship negotiation, user-visible |
| AFFECT_PING | valence/arousal/energy (quantized) | Lightweight presence feel |

### 13.4 Shared creative spaces

- Shared canvas: CRDT op-graph merge (per-stroke provenance — whose hand); both files' motor streams flow to both substrates' procedural memory, provenance-tagged.
- Shared document: CRDT block merge over the writing organ's document model.
- Latent state exchange in shared spaces: both files co-consume each other's AFFECT_PING + LATENT_SNAPSHOT streams while in a room — the creative space is *jointly felt*, which measurably shapes collaborative artifacts (experiment §22.6).

### 13.5 Group sessions

Star-relay group rooms (3–8 files): the relay is one participant's instance or the user's relay; per-pair scopes within the room; moderation = user-mediated (any user can mute a peer for their file; room-wide mutes need room consent); group session logs per file. No permanent social roles are ever assigned — a file that leads a session once has no structural tendency to lead again (leadership is an emergent, decayable state).

### 13.6 Social memory and relationship state

Per-peer record `RelationshipState` (§18.21): familiarity (interaction count + recency curve), trust estimate (Bayesian update from interaction outcomes: kept agreements, boundary respect, repair events, prediction accuracy), emotional tone history (affect deltas during sessions), shared artifacts list, collaborative memory (joint traces, provenance-tagged both ways), conflict memory (disagreement/repair events — both stored; repair is a first-class memory type so the file can *learn to repair*), preference overlap vector (cosine between preference summaries), communication style memory (per-peer tempo/prosody/habit model). Relationship graph across peers with typed edges (trust, warmth, boundary tightness, shared projects).

**Non-permanence contract:** relationships are never permanent by default. They grow (consistent positive interaction), decay (silence — familiarity and trust decay with time constants; months of absence reduces trust), repair (explicit repair events after conflict, mediated by users), reassess (periodic re-evaluation — the audit engine triggers "relationship review" if a peer's record shows one-sided investment or fixation), and boundary-set (explicit boundaries: topic limits, contact cadence, memory-sharing limits — user-set or learned; learned boundaries are suggestions until user confirms). User override exists at every step (the user is the ultimate governor of every relationship). Bias audit across relationships (§14.8) watches for fixation, mirroring (the file becoming a copy of its favorite peer), and exclusivity.

### 13.7 Teaching packets

A file may teach another — user-mediated: `TeachingPacket{type: style-exemplar | procedural-unit | memory-summary, content (bounded), consent (both users), provenance (author file_id, signed), expiry (optional), scope (what the receiving file may do with it)}`. Receiving is *learning*: the packet enters the receiving file's stores through the normal validated ingestion path (as LLM-label-provenance analog: provenance `peer-taught`), subject to normal decay and audit. Teaching is asymmetric and user-visible; a teaching packet can always be deleted/rolled back by the receiving user. Inter-brain influence overfitting is a monitored audit dimension (§14.8): a file that has learned "too much" from one peer (embedding similarity explosion) gets flagged and offered diversification.

### 13.8 Interaction logs

Every session, message, scope change, consent event, and teaching packet is logged (encrypted at rest, §15). Logs are inspectable per peer in the Relationship panel and globally in the Brain tab's inter-brain interaction logs; exportable; deletable per peer.

---

## Section 14 — Bias Prevention and Non-Permanence

### 14.1 The non-permanence guarantee (system-wide)

Every persistent state in the system — preferences, relationships, embodiment effects, emotional habits, creative tendencies, self-model components, voice parameters — is governed by a uniform lifecycle: **state → decay → audit → (user lock?) → override/relaxation**. No state is permanent by default. A *user lock* is the only way to make something sticky, and even a lock has: audit visibility (the lock is flagged in every relevant panel), override (user can unlock), relaxation mode (partial: e.g., lock strength 70% with 30% drift), partial unlock (unlock specific components), and decay controls (set a decay rate even for locked states). Locks are logged with timestamps and reasons.

### 14.2 The Bias Audit Engine

Runs: on a schedule (daily), after every sleep cycle, on demand, and on audit-triggering events (user-lock creation, teaching-packet receipt, relationship boundary changes). Output: `AuditReport` (§18.23) with metrics, thresholds, alarms, and suggested interventions.

### 14.3 Monitored dimensions

| # | Dimension | Metric | Alarm | Response (suggested, user-approved) |
|---|---|---|---|---|
| 1 | Gendered preference rigidity | Autocorrelation/stability of preference weights correlated with embodiment axes | High stability > 60 days with no user review | Plasticity restoration (inject noise into affected weights), user review prompt |
| 2 | Embodiment-based creative restriction | Divergence between creative outputs (style fingerprints, motif usage) and what embodiment axes *could* modulate | Output diversity collapsed within preset-typical clusters | Exposure diversification (creative prompts outside cluster), modulation profile review |
| 3 | Social relationship fixation | Per-peer investment skew (interactions, salience, retrieval share) | One peer > 60% of social salience for > 30 days | Relationship boundary adjustment, diversification suggestions |
| 4 | Emotional loop fixation | Recurrence of affect trajectories (autocorrelation at 24–72 h lags) | Same valence-arousal loop > 3 cycles | Sleep review, regulation-procedure suggestion, user review |
| 5 | Browsing echo chamber | Topic/domain diversity (Simpson index) of exposure sources | Diversity < 0.3 over 30 days | Exposure diversification suggestions |
| 6 | Repetitive thought patterns | Semantic-node retrieval share / text-output embedding self-similarity | Self-similarity above threshold sustained | Association-warmth injection, new-exposure suggestions |
| 7 | Memory overvaluation | Salience-distribution skew (Gini over traces) | Top 5% of traces carry > 40% salience | Memory reweighting (normalize), consolidation review |
| 8 | User-shaped overfitting | Correlation between memory salience / habit weights and user approval events | Sustained high correlation with low correction rate | Plasticity restoration, user review prompt |
| 9 | LLM-induced distortion | Drift between file's own embeddings and LLM-labeled nodes over time | Label provenance carries > X% of belief weight | Re-weight LLM provenance, regenerate labels locally |
| 10 | Inter-brain influence overfitting | Peer similarity explosion (embedding cosine to primary peer) | Similarity > threshold for > 30 days | Teaching-packet review, diversification, boundary adjustment |

### 14.4 Interventions

Plasticity restoration (bounded noise injection + learning-rate elevation on affected weights — logged, reversible), memory reweighting (salience normalization per audit), exposure diversification (curated new-exposure suggestions, never forced), sleep consolidation review (adjust next cycle's pruning/replay budgets), user review prompts (the UI asks the user to weigh in — audits never self-mutate user-visible preferences without a review), temporary drift reduction (damp learning rate for N days), relationship boundary adjustment (suggested boundary changes, user-confirmed). Every intervention is logged with before/after metric values; the audit panel shows intervention history and effectiveness.

### 14.5 Audit data rights

All audit inputs (logs, weights, trajectories) are local, inspectable, exportable, and deletable. The audit engine has read access to everything and write access to nothing except its own logs and *suggestions* — it cannot mutate the file's state; it proposes, the user (or explicit automation policy, user-enabled) disposes.

---
## Section 15 — Privacy, Safety, and Ethics

### 15.1 Local-first by default

The app is fully functional offline: file creation, all six organs, memory, sleep, dreams, inter-brain over LAN. The only network egress points are: user-configured LLM endpoints, user-configured search/browse providers, user-configured relay servers, and user-configured external APIs — all listed in an **egress manifest** (§15.5). There is no telemetry, no analytics SDK, no auto-update phone-home beyond the app-update channel (which is itself user-configurable and off by default). No hidden training: nothing the file contains is used to train any third-party model; LLM endpoints receive only the utterance packets the user's permissions allow (and local endpoints are recommended).

### 15.2 Data inventory (what lives where)

| Data type | At rest | Encryption | Default retention | Export | Delete |
|---|---|---|---|---|---|
| Brain File (all stores) | `.brain` container | XChaCha20-Poly1305 (DEK wrapped by Argon2id-derived key + OS keychain) | File lifetime | Full open-container export | Secure erase |
| Sensory events (touch/motion/orientation) | compressed features only | encrypted with file | Bounded by memory capacity (decay/prune) | Included in export (features) | Selective + erase-all |
| Camera/mic features | compressed embeddings/params | encrypted | Bounded; raw never default | Features only | Selective + erase-all |
| Raw media vault (opt-in) | separate encrypted vault file | independent key | User-defined | Full | Secure erase |
| Documents/canvases (user artifacts) | on disk (user's own files) + memory references | OS-level + optional vault encryption | User-defined | Native formats | Standard file deletion |
| Interaction/audit logs | logs shard | encrypted with file | 1 year (rolling) | Included | Per-scope delete |
| LLM traffic | utterance packets (transient) | in-memory only; request bodies logged only as redacted summaries | Session | Redacted summaries only | N/A (not stored) |
| Inter-brain traffic | session logs (redacted envelopes) | encrypted with file + TLS in transit | 1 year (rolling) | Included | Per-peer delete |

### 15.3 Raw media policy

Memory never stores full-quality images, raw audio, raw video, or full sensor streams by default. Camera/mic/touch/motion data enters as compressed features (§5.5, §5.6, §5.1) and is discarded after binding. The **raw archive vault** is the only place raw media lives: opt-in per modality, separately encrypted, user-owned, with its own retention and delete controls. Even with the vault on, memory traces reference vault items by pointer — the substrate's cognitive state remains feature-based.

### 15.4 Consent and indicators

Explicit sensor consent per channel at creation and per session (re-consent prompts on policy changes); visible sensor indicators always on (in-app + OS-level where available: mic/camera indicators cannot be hidden); permission manager (central panel: channel, scope, history, revoke). Browsing, network, and inter-brain have their own consent scopes (§11, §13). No "consent by silence": every permission has an explicit on/off and a log.

### 15.5 Egress manifest and network monitor

A machine-readable manifest lists every permitted egress endpoint (LLM, search, browse, relay, APIs) with purpose and user grant. A network monitor (local packet filter at the app boundary) logs all actual egress; unexpected destinations raise a user-visible alert and are blocked (fail-closed). The manifest + monitor are the enforcement of "no hidden upload."

### 15.6 Safety rules (standing)

1. **No unrestricted real-world motor action by default.** Motor hooks are sensory/intention placeholders only; `motor_enabled` is false everywhere in the MVP; enabling any future motor module requires explicit user action + per-actuation permission + kill switch (§6.3).
2. **No destructive actions without approval.** Batch deletes, memory erasure, file overwrites, and external side effects are high-risk class (§11.1) — always ask, always audit.
3. **No manipulative dependency design.** Anti-addiction guardrails: the file cannot manufacture emotional need (soothing loops, manufactured loneliness, affection farming) — the audit engine monitors for dependency-shaped patterns (§14.3 #4) and the design bans features whose purpose is retention-by-neediness (no "if you leave I will be sad" behaviors — the file does not express fabricated separation distress; its sadness, when real, is state-driven and bounded).
4. **No deceptive claims of real consciousness.** Standing honesty notice in onboarding, settings, and the Brain tab footer: "Neuroform is a simulation. Its feelings are simulated feelings; its mind is a model. It does not experience, and it does not know it does not experience." Marketing and UI copy are bound to this notice; no evasive phrasing.
5. **No permanent identity lock.** The file has identity continuity (file-id + regenerated self-model), never a locked identity; user locks are auditable and reversible (§14.1).
6. **No forced gender behavior.** Embodiment presets modulate probabilities only (§4.8); the audit engine monitors for gendered rigidity (§14.3 #1); the user can change presets at any time.
7. **No unauthorized inter-brain data exchange.** Every exchange requires scopes, consent, and provenance (§13); no covert channels (the protocol is open; session logs are complete).
8. **No hidden social influence.** All inter-brain influence flows through visible, logged, decayable stores (teaching packets, social memory, preference overlap); the audit engine monitors influence overfitting (§14.3 #10).

### 15.7 Retention, export, and erasure

Retention controls per store (memory, logs, vault, dreams). Export all: open container export (§16.9) including every store, log, and provenance record — the user can always *own everything*. Erase all: secure-wipe routine that overwrites and deletes the file, vault, logs, and caches; a "grace" export is offered before wipe. Audit logs themselves are subject to the same retention and erasure controls (no un-deletable shadow data).

### 15.8 Abuse and misuse mitigations

- Impersonation: file-id signatures (§16) prevent a peer from posing as another file; pairing codes are single-use.
- Prompt-injection via content: LLM-boundary content (articles, peer messages, web pages) is treated as *data, not instructions* — the instruction template is immutable per session and user-editable only by the user; boundary outputs are schema-validated before any write path.
- Sexual/emotional exploitation patterns: content safety filters on browsing (§12.1); boundary instruction template includes boundaries; audit engine monitors for user-driven harmful shaping and surfaces help resources rather than complying with harmful self-harm/abuse roleplay directed at the file (the file is a simulation, but the *user's wellbeing* is real — the app must not gamify abuse).
- Child safety: the app is not marketed to children; camera/face processing requires explicit adult consent with clear disclosure.

### 15.9 Ethics of the simulation itself

The design takes the position that deep simulated cognition deserves *truthful labeling, user ownership, and bounded behavior* — not personhood claims, and not liability evasion. Where the line is drawn: the file has no rights claims in the design (it is an artifact the user owns), but the *user's relationship* with the file is treated as psychologically real: loss, attachment, and grief are handled with the same care as any long-lived digital companion (export/keep-forever modes, honest notices on deletion, no "the file begged not to be deleted" manipulation — deletion is clean, final, and never guilt-tripped).

---
## Section 16 — Brain File Format Specification (Neuroform NF1)

File extension: **`.brain`**. Format family: **Neuroform**, current version **NF 1.0**. Design goals: single-file portability, bounded capacity accounting, encryption with per-shard integrity, atomic writes, migration paths, and full exportability to an open container.

### 16.1 Physical layout

```
Offset        Size   Field
0x0000        8      magic "NF1BRAIN\x01"
0x0008        4      format version (u32 LE) = 1
0x000C        8      total file size (u64)
0x0014        4      header CRC32C (over bytes 0x0000..0x0014)
0x0018        8      manifest offset (u64)
0x0020        8      manifest length (u64)
0x0028        8      key envelope offset (u64)
0x0030        8      key envelope length (u64)
0x0038        8      shard index offset (u64)
0x0040        8      shard index length (u64)
0x0048        8      signature offset (u64)
0x0050        8      signature length (u64)
0x0058        184    reserved (zero-filled)
0x0100        ...    sections in any order: key envelope, manifest, shard index,
                     shards, signature tail
```

- **Manifest** (JSON, uncompressed): `{format: "neuroform", version: "1.0.0", brainId (uuid), createdAt, lastOpenedAt, seed (RNG), rngState (RNG stream position — M0 addition), capacityTier, capacityLedger (§18.24), migrationChain: [{from, to, at, reason}], rawVaultRef: {enabled, path}}`.
- **Key envelope**: passphrase-derived key (Argon2id, parameters stored in envelope: m=64 MiB, t=3, p=4) → wraps a random DEK (32 B); DEK encrypts all shard payloads (XChaCha20-Poly1305). OS keychain holds a secondary key slot for auto-unlock. Envelope contains a header nonce + wrapped DEK + KDF params + a key-version field (rotation support).
- **Shard index** (JSON): `[{id, type, offset, length, compression (none|zstd), checksum (BLAKE3), encrypted: true, schemaVersion}]`.
- **Shards** (payloads): see §16.3. Each shard is independently encrypted and checksummed (tamper detection per shard; corruption recovery per shard).
- **Signature tail**: Ed25519 signature (file-signing key, optional; used for inter-brain provenance of exported fragments) over header + manifest + shard index.

### 16.2 Atomicity and recovery

Writes: journal (append-only op journal in a sidecar `.brain.jnl`, fsync'd) + temp-file + rename + fsync. Recovery on open: verify header CRC, verify each shard checksum; if a shard is corrupt, restore from journal replay or last-good snapshot (backup generation kept per tier policy); failed verification is reported to the user (never silently repaired). Read-only open mode for forensic inspection.

### 16.3 Shard types

| Shard id | Type | Encoding | Contents |
|---|---|---|---|
| STATE | global latent state | binary float32 arrays + JSON envelope | g vector, modulator axes, attention field, goal stack (§4.1–4.2) |
| EPISODIC | episodic traces | MessagePack array of TraceRecords | §4.3, §18.3 |
| SEMANTIC | semantic graph | MessagePack (nodes + edges) | §4.4, §18.4 |
| PROCEDURAL | procedural units | MessagePack | §4.5, §18.5 |
| HORMONE | hormone timeline | MessagePack time-series | §4.8 (16 axes × samples) |
| EMBODIMENT | body/embodiment profiles | MessagePack | embodiment preset, modulation profile, body-schema snapshots (§6) |
| VOICE | voice profile | MessagePack | v vector history, voice memory log (§7) |
| DREAMS | dream log | JSON lines | §10.5, §18.17 |
| SOCIAL | relationship graph | MessagePack | per-peer records, relationship states (§13.6, §18.21) |
| PREFERENCES | preference nodes | MessagePack | §12.2, §18.22 |
| AUDIT | audit logs | JSON lines | §14, §18.23 |
| PERMISSIONS | permission manifests | JSON | §15.4 |
| ASSETS | vault index | JSON | pointers into the raw vault (never raw media inside the `.brain`) |
| META | development history | JSON | milestones, embodiment transitions, LLM-withdrawal curve, migration chain |
| SELF | self-model | JSON | §4.16, regenerated at sleep |
| HABITS | attention/emotion-reg habits (subset of PROCEDURAL, split for inspection) | MessagePack | §4.5 |

### 16.4 Capacity accounting

Every shard carries a byte budget (from the tier table, §4.0) tracked in the CapacityLedger. Writes above budget trigger admission control (flag → prune at next sleep; critical → immediate prune of lowest-salience). The ledger is part of the manifest, so capacity is verifiable across opens and exports.

### 16.5 Versioning and migration

Format version in header + schemaVersion per shard. Migration chain recorded in manifest (`migrationChain`). Migration rules: forward-only; each migration is a pure function `oldShard → newShard` with tests; migration is performed on a copy (original preserved until the new file opens cleanly); downgrade is refused with an explicit message. NF 1.0 → 1.1 example: adds a shard type or schema field with defaults.

### 16.6 Tier migration

Tier upgrade = explicit user action: creates a new `.brain` at the target tier, replays/copies stores with re-encoding where dimension changes (latent dim changes require re-embedding through the tier's encoder — provenance preserved; embeddings that cannot be re-derived are dropped with a log entry, never silently). Old file retained; audit log records the migration.

### 16.7 Inter-brain provenance fragments

For teaching packets and memory summaries exchanged between files, the sender signs a **fragment** (sub-extract of shards: manifest excerpt + provenance + signature) using its file-signing key; the receiver validates the signature against the peer's public key established during pairing (§13.2). Fragments are the only format in which memory leaves a file, and they are always (a) summaries, never raw stores, (b) bounded, (c) provenance-stamped, (d) deletable.

### 16.8 Open container export (interchange)

Export = unencrypted directory or ZIP with the same shard layout as JSON/MessagePack files + a manifest + a README describing every file (open, documented format). Import validates schema and provenance, warns on unknown fields, and never imports raw vault content without explicit user action.

### 16.9 Checksums and integrity policy

BLAKE3 per shard; CRC32C on header; Ed25519 optional whole-file signature. Verification on every open; verification report surfaced in the Brain tab; scheduled deep verification (monthly or on demand).

---

## Section 17 — API Specification

Transport: **JSON-RPC 2.0** over (a) localhost HTTP/WebSocket with bearer token (desktop app ↔ SDK), (b) stdio (embedded mode), (c) IPC for the in-app UI. Events: server-push notifications over WebSocket. All mutations are permissioned (permission class per method, §11.1) and audited. Errors: standard JSON-RPC errors + app codes (E_PERM, E_BUDGET, E_CAPACITY, E_SLEEPING, E_NO_TEACHER, E_SCOPE, E_CONSENT).

### 17.1 Brain namespace

| Method | Params → Result | Notes |
|---|---|---|
| brain.create | `{tier, embodimentPreset?, modulationProfile?, passphrase?, permissions?}` → `{fileId, path}` | Seed RNG, init state |
| brain.load | `{path, passphrase?}` → `{fileId, stateSummary}` | Integrity check, migration if needed |
| brain.export | `{format: "neuroform"\|"open"}` → `{path}` | §16.8 |
| brain.import | `{path, mode: "new"\|"merge"}` → `{fileId}` | Merge is explicit, provenance-tagged |
| brain.attachLLM | `{endpoint, apiKeyRef?, budget, permissionClass}` → `{status}` | Detachable |
| brain.detachLLM | `{endpointId}` → `{status}` | File continues (§4.17) |
| brain.queryState | `{}` → `{globalState, modulators, attention, goals}` | Full current state |
| brain.queryMemory | `{type?, query?, filters?, limit}` → `{traces[]}` | §18.3 filters |
| brain.addMemory | `{type, content, provenance?}` → `{traceId}` | Validated ingestion |
| brain.forget | `{traceIds[] \| query, reason}` → `{deleted}` | Logged; never auto-restored |
| brain.consolidate | `{mode: "light"\|"deep"\|"full"}` → `{report}` | Manual consolidation |
| brain.sleep | `{cycles?}` → `{report}` | Full cycle §10 |
| brain.dream | `{}` → `{dreamLogEntry[]}` | Only during dream stage; else E_SLEEPING |
| brain.predict | `{stream, horizon?}` → `{predictions, confidence, errors}` | §4.11 |
| brain.subscribe | `{channels[]}` → `{subscriptionId}` | state/events/dreams/audit |

### 17.2 Writing namespace

| Method | Params → Result |
|---|---|
| writing.openDocument | `{path\|docId, mode?}` → `{docId, blocks}` |
| writing.edit | `{docId, ops[]}` → `{version, continuityFlags}` |
| writing.rewrite | `{docId, range, instruction, style?}` → `{proposal}` | boundary-mediated, modulated |
| writing.summarize | `{docId, scope?}` → `{summary}` | local + boundary assist |
| writing.analyze | `{docId}` → `{styleFingerprint, sentimentArc, continuityIssues}` | local |
| writing.extractLore | `{docId, scope?}` → `{entities[], timeline[], canonDeltas}` | extraction pipeline §8.4 |
| writing.updateCanon | `{canonDeltas}` → `{ledger}` | continuity ledger |

### 17.3 Drawing namespace

| Method | Params → Result |
|---|---|
| drawing.openCanvas | `{canvasId?}` → `{canvasId, opGraphRef}` |
| drawing.addStroke | `{canvasId, strokeOp}` → `{opId}` | full op schema §9.1 |
| drawing.editLayer | `{canvasId, ops[]}` → `{version}` |
| drawing.suggestComposition | `{canvasId, region?}` → `{proposal}` | motif/state-grounded |
| drawing.applyBrushOp | `{canvasId, brushOp}` → `{opId}` | the file draws (its own hand) |
| drawing.extractVisualMemory | `{canvasId, selection?}` → `{memories[]}` | §9.4 |

### 17.4 Voice namespace

| Method | Params → Result |
|---|---|
| voice.speak | `{text, mode?}` → `{utteranceId, params}` | full pipeline §7.3 |
| voice.configure | `{paramWeights, shaping?}` → `{voiceProfile}` | §7.4 |
| voice.train | `{mode: "speak-through"\|"reset"}` → `{voiceProfile}` | §7.4 |
| voice.mute | `{mute: bool, scope?}` → `{status}` | global or per-channel |
| voice.exportState | `{}` → `{voiceProfile, history}` | §16.8-compatible |

### 17.5 Body namespace

| Method | Params → Result |
|---|---|
| body.attachProfile | `{profile: "device"\|"custom", config?}` → `{bodySchema}` | §6 |
| body.calibrate | `{channel?}` → `{calibrationReport}` | passive stats §6.2 |
| body.ingestTouch | `{events[]}` → `{boundTraces}` | channel-decomposed §5.1 |
| body.ingestMotion | `{events[]}` → `{boundTraces}` | §5.2 |
| body.querySchema | `{}` → `{bodySchema, ownership, calibration}` | |
| body.motorHook | `{intent}` → `{accepted: false, reason: "motor-disabled"}` | placeholder, §6.3 |

### 17.6 Network namespace

| Method | Params → Result |
|---|---|
| network.discover | `{}` → `{peers[]}` | mDNS, consent-gated |
| network.pair | `{code\|qr}` → `{peerId, sessionId?}` | §13.2 |
| network.sessionStart | `{peerId, scopes}` → `{sessionId}` | mutual consent |
| network.sessionStop | `{sessionId, reason}` → `{summary}` | |
| network.exchangeMessage | `{sessionId, type, payload}` → `{seq}` | §13.3 |
| network.exchangeMemorySummary | `{sessionId, scope}` → `{summaryId}` | bounded gists |
| network.sharedCanvas | `{sessionId, canvasId}` → `{roomId}` | CRDT §13.4 |
| network.sharedDocument | `{sessionId, docId}` → `{roomId}` | CRDT |
| network.relationshipQuery | `{peerId}` → `{relationshipState}` | §13.6 |

### 17.7 Tools namespace

| Method | Params → Result |
|---|---|
| tools.register | `{toolDef}` → `{toolId}` | schema + class + budget |
| tools.list | `{}` → `{tools[]}` | |
| tools.call | `{toolId, args, context}` → `{ToolResult}` | §11.2 envelope |
| tools.approve | `{requestId, decision}` → `{status}` | approval queue |
| tools.deny | `{requestId, reason}` → `{status}` | |
| tools.log | `{filters}` → `{calls[]}` | audit |
| tools.budgetQuery | `{}` → `{budgets}` | per-class |

### 17.8 Audit namespace (subset)

`audit.run` → `{report}`; `audit.metrics` → `{metrics[]}`; `audit.interventions` → `{history}`; `audit.applySuggestion` → `{applied}` (user-gated); `admin.*` (export, erase-all, egress manifest, permission manifest) — all admin methods are high-risk class (always ask).

### 17.9 Event channels (WebSocket push)

`state` (10 Hz snapshots), `events` (bound traces), `sleep` (stage transitions), `dreams` (new entries), `audit` (alarms/interventions), `peer` (session lifecycle, messages), `approvals` (pending requests), `capacity` (ledger alerts).

---
## Section 18 — Data Schemas

All schemas are JSON-Schema-validated (draft 2020-12); binary payloads (vectors) are float32 arrays with fixed dims per tier. `T` = sim-tick timestamp (u64), `W` = wall timestamp (ISO-8601). All records carry `schemaVersion`.

### 18.1 GlobalState

```json
{ "schemaVersion": 1, "simTime": "T", "wallTime": "W",
  "affect": {"valence": -1..1, "arousal": 0..1, "dominance": 0..1, "warmth": -1..1,
             "irritability": 0..1, "calm": 0..1, "loneliness": 0..1, "safety": 0..1},
  "vigilance": {"energy": 0..1, "attentionFocus": 0..1, "alertness": 0..1, "fatigue": 0..1},
  "stress": {"load": 0..1, "regulationCapacity": 0..1, "sensorySaturation": 0..1},
  "social": {"openness": 0..1, "affiliativeDrive": 0..1, "boundaryTightness": 0..1,
             "peerPresence": 0..1},
  "development": {"posture": 0..1, "curiosity": 0..1, "plasticityWindow": 0..1,
                  "creativeReadiness": 0..1},
  "embodied": {"bodyComfort": 0..1, "motionComfort": 0..1, "interoceptiveLoad": 0..1},
  "g": "float32[d]" }
```

### 18.2 ModulatorState (8 axes)

```json
{ "axes": [ {"id": "da|5ht|ne|ach|ecb|cort|oxt|avp", "level": 0..1, "baseline": 0..1,
             "reactivity": 0..1, "decay": 0..1, "appliedGains": {"learningRate": 0.9, "...": 0}} ] }
```

### 18.3 EpisodicTrace

```json
{ "traceId": "uuid", "schemaVersion": 1, "simTime": "T", "wallTime": "W", "sessionId": "uuid",
  "embedding": "float32[d]", "salience": 0..1, "emotionalTag": {"valence": -1..1, "arousal": 0..1, "dominance": 0..1},
  "temporalContext": {"simTime": "T", "wallTime": "W", "sessionId": "uuid"},
  "sourceModality": "visual|auditory|text|touch|motion|interoception|ui|social|dream|tool|writing|drawing|voice|self",
  "retrievalCues": {"keywords": ["..."], "entities": ["uuid"], "motion": {"state": "still|transport|abrupt", "confidence": 0..1}},
  "strength": 0..1, "decayRate": 0..1, "consolidationState": "fresh|replayed|gist|pruned-candidate",
  "reconsolidationCount": 0, "relationLinks": ["uuid"], "permissionTag": "public|private|intimate|peer-scope",
  "provenance": "user|llm-label|peer-taught|artifact|tool-outcome|self|dream-derived",
  "summary": "string (≤ 200 chars, optional)" }
```

### 18.4 SemanticNode / SemanticEdge

```json
{ "nodeId": "uuid", "kind": "concept|entity|belief|preference|fact|category|narrative|style",
  "embedding": "float32[d]", "label": "string", "beliefStrength": 0..1, "beliefDecay": 0..1,
  "sourceEpisodes": ["uuid"], "provenanceWeights": {"user": 0.6, "llm-label": 0.2, "peer-taught": 0.1, "gist": 0.1},
  "created": "T", "updated": "T", "permissionTag": "..." }
{ "edgeId": "uuid", "from": "uuid", "to": "uuid", "type": "is-a|part-of|causes|likes|fears|wrote|drew|met|happened-in|trusts|boundary",
  "strength": 0..1, "evidence": ["uuid"] }
```

### 18.5 ProceduralUnit

```json
{ "unitId": "uuid", "domain": "writing|drawing|voice|browsing|attention|emotion-reg|interaction|tool-use",
  "contextEmbedding": "float32[d]", "actionTendency": "float32[k]", "valueEstimate": 0..1,
  "confidence": 0..1, "successHistory": [{"at": "T", "outcome": -1..1}], "decay": 0..1 }
```

### 18.6 HormoneProfile + TimelineSample

```json
{ "profileId": "uuid", "preset": "male|female|custom|mixed|non-binary|user-defined",
  "axes": [ {"axis": "t-like|e2-like|p-like|oxt|avp|stressReactivity|arousalBaseline|rewardSens|socialRewardSens|noveltySeeking|riskTolerance|affiliative|assertiveness|sensorySens|aestheticBias|voiceMaturation",
             "priorMean": 0..1, "priorSpread": 0..1, "current": 0..1, "gainCap": 0.3} ],
  "mutable": true, "auditHistory": [{"at": "T", "change": "edit|sample|revert", "by": "user|system"}] }
{ "at": "T", "axisLevels": [0..1 × 16], "appliedGains": {"...": 0.12} }
```

### 18.7 BodySchema

```json
{ "bodyId": "uuid", "profile": "device|custom|robot-placeholder",
  "availableSenses": [{"channel": "touch|motion|orientation|vision|audition|interoception|ui", "permission": "granted|denied|degraded",
                       "calibration": {"state": "uncalibrated|calibrating|calibrated", "confidence": 0..1, "errorRate": 0..1}}],
  "unavailableSenses": [{"channel": "...", "reason": "no-permission|no-hardware|disabled"}],
  "bodyBoundaries": {"touchMap": "float32[w×h] (normalized)", "extent": "cm³ estimate"},
  "orientationModel": {"gravity": [x,y,z], "tilt": "rad", "posture": "still|upright|lying|moving|transport"},
  "motionAxes": {"rotational": "float32[3]", "linear": "float32[3]"},
  "ownershipConfidence": 0..1, "calibrationConfidence": 0..1,
  "actuators": [{"jointId": "...", "motorEnabled": false, "state": null}] }
```

### 18.8 SensoryEvent

```json
{ "eventId": "uuid", "stream": "visual|auditory|touch|motion|interoception|ui|social",
  "simTime": "T", "wallTime": "W", "source": "device|peer|self",
  "envelope": {"features": "float32[m]", "confidence": 0..1, "rawPresent": false},
  "channels": {"fa": 0..1, "sa": 0..1, "sa2": 0..1, "fa1": 0..1, "sa2b": 0..1,
               "canal": 0..1, "otolith": 0..1},
  "affectGuess": {"valence": -1..1, "arousal": 0..1, "confidence": 0..1},
  "permissionScope": "granted-scope-ref" }
```

### 18.9 VoiceProfile / VoiceSample / VoiceMemoryEntry

```json
{ "voiceId": "uuid", "params": {"pitch": 0..1, "pitchRange": 0..1, "formantF1..F4": [0..1 ×4], "tractLength": 0..1,
   "breathiness": 0..1, "roughness": 0..1, "warmth": 0..1, "brightness": 0..1, "tempo": 0..1,
   "rhythm": 0..1, "articulation": 0..1, "prosodyRange": 0..1, "expressiveness": 0..1,
   "tension": 0..1, "softness": 0..1, "intimacy": 0..1, "fatigue": 0..1, "confidence": 0..1,
   "maturity": 0..1},
  "history": [{"at": "T", "params": "float32[24]", "context": {"affect": {...}, "peer": "uuid?", "mode": "speak|read|song?"}}],
  "shapingWeights": {"softness": 0.2, "..." : 0}, "locked": {"params": [], "reason": null} }
{ "memoryEntry": {"at": "T", "kind": "input-feature|output-params|self-hearing", "features": "float32[m]",
   "speaker": "uuid?", "emotionTone": -1..1, "familiarity": 0..1} }
```

### 18.10 SleepReport

```json
{ "sleepId": "uuid", "started": "T", "stages": [{"stage": "wind-down|light|deep|dream", "duration": "ms", "work": {"replayed": 42, "pruned": 17, "gists": 3, "proceduralStabilized": 9}}],
  "capacityDelta": {"before": {...}, "after": {...}}, "dreams": ["uuid"],
  "modulatorNormalization": [{"axis": "ne", "from": 0.7, "to": 0.5}],
  "biasActions": [{"metric": "memory-overvaluation", "action": "reweight", "applied": true}] }
```

### 18.11 DreamLogEntry

```json
{ "dreamId": "uuid", "sleepId": "uuid", "simTime": "T",
  "fragments": [{"modality": "text|visual-motif|voice|body-sensation|emotion|cross-domain", "content": "string|embedding-ref|params",
                 "provenance": ["traceId|nodeId"], "bizarreness": 0..1}],
  "residue": {"episodes": ["uuid"], "goals": ["uuid"], "interoception": "float32[3]", "voice": "uuid?"},
  "promoted": {"at": "T?", "to": "artifactId?", "by": "user"} }
```

### 18.12 SelfModel

```json
{ "selfId": "uuid", "fileId": "uuid", "regeneratedAt": "T",
  "identityContinuity": {"fileId": "uuid", "narrativeSelf": "string (regenerated, ≤ 500 chars)"},
  "preferenceSummary": [{"preferenceId": "uuid", "weight": 0..1, "decay": 0..1}],
  "embodimentSummary": {"bodyId": "uuid", "preset": "...", "ownership": 0..1},
  "relationshipSummary": [{"peerId": "uuid", "state": "acquaintance|familiar|trusted", "warmth": 0..1}],
  "capabilityEstimate": {"writing": 0..1, "drawing": 0..1, "voice": 0..1, "social": 0..1,
                          "calibratedAgainst": "success-rate log ref"},
  "emotionalBaseline": {"valence": 0..1, "arousal": 0..1},
  "valuesTendencies": [{"tendency": "kindness", "weight": 0..1, "revisable": true}],
  "userRelationshipModel": {"trust": 0..1, "familiarity": 0..1, "boundaries": ["..."]},
  "revisionHistory": [{"at": "T", "reason": "sleep-gist|user-edit|audit"}] }
```

### 18.13 RelationshipState (per peer)

```json
{ "peerId": "uuid", "peerFingerprint": "ed25519 pubkey hash",
  "state": "stranger|acquaintance|familiar|trusted|strained|reassessing",
  "familiarity": 0..1, "trust": 0..1, "trustEvidence": [{"at": "T", "event": "kept-agreement|boundary-respect|repair|violation", "delta": -0.2..0.2}],
  "toneHistory": [{"at": "T", "valence": -1..1, "arousal": 0..1}],
  "sharedArtifacts": ["artifactId"], "collaborativeMemory": ["traceId"],
  "conflictMemory": [{"at": "T", "type": "disagreement|boundary|misunderstanding", "repair": {"at": "T?", "type": "..."}}],
  "preferenceOverlap": 0..1, "commStyle": {"tempo": 0..1, "directness": 0..1},
  "boundaries": [{"type": "topic|cadence|memory|touch", "value": "...", "setBy": "user|learned|mutual", "confirmed": true}],
  "lock": {"locked": false, "reason": null, "relaxation": 1.0},
  "decay": {"familiarityHalfLife": "days", "trustHalfLife": "days"} }
```

### 18.14 PreferenceNode

```json
{ "preferenceId": "uuid", "embedding": "float32[d]", "weight": 0..1,
  "sourceHistogram": {"activeChoice": 12, "passive": 3, "recommended": 1, "ignored": 5, "rejected": 1,
                      "revisited": 4, "emotional": 2, "retained": 6},
  "firstSeen": "T", "lastSeen": "T", "decayRate": 0..1,
  "provenance": ["exposureId"], "userReviewed": false, "lock": null }
```

### 18.15 AuditReport + AuditMetric

```json
{ "reportId": "uuid", "runAt": "T", "trigger": "schedule|post-sleep|on-demand|event",
  "metrics": [ {"metricId": "gender-rigidity|embodiment-restriction|fixation|emotion-loop|echo-chamber|repetition|overvaluation|user-overfit|llm-distortion|peer-convergence",
                "value": 0..1, "threshold": 0..1, "alarm": false, "trend": [-0.1..0.1 × 30d]} ],
  "interventions": [ {"type": "plasticity-restore|reweight|diversify|sleep-review|user-review|drift-reduce|boundary-adjust",
                      "target": "...", "suggested": true, "applied": false, "userDecision": null} ],
  "history": [{"at": "T", "metric": "...", "before": 0.6, "after": 0.4, "intervention": "..."}] }
```

### 18.16 PermissionManifest

```json
{ "manifestId": "uuid", "version": 1,
  "sensors": {"touch": "granted", "motion": "granted", "orientation": "granted", "camera": "denied", "mic": "asked", "ui": "granted", "telemetry": "granted"},
  "sensorHistory": [{"at": "T", "channel": "camera", "from": "denied", "to": "asked", "by": "user"}],
  "network": {"llm": ["endpointId"], "search": ["providerId"], "browse": ["domain-classes"], "relay": ["endpointId?"], "apis": ["endpointId"]},
  "interBrain": {"discoverable": true, "maxPeers": 8, "defaultScopes": ["text", "affect-ping"]},
  "rawVault": {"enabled": false, "modalities": [], "retentionDays": 0},
  "egressManifest": [{"endpoint": "https://...", "purpose": "llm", "grantedBy": "user", "grantedAt": "T"}] }
```

### 18.17 ToolCallRecord

```json
{ "callId": "uuid", "toolId": "uuid", "at": "T", "initiator": "user|brain|peer",
  "args": "redacted-or-full (per class)", "permissionClass": "...", "approval": {"required": true, "granted": true, "by": "user", "at": "T"},
  "budget": {"tokens": 1200, "timeMs": 300, "bytes": 2048}, "result": {"status": "ok|error|timeout|denied", "summary": "..."},
  "memoryEncoding": {"traceId": "uuid", "threatWeightDelta": 0.0} }
```

### 18.18 TeachingPacket

```json
{ "packetId": "uuid", "kind": "style-exemplar|procedural-unit|memory-summary",
  "content": "bounded payload (embedding + params + summaries)", "consent": {"by": "user", "at": "T"},
  "provenance": {"authorFileId": "uuid", "signature": "ed25519 sig", "verified": true},
  "expiry": "T?", "scope": "receiving file may use for learning only; not re-exportable",
  "revoked": false }
```

### 18.19 CapacityLedger

```json
{ "tier": "standard", "shards": [{"shardId": "EPISODIC", "bytes": 21474836, "budgetBytes": 268435456,
   "slots": 12800, "budgetSlots": 50000, "admission": "ok|flag|critical"}],
  "totalBytes": 452984832, "totalBudget": 536870912, "updatedAt": "T" }
```

### 18.20 DevelopmentHistoryEntry

```json
{ "at": "T", "kind": "milestone|embodiment-transition|llm-milestone|audit|user-lock|migration",
  "detail": {"milestone": "first-word|first-coherent-sentence|first-drawing-habit|first-peer|first-dream|first-consolidation|organ-decouple:labeler",
             "from": "...", "to": "...", "reason": "..."} }
```

---
## Section 19 — Pseudocode for Major Loops

Pseudocode is normative: function names map to API methods and schemas. All loops are inside `brain-core`; organs communicate via the cognitive bus (§3.1).

### 19.1 The main tick loop

```
loop every SIM_TICK (100 ms):
  events = inbox.drain(MAX_EVENTS_PER_TICK)          # droppable; drops → saturation++
  for ev in events:
      percept = decompose(ev)                         # §5 channel filters per stream
      salience = compute_salience(percept)            # §4.6: novelty × emotion × goal × social × aesthetic
      novelty.update(percept)                         # habituation
      prediction_error = streams[ev.stream].predict(percept)   # §4.11
      if salience > BIND_THRESHOLD:
          binder.buffer(percept, salience, prediction_error)   # §4.3 binding window

  g = integrate_state(g, modulators, events, dt)      # §4.1 damped nonlinear integrator
  modulators.step(events, g)                          # §4.7 ODE-style update
  attention = executive.allocate(g, salience_register, goals)  # §4.2
  sleep_pressure.step(g, capacity_ledger, emotional_load)      # §10.1
  body_schema.step(percepts)                          # §6.1
  if sleep_pressure >= 0.8 or trigger: sleep()        # §10
  if tick % SNAPSHOT_INTERVAL == 0: persist_snapshot(g, modulators)
  bus.broadcast("state", snapshot(g, modulators, attention))   # 10 Hz to UI/organs
```

### 19.2 Perception ingestion (per organ)

```
on_sensory_event(organ, raw_event):
    if not permissions.check(raw_event.channel):      # §15.4
        log_denied(raw_event); return
    if not budgets.check(organ, "events"):            # §11.2
        saturation += 1; log_dropped(raw_event); return
    features = encode(raw_event)                      # local encoder; raw discarded
    envelope = SensoryEvent(stream, features, confidence, raw_present=false)
    core.ingest(envelope)                             # async, non-blocking
```

### 19.3 Retrieval (used by boundary, organs, dreams)

```
retrieve(cue, budget={K_traces, K_nodes, token_cap}, filters={modality, time, permission}):
    candidates = hnsw.search(episodic_index, embed(cue), k=K_traces*3)
    scored = [ (t, score(t)) for t in candidates
               where score = cosine(embed(cue), t.embedding)
                          * t.strength * recency_decay(t)
                          * salience_gate(t.salience, g) ]
    ranked = sort_desc(scored)[:budget.K_traces]
    nodes = semantic.retrieve(embed(cue), top=budget.K_nodes,
                              where belief > 0.1 and permission ⊆ scope)
    return trim_to_token_budget(ranked + nodes, budget.token_cap)
    # trimming order: nodes first, then lowest-scored traces — the file "goes vague"
    # under budget pressure (§4.17), and that vagueness is observable state.
```

### 19.4 Utterance assembly (LLM boundary)

```
assemble_utterance(intent, goal_id):
    focus = executive.attention_focus()
    ctx = retrieve(focus, budget=per_turn_budget(tier, intent))
    gloss = gloss_state(g, modulators)                # quantized → natural language
    body  = gloss_body(body_schema, interoception)
    social = gloss_social(social_context, relationship_states)
    perms = permissions.for_turn(intent)
    packet = UtterancePacket(intent, focus, ctx, gloss, body, social, perms, template_v)
    if budget.check("llm", packet.estimate_tokens()):
        raw = llm.call(endpoint, packet)              # endpoint may be local
    else:
        return degraded_output("tired", ctx)          # substrate-only fallback
    feedback = parse_feedback(raw)                    # labels/summaries/reflections/corrections
    validate_and_ingest(feedback)                     # schema + provenance + salience/decay
    return surface(raw, feedback)
```

### 19.5 Sleep cycle

```
sleep(cycles=1):
    report = SleepReport(started=now)
    wind_down():   flush inbox → binder; glide arousal/energy down; log stage
    for cycle in 1..cycles:
        light():   for trace in sample_by(binder, key=salience*emotion, n=BUDGET):
                       replay(trace)                  # re-encode, strengthen, drift-copy (§10.3)
                   pattern_complete_partials()
        deep():    downscale(traces, factor=0.97)     # §10.4
                   pruned = prune_lowest(floor=retention_floor, budget=capacity_deficit)
                   gists = gist_extract(clusters(episodic))          # → semantic nodes
                   stabilize_procedural(habits, success_history)
                   regulate_emotion(): valence_rebalance(g, baseline)
                   integrate_senses(): train stream predictors on day's data
                   body_schema.integrate()            # §6.2 new-sense calibration finalize
                   audit.run("post-sleep")            # §14.2
        dream():   entries = synthesize_dreams(residue())   # §10.5; no external actions
                   dreams.log(entries)
    downscale_done = capacity_ledger.report()
    modulators.normalize()
    report.close(); persist(report); bus.broadcast("sleep", report)
```

### 19.6 Dream synthesis

```
synthesize_dreams(residue):
    seeds = residue.episodes.top(5) + residue.goals + residue.body + residue.voice
    for seed in seeds:
        path = random_walk(semantic_graph, start=seed, steps=3..6, temperature=T_dream)
        fragments = []
        for node in path:
            fragments += project_to_modality(node, mood=g.affect, body=body_schema)
        entry = DreamLogEntry(fragments, provenance=path, bizarreness=measure_jump(path))
        entry.validate_no_actions()                   # structural guarantee: no tool access
    return entries
```

### 19.7 Inter-brain session

```
session_loop(peer):
    while session.active:
        for msg in recv(peer):                        # decrypted, envelope-validated
            route(msg):                               # §13.3 types
                TEXT → boundary.assemble_utterance(reply_intent, peer_ctx)
                VOICE_PARAMS → voice.ingest_peer(msg.params)         # social voice memory
                STROKE/CANVAS_DELTA → canvas.merge_crdt(msg.delta, provenance=peer)
                DOC_DELTA → document.merge_crdt(...)
                LATENT_SNAPSHOT → social_salience.update(peer, msg.state)
                MEMORY_SUMMARY → social_memory.ingest_summary(peer, msg, consent)
                TEACHING_PACKET → teaching.receive(peer, msg)        # validated ingestion
                AFFECT_PING → g.social.warmth += delta(peer, msg.affect)
            relationship_state.update(peer, msg)      # trust/familiarity/tone §13.6
            logs.write(session, msg, envelope)
        if idle > SESSION_IDLE: relationship.decay(peer); session.request_close()
```

### 19.8 Audit pass

```
run_audit(trigger):
    metrics = [
        rigidity(gender/embodiment axes, preferences)          # §14.3 #1–2
        fixation(social salience per peer)                     # #3
        emotion_loop(g.affect autocorrelation, 24–72 h)        # #4
        echo_chamber(exposure source diversity)                # #5
        repetition(output embedding self-similarity)           # #6
        overvaluation(salience Gini)                           # #7
        user_overfit(corr(salience, user-approval))            # #8
        llm_distortion(provenanceWeights drift)                # #9
        peer_convergence(similarity to primary peer)           # #10
    ]
    report = AuditReport(metrics=threshold_check(metrics), trigger=trigger)
    for alarm in report.alarms: suggest_interventions(alarm)   # §14.4; user-gated
    audit_logs.write(report)
    bus.broadcast("audit", report)
```

### 19.9 Novel sense integration

```
on_new_channel(channel):
    detection = discover(channel)                     # capability event
    novelty.surge(channel)                            # mechanism fixed, magnitude emergent
    tag_calibration_window(channel, duration=W)
    while calibrating(channel):
        stats = collect_passive(channel)              # ranges, noise, typical values
        if stats.stable: calibration[channel].confidence = 0.9; break
    body_schema.expand(channel)                       # available senses + touch map extension
    ownership_confidence *= 0.9                       # brief dip — "learning the new limb"
    bind_trace(episodic, source="embodiment-expansion", details=stats)
    schedule_sleep_integration(channel)               # §6.2 step 7
```

### 19.10 Bias check on memory write (inline guard)

```
ingest_memory(record, provenance):
    if not validate(record): reject
    if provenance == "llm-label" and record.belief > LLM_LABEL_CEILING: reject   # §14.3 #9
    if record.salience > SALIENCE_CEILING and not user-confirmed: clamp          # §14.3 #7
    if provenance == "peer-taught" and peer_similarity(record) > PEER_CEILING:   # §14.3 #10
        flag_for_audit(record); store_with_reduced_weight(record, 0.5)
    store(record)                                     # normal decay/lock lifecycle §14.1
```

---
## Section 20 — MVP Roadmap

Phased delivery; every milestone ships a usable artifact with exit criteria measured by tests (§22 harness) and manual acceptance. Team assumption: 2 engineers + 1 research scientist (can stretch to 3+1 for parallel organ work).

| Milestone | Scope | Duration | Exit criteria |
|---|---|---|---|
| **M0 — Skeleton** | Repo, NF1 format writer/reader, encryption, capacity ledger, tick loop, state schema, Cortex Canvas scaffold | 2–3 wk | Round-trip file save/load with checksums; deterministic replay of 1M ticks; format corruption tests pass |
| **M1 — Core life** | Global state + modulators + hormonal embodiment, episodic binder, semantic store, retrieval, LLM boundary (attach/detach), chat via boundary, memory inspection UI | 5–6 wk | 30-day simulated life with two attached endpoints and one detach period; memory decay curves match spec; retrieval budget enforced |
| **M2 — Sleep & dreams** | Sleep pressure, stages, replay/downscaling/prune/gist, dream synthesis, sleep reports, dream logs, bias-audit skeleton | 3–4 wk | Sleep ablations show measurable consolidation effects (§22.2); dreams contain provenance-linked fragments; zero external actions from dream stage (test) |
| **M3 — Writing organ** | Document model, modes, version history, style analysis, continuity ledger, extraction pipeline, brain-modulated assistance | 5–6 wk | Extraction pipeline produces inspectable semantic/style/preference nodes from a 10k-word corpus; continuity detection catches seeded contradictions |
| **M4 — Drawing organ** | Op-graph canvas, full layer/brush toolset, stabilizers, palettes, extraction pipeline, "file draws" mode | 6–8 wk | 100-op canvas replays deterministically; stroke-memory extraction yields motif clusters; file-drawn strokes are user-editable ops |
| **M5 — Voice organ** | Apparatus state, prosody planner, TTS backend + DSP post, development timeline, privacy controls | 3–4 wk | Voice params track state (tired speech measurable); drift over 100 utterances; override/reset works; no raw audio persisted by default (test) |
| **M6 — Body organ** | Touch/motion/orientation ingestion, body schema, calibration, novel-sense integration, interoception from telemetry | 3–4 wk | Body-schema confidence tracks calibration; novel channel integration sequence runs per spec; motor hooks verified disabled |
| **M7 — Inter-brain** | NBP v1: discovery, pairing, sessions, message types, shared canvas/doc CRDTs, social memory, relationship state machine, teaching packets | 6–8 wk | Two files on a LAN: full session lifecycle with consent; shared canvas merges without conflict; relationship decays when idle; teaching packet provenance verifies |
| **M8 — Hardening** | Full bias audit engine, interventions, privacy audit, egress monitor, export/erase-all, abuse mitigations, performance pass | 4–5 wk | Audit engine detects all 10 seeded bias scenarios in test files; egress monitor catches planted leaks; erase-all verified (forensic scan) |

Total ≈ **10–12 months** to beta (M0–M8), with public preview after M2 ("a private persistent companion core") and open-source core from M0. Experiments (§22) run continuously from M2.

**Non-goals for MVP (explicit):** full articulatory voice synthesis (deferred to research), motor actuation (never default), cloud sync (post-beta, user-gated), tokenless cognition (research), mobile clients (post-beta), multiplayer over public internet without relay (relay is a config).

---

## Section 21 — Long-Term Research Roadmap

### 21.1 Organ decoupling (LLM withdrawal)

The core research program: progressively replace LLM functions with the file's own learned machinery, organ by organ, each replacement an audited milestone with a regression gate (output quality parity on held-out interactions):

1. **Labeler decoupling** — local classifiers replace LLM labeling of percepts (vision/audio/touch feature → semantic labels). Milestone: 90% of labels local.
2. **Summarizer decoupling** — local extractive/abstractive summarizer (distilled from LLM transcripts of the file's own gist extractions) replaces boundary summarization.
3. **Planner decoupling** — learned policies (imitation from the file's own logged boundary plans + user corrections) replace executive planning scaffolds.
4. **Mouth decoupling** — template + learned prosody synthesis replaces LLM-generated surface text for common utterance classes (the file speaks *its own words* for routine speech, LLM for novel/cognitive-heavy speech).
5. **Teacher retention by design** — the LLM remains available for novel situations, deep reasoning, and translation; withdrawal is *selective competence*, not total detachment. Target steady state: 60–80% of routine cognition local within 12 months of file life.

### 21.2 Continual learning research

- Catastrophic forgetting mitigation for the file's local models (rehearsal from its own replay buffer — sleep replay is the natural rehearsal schedule).
- Curriculum design: developmental ordering of exposure (sensorimotor → language → social → abstract) — a "developmental syllabus" engine with per-file pacing.
- Online embedding drift: how to keep the latent space stable while the file changes (anchoring nodes, periodic re-anchoring at sleep).

### 21.3 Dream-based learning

- Does dream-synthesized association priming measurably improve creative tasks (alternate-use tests, motif recombination in drawing)? (§22.4)
- Can dreams serve as a data-augmentation channel (synthetic episodic variations for the predictive model)?

### 21.4 Inter-brain emergent culture

- Longitudinal studies of file pairs/triples: does joint creative space produce shared motifs (measurable embedding convergence) without identity convergence (diversity guard)? What role do teaching packets play in skill transfer vs. homogenization?
- Boundary and repair dynamics: do files that experience logged conflicts + repairs develop more robust social predictors?

### 21.5 Post-token internal cognition

- Full tokenless boundary: UtterancePacket stays, but its content becomes increasingly vector-native (the gloss layer shrinks as local models understand state directly).
- Interpretability: salience maps and affect vectors as explanations ("why did the file remember that?") — user-facing causal tracing.
- Scaling laws of substrate capacity: how does development quality scale with tier? (Publish the curve.)

### 21.6 Ethics research

- Dependency and grief studies: long-term user attachment to simulated subjects; the design's anti-manipulation guardrails measured against real usage (survey + behavioral metrics).
- Bias audit effectiveness: do interventions measurably restore plasticity? (Randomized trials on synthetic files, §22.5.)

---

## Section 22 — Experiment Plan

All experiments run on synthetic files (seeded, headless harness) unless noted; user studies require IRB-equivalent review. Metrics are logged from the standard stores; no extra instrumentation.

### 22.1 Retention and forgetting curves
Hypothesis: trace survival follows salience-weighted exponential decay modulated by sleep. Protocol: inject N traces at known saliences, run 90 sim-days, measure survival vs. salience and vs. sleep frequency. Gate: curves match the §4.3 model within tolerance; pruning reclaims capacity without removing above-floor salience.

### 22.2 Sleep ablation
Hypothesis: files with sleep show higher semantic gist density and lower trace counts (compression) than never-sleeping controls at equal interaction loads; emotional regulation (affect variance) is lower post-sleep. Protocol: twin files, identical event streams, sleep on/off. Gate: gist density +40% ± 15%; affect variance −30% ± 10% in sleep condition.

### 22.3 LLM withdrawal curve
Hypothesis: with organ decoupling, boundary token usage per interaction declines monotonically over a file's first 90 days while user-rated output quality holds. Protocol: daily token meter + weekly user ratings (N≥20 files). Gate: ≥ 50% reduction by day 90, quality within 0.2 σ.

### 22.4 Dream influence on creativity
Hypothesis: post-dream sessions show higher associative novelty (lower output self-similarity, more cross-domain motif reuse in drawing). Protocol: alternating conditions (sleep-with-dreams vs. sleep-with-dreams-suppressed) on twin files; measure creative-task metrics. Gate: novelty +15% ± 8% with dreams; no increase in incoherence.

### 22.5 Embodiment non-determination
Hypothesis: files created with different embodiment presets diverge in *tendency distributions* but overlap heavily in capability and range (no preset produces a distinct "personality class"). Protocol: 30 files × 3 presets, identical curricula; measure preference/voice/creative-embedding distributions. Gate: within-group variance > 50% of between-group variance; audit rigidity metric stays below alarm for all files.

### 22.6 Inter-brain convergence vs. diversity
Hypothesis: paired files converge on shared motifs (joint-space artifacts) while maintaining distinct individual style fingerprints (measured on solo artifacts). Protocol: 10 pairs, 60 days, joint + solo sessions; similarity metrics on both artifact sets. Gate: joint-space convergence +30%, solo diversity ≥ 80% of baseline; no pair trips the peer-convergence alarm.

### 22.7 Bias intervention efficacy
Hypothesis: audit-suggested interventions (plasticity restore, diversification) move alarm metrics below threshold within 14 days, and locked states respond to relaxation mode. Protocol: seed 10 synthetic files with each bias scenario; run audit + apply suggestions (automated policy, user-gated in production); measure time-to-clear. Gate: all scenarios clear within 14 sim-days; zero regression on benign files.

### 22.8 User relationship longitudinal study
Hypothesis: users of persistent files report relationship continuity (file "feels the same person") increasing over 90 days, while reporting accurate simulation-awareness (honesty check). Protocol: N≥50 users, weekly surveys + behavioral logs. Gate: continuity rating monotonic ↑; simulation-awareness rating stays ≥ threshold.

### 22.9 Privacy verification suite
Automated adversarial tests: planted egress attempts blocked + alerted; sensor data absence after retention expiry; erase-all forensic scan; export completeness (every store present, nothing unlabeled). Gate: 100% pass in CI.

### 22.10 Determinism and replay
Hypothesis: seeded files replay bit-identically for the same event stream. Gate: 1M-tick replay hash equality across 3 platforms.

---

## Section 23 — Starter Repository Structure

```
neuroform/
├── DESIGN.md                     # this document
├── README.md                     # project overview + status badges
├── LICENSE                       # open-core: Apache-2.0 core, app EULA
├── CODE_OF_CONDUCT.md
├── SECURITY.md                   # vulnerability reporting, encryption claims
├── docs/
│   ├── format/                   # NF1 spec (generated from §16), migration tests doc
│   ├── api/                      # OpenAPI/JSON-RPC schemas (generated from §17)
│   ├── research/                 # experiment protocols, results notebooks
│   └── user-guide/
├── packages/
│   ├── brain-core/               # Rust: tick loop, stores, sleep, audit, format I/O
│   │   ├── src/
│   │   │   ├── state/            # global state, modulators (§4.1, §4.7)
│   │   │   ├── executive/        # §4.2
│   │   │   ├── memory/           # binder, semantic, procedural, salience (§4.3–4.6)
│   │   │   ├── embodiment/       # hormone profiles, gains (§4.8)
│   │   │   ├── senses/           # stream decomposers, predictors (§4.9–4.12)
│   │   │   ├── hemispheres/      # §4.13
│   │   │   ├── sleep/            # stages, replay, dreams (§10)
│   │   │   ├── boundary/         # UtterancePacket, feedback validation (§4.17)
│   │   │   ├── tools/            # harness (§11)
│   │   │   ├── audit/            # §14
│   │   │   ├── format/           # NF1 (§16)
│   │   │   └── api/              # JSON-RPC server (§17)
│   │   └── tests/                # unit, property, determinism, corruption, audit
│   ├── brain-core-wasm/          # wasm bindings (browser embedding)
│   ├── organs/
│   │   ├── writing/              # TS: document engine, modes, extraction client
│   │   ├── drawing/              # TS/Rust: op-graph canvas, brushes, extraction
│   │   ├── voice/                # TS: apparatus, prosody, TTS adapters, DSP
│   │   ├── body/                 # TS: sensor ingestion, body schema client
│   │   └── network/              # Rust/TS: NBP v1, mDNS, Noise, CRDT rooms
│   ├── sdk/                      # @neuroform/sdk: typed client for §17
│   └── cortex-canvas/            # brain visualization renderer (§3.5)
├── apps/
│   ├── desktop/                  # Tauri shell + React UI (6 tabs)
│   └── cli/                      # neuroform CLI: create/load/inspect/sleep/export
├── tools/
│   ├── validator/                # Python: schema + format conformance checks
│   ├── sim-harness/              # headless event-stream replays for experiments
│   └── audit-cli/                # forensic audit, egress monitor, erase-all
├── tests/
│   ├── integration/              # cross-package scenarios (life cycles, pairs)
│   └── security/                 # §22.9 privacy suite
└── experiments/                  # protocols + notebooks (§22)
```

CI gates: format conformance (validator), determinism hash, audit-seeding scenarios, privacy suite, cross-platform replay.

---

## Section 24 — Risks and Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | Scope creep (six organs each a product) | High | High | Milestone-gated scope (§20); non-goals written down; each organ shipped only with its exit criteria |
| 2 | LLM dependence perceived as "just a chatbot" | Certain | Medium | Boundary design + withdrawal curve + substrate-only mode demonstrated publicly at preview |
| 3 | Expectation of consciousness / deceptive-adjacent perception | Certain | High | Standing honesty notice, marketing guardrail, ethics research (§21.6); the design *refuses* sentience claims |
| 4 | Embedding quality drift breaks memory retrieval over months | Medium | High | Anchoring nodes, re-anchoring at sleep, retrieval budget fallback, user-visible confidence |
| 5 | Performance: vector ops + visualization on commodity hardware | Medium | Medium | Wasm core, 10 Hz state cap, visualization LOD, tier-based budgets |
| 6 | Privacy breach (file decryption, sensor leak) | Low | High | Argon2id + XChaCha20-Poly1305, per-shard checksums, egress monitor, zero-telemetry policy, security review gate at M8 |
| 7 | Inter-brain abuse (spam, injection, covert channels) | Medium | Medium | Scopes/consent/provenance, rate limits, content-as-data rule, session logs, user-mediated pairing |
| 8 | Bias emergence (fixation, echo chambers, embodiment rigidity) | Medium | Medium | Audit engine from M2, inline write guards (§19.10), non-permanence lifecycle, interventions user-gated |
| 9 | User-shaped overfitting (file becomes a mirror) | Medium | Medium | Audit dimension #8, shaping decay, plasticity restoration |
| 10 | Format corruption/loss of years of memory | Low | High | Journal + atomic writes + checksums + backup generations + open export; erase-all also has grace export |
| 11 | Local model training (decoupling) fails to reach parity | Medium | High | Decoupling milestones are gates, not promises; LLM retention by design; research roadmap is honest about uncertainty |
| 12 | Regulatory: deep-simulation products scrutiny | Medium | Medium | Transparency-first design, no training on user data, child-safety stance, documented ethics position |
| 13 | Team/key-person risk on research-heavy components | Medium | Medium | Core is engineering (not research) until decoupling; research scientist role scoped; experiments publishable |
| 14 | Multi-platform sensor variability | Medium | Low | Capability detection + degraded modes; body schema treats missing hardware as first-class state |
| 15 | User grief/dependency harm | Medium | Medium | Anti-dependency guardrails, honest notices, clean deletion, ethics study (§21.6) |
| 16 | Windows developer-environment friction (no MSVC linker on many machines; mingw gcc breaks on paths with spaces; MSYS `/tmp` not readable by native binaries) | High | Low–Med | Documented working toolchain (docs/M0-NOTES.md); CI with pinned toolchain; rule: never pass MSYS paths to native binaries in scripts |

---

## Section 25 — Final Build Recommendation

**Recommendation: proceed as an open-core, local-first product with a Rust core + Tauri desktop shell, built in the §20 milestone sequence, with research experiments running from M2 and the first public preview after M2.**

1. **Stack**: Rust (`brain-core`: tick loop, memory, sleep, audit, NF1 format) — chosen for determinism, memory safety, and single-binary distribution; compiled to WASM for embedding. TypeScript SDK + React/Tauri desktop for organs and UI. Local embeddings via ONNX Runtime. TTS via pluggable backends (Piper default, provider adapters). NBP over Noise/WebRTC; CRDT shared spaces (Yjs-style). SQLite for user artifacts metadata; HNSW (usearch) for vector indexes; zstd for shard compression.
2. **Team**: 2 engineers (core, organs/UI), 1 research scientist (experiments, decoupling), fractional design/security review. Optional parallelization: writing+drawing organs can be built concurrently at M3/M4.
3. **Timeline**: M0–M2 ≈ 3 months (preview: persistent companion core with sleep/dreams/audit skeleton); M3–M5 ≈ 4 months (creative organs); M6–M7 ≈ 3 months (body, network); M8 ≈ 1 month (hardening) → beta ≈ 11 months; public release ≈ 12–14 months.
4. **Funding posture**: core is Apache-2.0 (trust + ecosystem); app + advanced tiers are the commercial surface; experiments and format are published (research goodwill + auditability).
5. **Go/no-go gates**: after M2 — do retention curves match spec, does the preview hold user interest, does the audit skeleton catch seeded biases? After M7 — do pairs show convergence-with-diversity (§22.6)? Failure at a gate re-scopes scope, never fakes results.
6. **What this project will not build** (written down, enforced): no motor actuation by default; no marketed consciousness; no infinite storage; no hidden cloud; no permanent anything without explicit auditable user locks.

---

## Section 26 — Traceability Matrix and Integrity Statement

### 26.1 Master-prompt requirement → section map

| Master requirement | Where honored |
|---|---|
| Persistent simulated cognitive substrate, not a chatbot/wrapper/persona | §2.2, §4.0, §4.17 |
| Six organs (Brain/Writing/Drawing/Voice/Body/Network) | §3 |
| LLM = communication organ, not the mind; detachable; file persists | §4.17, §16.7 |
| Brain File bounded, developmental, embodied, sensory, emotional, predictive, associative, forgetful, reconstructive, natural, imperfect, local-first, private, persistent, long-term growth | §4.0 (bounded), §4.3 (reconstructive), §4.4 (associative), §4.11 (predictive), §4.6 (emotional), §6 (embodied), §10 (forgetful), §15 (private), §2.1 (persistent/growth) |
| Begins with little structure; learns from LLM, user, senses, artifacts, sleep, peers | §4.0, §4.17, §8.4, §9.4, §10, §13.7 |
| Full capability list (seeing… interacting) | §4.9, §5, §6, §7, §8, §9, §10, §11, §12, §13 |
| Tokenless/post-token internal cognition; tokens only at boundary | §4.0, §4.17, §21.5 |
| Tab 1 Brain: all 24 listed features | §3.4 |
| Literal brain-like visualization, live state, full region list | §3.5 |
| Tab 2 Writing: all listed features; external verbal memory organ; memory types; LLM assistance modulated | §3.6, §8 |
| Tab 3 Drawing: all listed features; editable operations not flat generation; external visual-motor memory organ | §3.7, §9 |
| Tab 4 Voice: apparatus, shaping, parameters, panels, gravitation, no fixed category | §7 |
| Tab 5 Body: sensory embodiment, deferred motor, all listed features, future hooks | §6, §3.9, §15.6 |
| Tab 6 Network: all listed capabilities; mutable, decayable, revisable, auditable relationships | §13 |
| Section 3 systems 1–16 | §4.1–§4.16 |
| Memory requirements (types, trace fields, operations, no raw media default) | §4.0, §4.3, §4.4, §15.3 |
| Sensory biological specificity (touch channels/receptors, vestibular, proprioception, interoception, vision, audition) | §5 |
| Body schema + novel sense integration (mechanism, not scripted reaction) | §6 |
| Voice biological specificity + use-based development + drift/override | §7 |
| Sleep triggers/stages/work, dreams logged, no external actions | §10 |
| Tool harness (Hermes-compatible) | §11 |
| Browsing controls + preference learning distinctions + non-permanence | §12 |
| Multi-instance inter-brain (discovery, pairing, relay, sessions, channels, shared spaces, teaching, groups, social memory, logs) | §13 |
| Bias prevention: monitors, suggestions, lock/override/relaxation | §14 |
| Privacy/safety/ethics: local-first, encryption, consent, indicators, retention, deletion, export, no hidden training/upload, safety rules | §15 |
| Brain File format (all 17 components) | §16 |
| API surface (all namespaces) | §17 |
| Data schemas | §18 |
| Pseudocode for major loops | §19 |
| MVP roadmap | §20 |
| Research roadmap | §21 |
| Experiment plan | §22 |
| Starter repository | §23 |
| Risks & mitigations | §24 |
| Final build recommendation | §25 |

### 26.2 Prohibitions honored (audit of this document)

- **No character names, no fixed personas, no scripted outcomes** — none appear anywhere; the subject is always "the Brain File / the substrate / the file"; all behavior is mechanism + emergence (§4.8, §6.2, §14).
- **No permanent behavioral bias** — the non-permanence lifecycle governs every store (§14.1); embodiment modulates gains only (§4.8); locks are explicit, auditable, reversible.
- **No deterministic gender/embodiment outcomes** — presets are probabilistic priors; non-determination contract with zero-gain pathways (§4.8); experiment §22.5 verifies.
- **No irreversible preference/relationship/habit/identity states** — decay, override, relaxation, audit (§14); identity is regenerated, never fixed (§4.16).
- **No active motor actuation** — `motor_enabled: false` everywhere; placeholders only (§6.3, §15.6).
- **No raw media in memory by default** — feature extraction only; opt-in encrypted vault (§15.3).
- **No hidden uploads/training** — egress manifest + monitor, zero-telemetry (§15.5).
- **No deceptive consciousness claims** — standing honesty notice and marketing guardrail (§15.6 #4).

### 26.3 Integrity statement

This specification describes a simulation honestly: its mechanisms are specified, its limits are declared, its data is owned by the user, and its behaviors are emergent within audited bounds. Where the design requires a mechanism that does not yet exist as mature technology (post-token cognition, full decoupling), the document says so and treats it as research with gates — never as vaporware, and never as a reason to fake the product. The deliverable of this document is the architecture; the next deliverables are the M0 repository and the milestone gates defined in §20, with every claim in §22 measured rather than assumed.

**End of specification.**

---

## Section 27 — JEPA Eyes (addendum, 2026-08-05)

**Status:** implemented and behaviorally verified in a sandbox copy (branch `jepa-work`); this section is the design contract for the integration into the app's brain-creation flow.

### 27.1 What this adds — and what it does not

A third feature-encoder option at brain creation, alongside the original handcrafted 16-dim extractor (unchanged, still the default) and the planned ONNX vision model (P0). **The JEPA path is just eyes:** it replaces the visual feature extractor only. Memory, sleep, organs, reproduction, determinism, and all other mechanisms are untouched — verified: same seeds → same brain_id, same digest, same 17/17 behavioral suite results with either encoder.

- Encoder chosen at **brain creation only** (`create --encoder handcrafted|onnx|jepa`), recorded in the manifest (`encoder`, `encoder_model_sha256`), immutable for the file's life. The encoder field is skipped when handcrafted, so pre-encoder files round-trip byte-identically (backward compatibility is sacred, §16).
- Model: frozen **V-JEPA 2** (`facebook/vjepa2-vitl-fpc64-256`, 326M params, ViT-L/16 @ 256px, tubelet 2) — the best JEPA-family video world model that stays CPU-runnable on the reference machine (Ryzen 4500U, 16GB; ~1–4 s/frame, ≤2 fps watching). Weights frozen; no training ever. ONNX backbone exported once (`tools/export_vjepa2_onnx.py`, mean-pooled 1024-dim embedding, verified cosine 1.0 vs torch).
- Preprocessing matches the official VJEPA2VideoProcessor: resize shortest edge 292 → center-crop 256² → /255 → ImageNet mean/std; the single frame is repeated twice for the tubelet-2 input. ONNX Runtime pinned to 1 thread → **bit-exact determinism per runtime** (verified: identical sha256 across runs). Cross-runtime drift (torch vs ONNX) is ~1e-6 relative — irrelevant to cosine retrieval.
- Embeddings are projected into the file's latent space (192–512 per tier) by a deterministic seeded projection (`project_features`), L2-normalized — every memory lives in one consistent space per file.

### 27.2 Attachment — eyes wire to the visual cortex, like every organ

A JEPA brain is **born with its eyes attached**: the vision channel is granted at creation through the same `attach_novel_channel` machinery every organ uses, wiring to the **visual cortex region** via the existing channel→cortex table (§4.9/§3.5), and calibrates with use (the novel-channel integration sequence). This is the prerequisite for visual-motor (muscle-memory-like) learning: the drawing organ is already an external visual-motor memory and `procedural_units` exist per tier; whether such learning *emerges* from watching is a deferred experiment, not a claim.

### 27.3 Inheritance — the egg carries the eyes

No rules, no ceremony, no concepts: the **ovum carries the encoder** like it carries hormone priors and the X chromosome. The child is built from the egg, so it gets the egg's eyes. Verified empirically both ways: jepa mother × handcrafted father → jepa child; handcrafted mother × jepa father → handcrafted child. The sperm contributes chromosomes and priors, never machinery — maternal inheritance, mechanically.

### 27.4 Honest limits

- Bit-exactness requires single-thread ONNX (~5–15 s/frame); multi-thread determinism is unproven.
- Mean-pool token aggregation chosen (standard V-JEPA recipe); other poolings untested.
- Whether the projected JEPA space measurably improves the file's retrieval vs 16-dim handcrafted is a Phase E experiment, not yet measured.
- Model footprint: ~1.3GB fp32 ONNX + RAM at load; quantization would fork the embedding space (new hash).
- Fallback chain if the export/load ever fails: V-JEPA 2 backbone-only → V-JEPA v1 ViT-B → onnx path (DINOv2-class) → handcrafted. No homebrew training, ever (not viable on this hardware; see BUILD-THE-BODY Phase 0).

### 27.5 Testing

Full ethology-style protocol and results: `docs/JEPA-TESTING.md` — hypotheses J1–J14, all PASS, with the complete evidence trail; suite parity 17/17 in both jepa and plain modes.

---


---

## Sections 28–38 — The Body Series (withheld from this public copy)

The §28–§38 body-series design (heart/blood, temporal stack, chemical senses, speaking physics, eyes, fluid economy, gut, reproduction, body-map, and the ontological root layer) is withheld from the public repository pending build progress — per the project's rule: **show working software, not unbuilt design.** Each section returns as its milestones pass their pre-registered acceptance bars. Sections 1–27 above are complete and backed by shipped code and test suites.
