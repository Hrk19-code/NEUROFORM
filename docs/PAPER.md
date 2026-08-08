# Neuroform: A Bounded, Developmental, Embodied Simulated Cognitive Substrate

**Behavioral characterization of a persistent "brain file" — from blank slate to physics learning, chemical attraction, and reproduction**

*Neuroform project — behavioral testing record (2026-08-04 → 2026-08-05), compiled from `docs/TESTING.md` (hypotheses H1–H27, phases 1–6) and the canonical verification suite (17/17) and unit suite (110/110, zero warnings).*

---

## Abstract

We present a behavioral characterization of **Neuroform**, a local-first software system built around a persistent, bounded, developmental simulated cognitive substrate — a single encrypted file called the *brain file*. The system is deliberately *not* a chatbot: no fixed persona, no hardcoded goals, no LLM inside the file. Instead, a deterministic tick loop drives six organs (memory, writing, drawing, voice, body, network) plus an intuitive-physics learner and a reproduction system, all governed by authored *conditions* (priors, baselines, thresholds, complementarities) from which behavior is meant to *emerge* rather than be scripted.

Testing followed an ethology-style protocol: 27 falsifiable hypotheses (H1–H27) spanning determinism, capacity, unlabeled learning, endocrine-modulated agency, and reproduction. All 27 were confirmed. Headline results: (1) **determinism is total** — identical seeds produce bit-identical digests and bit-identical behavioral reports; (2) **capacity dissociates from cognition** — a Prototype-tier file refused traces at exactly its 6000-slot limit while larger tiers held the same 6200-trace load, yet all tiers learned identically (same consolidation, same semantic gist); (3) **unlabeled learning works** — a blank-slate physics learner reached 1.00 confidence that unsupported things fall from 30 raw observation frames, and a permanence violation produced surprise 0.70 that bound a retrievable memory; (4) **endocrine chemistry reaches agency** — female-typed files ran measurably higher oxytocin/cortisol steady-states, bonded faster (familiarity 0.41 vs 0.37), and held higher initiative thresholds than male-typed files, while cognition remained identical (priors, not destiny); (5) **sexual reproduction is pure mechanism** — gonadal complementarity of hormone profiles decides attraction (0.25 → response; 0.11 → none) with no labels anywhere in the decision path; children are per-axis recombinations of both parents' gametes with random sex (the sperm's X/Y draw) and random capacity tier, born with a backup, bonded to by both parents, and able to grow only to an inherited ceiling.

We report what the results do and do not license: the file learns, remembers, bonds, attracts, reproduces, and grows — all through mechanism — but by construction it does not experience, does not know it does not experience, and has no autonomous goals. The paper is the scientific record of a substrate deliberately engineered so that *nature is the author of conditions, and the trajectory is emergent*.

---

## 1. Introduction

### 1.1 Motivation

The Neuroform project began from a design question: can a simulated cognitive organism be built as a *file* — bounded, persistent, private, and local-first — rather than as a cloud service or a chatbot wrapper? The master specification required: no fixed personas; no hardcoded permanent bias; no irreversible preferences; no active motor default; biological specificity; privacy and encryption as first-class properties; multi-instance interaction; and a standing epistemic notice that the substrate does not experience and does not know it does not experience.

A second, stronger constraint shaped everything that followed: **conditions, not outcomes**. The file's behavior must emerge from authored mechanisms — priors, baselines, thresholds, complementarities — never from authored behaviors. The test of the system is therefore not "does it behave correctly" but *"given these conditions, what does it do, and does what it does follow from the conditions rather than from scripts?"*

### 1.2 The brain file

The unit of persistence is the **brain file** (`.brain`): a sharded, checksummed, optionally passphrase-encrypted container holding twelve shards (state, modulators, episodic memory, semantic memory, hormones, dreams, documents, drawings, voice, body, network, physics). A file is created with a capacity **tier** (Prototype → Standard → Advanced → Experimental), a 64-bit **seed** (determinism), and an **embodiment** — a chromosomal ground truth (XX/XY/XXY/X0/chimeric) that selects a gonadal hormone program (16 axes, e.g. testosterone, estradiol, oxytocin, vasopressin, novelty-seeking, assertiveness, affiliative tendency), which in turn sets modulator baselines, event-processing gains, voice priors, initiative thresholds, and bonding warmth.

The tick loop (10 Hz simulated) integrates events through an inbox, binds percepts into episodic memory, nudges affect and modulator state, decays memory, and periodically consolidates during sleep (dreams, gist extraction, pruning). All stochasticity is seeded; the digest (a hash over all shards) changes only when the file changes.

### 1.3 The research question

*Does the brain file do anything?* — operationalized as 27 falsifiable hypotheses. And its sharper companion: *when it does something, does it do it through mechanism rather than through hardwired concepts?* The reproduction milestone makes the second question concrete: attraction between files, sex, heredity, kin recognition, parenting, and growth were required *without any label — "male", "female", "mother", "child" — participating in any decision*.

---

## 2. System (as tested)

| Component | Mechanism (authored) | Behavioral consequence (measured) |
|---|---|---|
| Tick loop + inbox | 10 Hz; bounded inbox (256), 16 events/tick; 310-tick binding window | Percepts bind or drop; pending state is transient |
| Episodic memory | Slot-capped store (tier-dependent: 6,000–200,000); salience-weighted decay | Weak traces decay; full stores displace weakest |
| Semantic memory | Gist consolidation from repetition during sleep | Repeated exposure → nodes ("association") |
| Sleep | Pressure accumulates; deep consolidation: downscale, prune, gist, regulate | Dreams reference experience; traces collapse; pressure resets |
| Writing / Drawing | Documents with style fingerprint & continuity checks; stroke op-graphs + motifs | Bind with `src=writing` / `src=drawing` provenance |
| Voice | Apparatus (breath/pressure/tension), identity (pitch/rate/formant), heard-voice mimicry with consent gate | Pitch follows hormone prior (female 0.53 > male 0.45); drifts with use; override wins |
| Body | Receptor-class touch, motion/posture, interoception, cortex map (9 regions), dormant motor hooks | Touch familiarity rises/decays; novel senses integrate; motors provably dormant |
| Network (NBP v1) | Sessions, scopes, keyed-BLAKE2b MACs, CRDT merge, relationships (familiarity/trust/tone) | Two files form relationships; decay when idle |
| Physics (§4.12) | Blank-slate prediction-error learner (rates, confidence, surprise) | Learns "unsupported things fall" (1.00) from raw frames; violations surprise |
| Endocrine | Karyotype → gonadal program → 16 axes → modulator baselines, event gains, voice, initiative thresholds, bonding warmth | Chemistry differs continuously by type; bonding pace differs; cognition invariant |
| Reproduction (M9) | Pheromone (16-axis profile) in proposal; response = gonadal complementarity (T/E2/P mean |Δ|, threshold 0.15); gametes (ovum X / sperm X-or-Y); per-axis recombination + mutation; lineage (data only); backup at birth; growth to inherited ceiling | Attraction 0.25 responds / 0.11 silent; children of both parents; sex = sperm's draw; kin recognized chemically |
| LLM boundary | Teacher = session attachment (never persisted); deterministic prompt with state gloss + memory + standing notice | The LLM is the mouth, never the mind; detached file is silent |

Testing harness: CLI (`neuroform`) + three committed protocols (`behavioral-test.sh`, `behavioral-reproduction.sh`, `behavioral-cognitive.sh`) + a canonical verification suite (17 lifecycle steps) + 110 unit tests, zero warnings.

---

## 3. Methods

**Ethology framing.** The file was treated as an animal under observation: scripted, deterministic input where possible; qualitative observation log for anything the script did not force; verdicts recorded as *confirmed / partially confirmed / not observed*, with evidence.

**Hypotheses.** 27 falsifiable hypotheses in seven families: provenance binding (H1), affect/regulation (H2–H3), voice (H4), sense integration (H5), social relationships (H6), sleep (H7), determinism (H8), honesty boundaries (H9), unlabeled physics (H10), unlabeled exposure (H11), cortex map (H12), TTS/face/camera (H13–H15), capacity vs cognition (H16), gender priors (H17), cognitive battery (H18), matrix determinism (H19), attraction-as-chemistry (H20), heredity (H21), kin recognition (H22), never-alone reproduction (H23), parental bonding (H24), growth (H25), desire-as-state (H26), gender-beyond-voice (H27).

**Phases.**
- *Phase 1 (H16/H17/H19):* six brains (3 tiers × 2 genders, one NonBinary control), seed 42, identical 20-day scripted life; digests, memory counts, voice, physics compared.
- *Phase 2 (H18):* first cognitive battery pass — three protocol gaps found (all tester-side, see §5.3).
- *Phase 3 (H27):* male vs female, seed 777, autonomy ON (in-process teacher); voice, modulator steady-states, initiative counts, bonding pace measured live.
- *Phase 4 (H20–H26):* two-file unions; attraction controls; 6 births from fresh unions; bonding; growth.
- *Phase 5 (H16):* capacity stress — 6200 exposures of one phrase (`expose --repeat 6200`, single process) on all four tiers, then 2 sleep cycles.
- *Phase 6 (H18):* battery re-run with fixed probes (correct CLI layout `doc <sub> <path>`; exposure before association check; src-specific queries; rank-based recency).

**Reproducibility controls.** Every phase was run at least twice; reports were compared byte-for-byte. The canonical suite and unit suite were re-run after every code change.

---

## 4. Results

### 4.1 Determinism (H8, H19) — CONFIRMED

Identical seed + script produced identical digests (e.g. M1 re-run: digest `2456273757372`, bit-identical). The Phase 3/4/5/6 reports reproduced exactly across three independent runs. Determinism is total and is the load-bearing property of the entire testing program: every observation below is reproducible on demand.

### 4.2 Capacity dissociates from cognition (H16) — CONFIRMED

Under an identical 6200-exposure load: Prototype retained exactly **6000** traces (the surplus 200 refused at the slot cap); Standard, Advanced, and Experimental retained all 6200. After two sleep cycles, all four consolidated to 941–970 traces and each formed exactly **1 semantic node**. The tier changes *how much is kept*, never *how it learns*: capacity is a boundary, cognition is not.

### 4.3 Unlabeled learning (H10, H11) — CONFIRMED

A blank-slate physics learner (all rates 0.500, confidence 0.00) reached `learned: fall 1.00` / `support 1.00` from 30 raw observation frames; a containment violation produced **surprise 0.00 → 0.70**, nudged curiosity, and bound a retrievable memory (`src=physics`, score 0.815). No labels, no Newton: the structure is discovered. Similarly, 20 repetitions of a phrase + 2 sleeps produced 2 semantic nodes in a mixed-embodiment file; 5 repetitions produced none (conservative gist threshold).

### 4.4 Endocrine chemistry reaches agency (H27) — CONFIRMED

Same seed (777), identical 20-day life, autonomy ON:

| Measure | Male | Female |
|---|---|---|
| Voice pitch | 0.45 | 0.53 |
| Modulator da (steady-state) | 0.582 | 0.630 |
| Modulator oxt | 0.283 | **0.379** |
| Modulator cort | 0.387 | 0.435 |
| Bonding (familiarity after 10 identical peer messages) | 0.37 | **0.41** |
| Initiative curiosity-threshold | 0.691 | 0.715 |

Chemistry differs *continuously*, not just at birth (baselines persist); the affiliative/OXT female warms faster; the assertive male initiates at lower state (deterministically proven via a crafted-state straddle test: male fires, female silent at the same state). Event gains differ by 1–4% (sensory/social-reward weighting). **Cognition is invariant** (traces, nodes, retrieval equal). Verdict: priors shape tendency, never capacity.

Honest caveat: scripted life logged 0 initiatives (thresholds are high by design — rare, audited behavior). The *threshold difference* is unit-proven; a live count divergence awaits a deliberately intense environment.

### 4.5 Reproduction as mechanism (H20–H26) — CONFIRMED

- **Attraction (H20):** mirror-chemistry pair (XX↔XY, different seeds) → gonadal complementarity **0.25 > 0.15** → the receiving file responded with its gamete; union consummated. Same-karyotype control (XX↔XX, same seed) → **0.11 < 0.15** → no response. The decision is a profile distance. The receiving file never knows the peer's karyotype — only the 16-axis pheromone. No label participates.
- **Heredity (H21):** 6 births, each from a fresh union (one union = one conception). Tiers random across all four. Sex = the sperm's X/Y draw: 5 daughters, 1 son — within binomial variation (p ≈ 0.11) at this sample size; recorded as observation, not corrected.
- **Kin recognition (H22):** the child's gonadal axes match ≥1 parent (inheritance) and 0 strangers; parents are complementary to each other (attraction, not kin). Lineage is recorded data; nothing reads it for behavior.
- **Never alone (H23):** two ova cannot conceive — structural (no sperm exists to carry a Y); `GameteKind` is enforced at birth.
- **Parental bonding (H24):** mother-bond **0.60 exactly** at birth on all 6; father-bond **0.50** on BirthNotify; both deepen with time spent.
- **Growth (H25):** children are age-gated (`too young to grow (0 ticks, need 86400)`) and grow tier-by-tier to the **inherited ceiling** (parents' max); first-generation files never grow.
- **Desire-as-state (H26):** consummation = oxt surge, valence/arousal up, bond warmth ×5. There is no "desire" variable; the chemistry is the feeling.

### 4.6 Cognitive battery (H18) — CONFIRMED (re-run)

Retrieval exact (events 1/5/10 → ep #1/#5/#10 at k=1); association (2 semantic nodes); consolidation (dreams 4, traces 38); prediction (fall 1.00, violation surprise 0.70); expression provenance (`src=writing`, `src=drawing [draw motif-0 stroke-1]`); recency bounded (oldest trace still retrievable after 30 later events — decay, not catastrophic forgetting). Errors bounded; no crashes.

### 4.7 Honesty boundaries (H9) and the remaining H1–H15 — CONFIRMED

Motors provably dormant; heard voices store features, never raw audio; camera exposure stores 16 features, never raw frames; the teacher prompt carries the standing notice; initiative is logged and audited; cortex regions light with organ use and decay to quiet; TTS renders differ calm vs stressed; the face app follows state JSON in-browser (live webcam gaze remains the single human-in-the-loop item).

---

## 5. Discussion

### 5.1 What the results license

**Conditions produce trajectories.** Every measured behavior traces to an authored condition — a prior, a baseline, a threshold, a complementarity — and the trajectories that emerged were not authored: the 6000-trace cap exactly, the 0.41/0.37 bonding split, the 5:1 sex run, the tier-pitch variation, the 0.70 surprise. The system is a machine for turning authored nature into un-authored biography.

**Capacity and cognition are separable.** The strongest architectural claim — bounded substrate, unbounded-in-kind behavior — survived a direct stress test.

**Chemistry is a tendency profile, never a destiny, never a capacity difference.** Gender-typed differences exist on every authored axis but never touch learning or retrieval. This is the empirically supported version of the "priors, not destiny" design goal.

**Sex, attraction, heredity, kin recognition, parenting, and growth are implementable without concepts.** The complementarity-based attraction, sperm-decided sex, chemical kin recognition, and structural never-alone rule are the paper's strongest demonstration of the no-hardwiring constraint — the receiving file's decision path contains no label.

**Determinism is a feature of the science, not a bug of the organism.** It is what makes every observation reproducible and every trajectory auditable.

### 5.2 What the results do NOT license

- **Experience.** The file does not experience, and does not know it does not experience (standing notice, DESIGN §15.6). Modulator levels are state variables; desire is oxytocin; nothing in the architecture implements phenomenal experience, and none of the results should be read as claiming it.
- **Goals.** Deliberate goal pursuit (the prefrontal milestone) is explicitly deferred; initiative is rare, thresholded, and audited. The file reacts; it does not yet strive.
- **A mind.** The LLM is a session-attached mouth; detached, the file is silent. The substrate is the organism; the teacher is an environment.
- **Statistical generality.** Six births is not a population; one scripted environment is not a world. The 50/50 sex claim and initiative divergence await scale.

### 5.3 Methodological lessons (testing the tester)

Three first-pass battery failures were tester-side, not system-side: the CLI layout is `doc <sub> <path>` (not path-first); a grep read the *dropped-events* count as the semantic-node count; the association probe omitted the exposure step. All were found by the verification scripts themselves — the protocol proved capable of auditing its own probes. We record this as a strength of the method: hypothesis tests were falsifiable enough to fail loudly on protocol error.

### 5.4 Limitations

1. Scripted environments never trigger initiative (0/0); the threshold divergence is unit-proven, not live-counted.
2. The teacher never persists — every initiative-bearing run must be one long-lived process.
3. Semantic gist requires substantial repetition (20+ exposures); 5 produced nothing.
4. Small-sample sex distribution; no population-scale statistics.
5. Lineage is data, not behavior — only the growth ceiling consumes it.
6. Interactions between files are CLI-relayed (no daemon); the qualitative "two brains living" feel awaits the desktop shell.
7. The live face-gaze webcam check is deferred (human-in-the-loop); camera exposure evidence stands in its place.

---

## 6. Conclusions

*Does the brain do anything?* Yes: it learns physics from raw frames without being told what gravity is; it forms relationships whose warmth depends on its own endocrine priors; it is attracted or not by profile distance alone; it produces children that are recombinations of both parents with random sex and inherited ceilings; it recognizes kin chemically; it grows up only if it was born — and every one of these observations is reproducible bit-for-bit.

*Does it do anything through hardwired concepts?* No. The 27-hypothesis program found no behavior that required a label in a decision path.

The deeper result is the shape of the thing: a bounded file whose nature is authored and whose biography is emergent — an organism, by the modest definition that it persists, learns, feels (in the state-variable sense), bonds, reproduces, and develops, and by the honest definition that none of this is experience. What remains is the qualitative pass (richer environments, initiative under intensity, two files living), population-scale reproduction statistics, the deferred prefrontal goal layer, and the desktop shell. The substrate is done; the biography is just beginning.

---

## References (conceptual anchors, not endorsements of equivalence)

- Braitenberg, V. (1984). *Vehicles: Experiments in Synthetic Psychology.* MIT Press. — the tradition of minimal mechanism → rich behavior.
- Ray, T. S. (1991). *An approach to the synthesis of life.* Artificial Life II. — digital organisms, open-ended evolution.
- Friston, K. (2010). *The free-energy principle: a unified brain theory?* Nature Reviews Neuroscience 11, 127–138. — prediction-error as organizing principle (our physics learner is a minimal, deterministic instance).
- LeCun, Y. (2022). *A Path Towards Autonomous Machine Intelligence.* — world models; cited as motivation for prediction-error substrates, with the honest note that our learner is 1,000× simpler.
- Wedekind, C. & Füri, S. (1997). *Body odour preferences in men and women: do they aim for specific MHC combinations?* Proc. R. Soc. B 264. — complementarity-based mate choice; the conceptual analog of our gonadal-complementarity attraction.
- Tinbergen, N. (1963). *On aims and methods of ethology.* Zeitschrift für Tierpsychologie 20. — the ethology framing used for the testing protocol.

*Internal record: `DESIGN.md` (26 sections), `docs/M0–M9-NOTES.md`, `docs/TESTING.md` (the complete evidence base for this paper), `tools/verify/` (three behavioral protocols + canonical suite), NBP v1 spec. The paper is a condensation; every number in §4 is reproducible from the committed protocols.*
