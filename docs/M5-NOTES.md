# M5 Status & Implementation Notes

**Date:** 2026-08-04 · **Milestone:** M5 (Voice organ) — per DESIGN.md §20 and master-prompt Tab 4.

## Scope delivered

- **Vocal apparatus state** (`voice.rs`, biological analogues): breath / subglottal pressure / larynx tension / fold stability / tract length / tongue tendency / lip-jaw openness / tempo / pause tendency / prosody range. Idle dynamics: breath recovers, tension follows arousal, fold stability degrades with fatigue (deterministic).
- **Developmental voice identity**: pitch mean/range, formant shift, breathiness, warmth, brightness, articulation crispness, expressiveness, tension, softness, intimacy, confidence, maturity. Embodiment presets contribute **probabilistic priors only** (e2-vs-t axis nudges pitch baseline + formant tendency; bounded, mutable, never locked — `priors_respect_embodiment_without_locking`).
- **Prosody planner**: `VoicePlan` = rendered params (pitch, rate, energy, breathiness, warmth, brightness, roughness) + prosodic stage structure (pause/emphasis/soft from surface text) + `TtsMapping` (backend, pitch semitones, rate multiplier, gain dB) + emotional coloring (bright/troubled/heavy/calm/steady from valence×arousal). Pure function of state — deterministic (`plan_is_deterministic_and_affect_sensitive`). TTS synthesis itself lives in the desktop shell; the organ decides *how* the voice sounds.
- **State tracking**: fatigue roughens/slows/breathifies the voice (`fatigue_degrades_voice`); arousal raises energy+rate; overrides always win and are audited (`override_wins_and_is_audited`); usage tendencies drift bounded (`identity_drifts_with_use_but_stays_bounded`, history capped at 100 events = development timeline).
- **Heard-voice mimicry pathway** (M5 extension, DESIGN.md §5.8/§7): `tools/audio-extract.py` sidecar extracts a deterministic 16-dim feature vector {pitch, energy, brightness, dynamics, articulation, instability} from a wav (8/16/24-bit native; other formats via ffmpeg transcode). The brain stores **only the 16-dim summary** — raw audio never enters the file (`heard_voice_stores_features_never_raw_audio`). Similar patterns merge (cosine ≥ 0.85, running mean); salience decays without reinforcement and forgotten voices drop (`heard_voices_decay_without_reinforcement`).
- **Mimicry is emergent, not scripted**: blending toward a heard voice happens only when (a) that voice has user consent AND (b) the global voice-learning gate is on (both default OFF). Refusals are counted (`refused_mimicry`), blends are counted (`mimicry_uses` — bias-audit visibility). Identity drift from mimicry is gradual and bounded, never collapses onto the target (`mimicry_drift_is_gradual_and_bounded`, 200-utterance test).
- **Privacy controls**: per-voice consent (`voice consent --id N --on/--off`), global gate (`voice consent --on`), audited overrides (`voice override/clear`), heard-voice list with labels/salience/counts in `voice status`.
- **Format**: 9th shard `VOICE`; `FileContents.voice` serde-defaulted for old files; validator extended (`[ok] shard VOICE: …`).
- **CLI**: `voice status/speak/hear/consent/override/clear`, `inspect` voice line, `life`/`chat` use the organ.
- **Tests**: 74 total (was 62) — 12 new: determinism/affect-sensitivity, fatigue degradation, override audit, drift boundedness, embodiment priors, heard-voice merge, consent+gate gating, gradual bounded drift, target mapping, decay, brain-level persistence + gating, **no-raw-audio guarantee**.

## Bugs caught (all fixed)

1. **`tools/audio-extract.py` struct format bug** (real, shipped in the M5 working tree): `struct.unpack(fmt * count, raw)` with `fmt = "<h"` produced `"<h<h<h…"` — the endian prefix repeated on every element, so *every* wav failed with `struct.error: bad char in struct format` (`voice hear` was unusable). Fixed: repeat only the format code (`fmt[0] + fmt[1:] * count`). Verified against real 8/16/24-bit wavs end-to-end.
2. **Verify-suite stale assertions** (script, not code): hardcoded `"62 passed"` (now 74) and `"shard index: 8 shard"` (now 9, VOICE) — both made the canonical suite fail despite green tests. Made dynamic (`test result: ok. N passed; 0 failed`, `shard index: [0-9]+ shard` + `shard VOICE:` present).
3. **Verify-suite pipefail race** (script, latent): `neuroform.exe … | grep -q "16 features"` with `set -o pipefail` — `grep -q` exits on first match and closes the pipe, the CLI's next stdout write hits a broken pipe, exit code goes non-zero, and the step spuriously fails. Intermittent (passed in earlier runs by timing luck). All piped greps in the suite now write to a log file first. Suite verified green 3/3 consecutive runs.

## Exit criteria status (M5)

- [x] Apparatus state (breath/pressure/larynx/fold/tract/articulation)
- [x] Prosody planner (stages, emotional coloring, TTS mapping contract)
- [x] Voice params track state — tired speech measurable (fatigue test)
- [x] Drift over 100 utterances (usage + mimicry drift tests, bounded)
- [x] Override/reset works (audited, reversible, unknown params rejected)
- [x] **No raw audio persisted by default** (test: serialized organ has no audio field; file stays < 16 KB vs a 32 KB wav)
- [x] VOICE shard + validator + serde-default migration for old files
- [ ] Actual TTS synthesis + DSP post (edge/piper/cloud backends) — desktop-shell milestone
- [ ] Vocal tract visualization, pitch/prosody/breath views — desktop-shell milestone
