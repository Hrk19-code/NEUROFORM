# tools/adapter — the universal organ adapter

Machine-side bridge between **any device** and the `.brain` file's organ
stubs. This is the §38 bridge (DESIGN.md "Embodiment & Calibration: the
phantom-to-physical bridge") built as infrastructure: organs advertise,
attach, feed summarized ingest, receive setpoints — through the EXISTING
CLI verbs. The engine is untouched; the file stays hardware-agnostic.

## What "universal" means here (all verified by `verify_adapter.py`)

- **Any morphology.** Biped, quadruped, car. Morphology is just *which
  organ instances attach at which schema coordinates*. A car = a chassis
  whose organs are wheels. Check 10 attaches 4 wheels + a front camera to
  one brain.
- **Many limbs of one kind.** Organs are keyed by instance id; auto-attach
  disambiguates coordinates (`head.front`, `head.front.2`, …). Manual
  override always available (`--to chassis.rear`).
- **Any sensor.** The registry maps `organ_type → channel + coordinate`
  (`adapter.py map`). Unknown types take `--channel/--to` manually, or the
  map is extended by one line.
- **Both encoder lineages, both ways.** Organs emit RAW frames; the FILE
  encodes them with its birth encoder (handcrafted 16-dim or frozen
  V-JEPA 2 — chosen at creation, immutable, sha256-pinned in the manifest).
  Pre-encoded organs must match the brain's birth encoder exactly;
  mismatch is refused (`feature-space mismatch`). Check 3.
- **Just the brain files, no robot needed.** Sim organs (`organs/sim_*.py`)
  attach to the same channel stubs a physical driver uses. Hardware is
  "a driver that isn't simulated."

## The §29/§38 rules, enforced in code

| Rule | Mechanism |
|---|---|
| Up = aggregation only | `Normalizer`: any organ rate collapses to one summary per ingest (numeric mean/min/max; vision keeps latest raw frame + count). 1000Hz chatter never crosses. |
| Down = setpoints only | `driver.set_setpoint(key, value)`; the organ/daemon computes realization. |
| Brain is hardware-agnostic | No URDF, no torques in the file. The adapter speaks `body`/`expose`/`event` CLI verbs only. |
| Sensor honesty | Dead organ → `sensor-failure` event at its coordinate (`--source system`), feed stops, recovery requires recalibration. Never synthesized data. Check 9. |
| Determinism | Identical ingest streams → bit-identical digests (check 8, twin brains). The adapter adds no entropy of its own. |
| VISCERA-forward | Chemical-sense organs (olfaction/gustation) register **PENDING** until P4 channels exist — recorded, never faked (check 11). The P11 sidecars (`physical-heart.py`, `e-nose.py`) subclass `organs/base.py` exactly like the sims. |

## Usage

```
python tools/adapter/adapter.py map                      # organ→schema table
python tools/adapter/adapter.py attach --brain B --driver sim_heart --id heart-1
python tools/adapter/adapter.py attach --brain B --driver sim_camera --id cam-1 --to head.left
python tools/adapter/adapter.py attach --brain B --driver sim_wheel --id w-fl --params coordinate=chassis.wheel.fl,speed=0.8
python tools/adapter/adapter.py run --brain B --seconds 5
python tools/adapter/adapter.py list --brain B
python tools/adapter/adapter.py fail --brain B --id heart-1   # test hook
python tools/adapter/adapter.py detach --brain B --id cam-1
python tools/adapter/adapter.py status --brain B
```

Attachment state persists per brain in `registry/` (machine-side,
gitignored). Detaching never erases the brain's channel history.

## Writing a driver (physical or sim)

Subclass `organs/base.py:OrganDriver`:

```python
class Driver(OrganDriver):
    organ_type = "heart"            # resolves via SCHEMA_MAP (or pass --channel)
    provides = ["interoception"]    # channels this organ feeds
    accepts = ["arousal_setpoint"]  # down-path setpoints it consumes
    default_coordinate = "thorax.midline-left"
    rate_hz = 50.0                  # native rate; adapter aggregates
    feature_space = "raw"           # raw | handcrafted | jepa
    direction = "both"              # sensor | actuator | both
    def read_frames(self): ...      # list of flat dicts; "image_path" for vision
    def health(self): ...           # False = dead/unplugged (never lie)
```

## Channel map (defaults)

`heart/pump→interoception@thorax.midline-left` · `camera/eye→vision@head.front` ·
`microphone/ear→audition@head.left` · `touch/patch→touch@skin.dorsal` ·
`imu→orientation@torso.core` · `wheel/track/leg/joint→motion@chassis.*` ·
`nose/e-nose→olfaction` (staged P4) · `tongue→gustation` (staged P4)

## Virtual bodies (sim organs)

The adapter cannot tell a simulator from a CAN bus — so the practice body
is virtual by construction. `organs/sim_car.py` is a whole virtual vehicle
as one organ: a kinematic bicycle model (fixed-dt dynamics, deterministic
per poll — wall-clock decides WHEN a poll happens, never WHAT the car
does), scripted drive cycle (accelerate → cruise → curve → brake) or
setpoint-driven mode (`scripted=false`: nothing moves unless a setpoint
arrives — the test socket for the staged motor-intent milestone).
Live proof: a fresh brain attached to `sim_car` recorded accelerating
motion events (lin 4.84 → 5.91 m/s) with its **vestibular cortex region
lighting up** (cortex: vestibular:0.69) — organ use visible in the file.
A virtual *character* is the same pattern with different geometry (joint
encoders → motion/orientation frames, contacts → touch frames); when the
motor-intent down-path lands (P10/EM-1), the same loop closes for avatars.

## Honest limitations (recorded, not hidden)

- **Actuator down-path is staged.** Motion *feedback* ingests today
  (`body motion`); driving arbitrary new joint ids is the engine's
  body-map phase (BD-1 schema growth, VISCERA P10). Descriptors already
  carry `accepts[]` so no adapter changes are needed then.
- **Orientation folds into motion** (`--rotational`) — today's CLI has no
  separate orientation ingest verb.
- **Audition** routes through `voice hear` (consent-gated) — wired in the
  engine, kept out of the default sim suite because consent is a
  per-brain user decision.
- Adapter writes are CLI processes (each bind = 310 sim-ticks). It is a
  session bridge, not a 1000Hz hard-realtime loop; the P11 reflex daemon
  is the hard-rate layer and plugs into the same registry.

## Verification

`python tools/adapter/verify_adapter.py` — 13 checks, all live: copy-brain
safety (original hash-pinned), auto/manual attach, encoder gate, multi-
instance, aggregation math, session ingest + file integrity, twin
determinism, failure honesty, car morphology, viscera staging, virtual
car body. Current state: **13/13 PASS** (2026-08-08). Uses a COPY of
`demo.brain` plus scratch brains in temp; originals are never touched.
