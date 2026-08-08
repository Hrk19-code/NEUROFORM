# The Adapter Guide — brains, bodies, virtual characters, and games

How to grow a brain file and give it a body — simulated or physical.
Every command in this guide was run and verified on 2026-08-08
(harness: `tools/adapter/verify_adapter.py`, 13/13).

---

## 1. What you need (per OS)

| | Windows | Linux | macOS |
|---|---|---|---|
| Rust toolchain | `rustup` from rustup.rs | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | same as Linux (or `brew install rust`) |
| Python | 3.10+ from python.org (tick "Add to PATH") | `sudo apt install python3` | `brew install python` |
| git | git-scm.com | `sudo apt install git` | `brew install git` |
| ffmpeg (optional — real cameras) | ffmpeg.org builds | `sudo apt install ffmpeg` | `brew install ffmpeg` |

Build the engine (all OSes, from the repo root):

```bash
cargo build --release
```

Where the binary lands:
- **Windows:** the repo overrides the target dir (apostrophe-path
  workaround) — find it with `cargo metadata --format-version 1 --no-deps`
  (field `target_directory`), then `<target>\release\neuroform.exe`.
  A convenience copy also sits at the repo root: `neuroform.exe`.
- **Linux/macOS:** `<target>/release/neuroform` (same metadata trick; the
  adapter resolves all of this automatically — or set `NEUROFORM_BIN`).

Python deps for the adapter: **none** (stdlib only).
For JEPA-encoder brains: the model sidecars in `models/` and the encoder
venv (machine-side, see §6 below).

---

## 2. Make a brain

```bash
neuroform create myfirst.brain --tier standard --seed 42
# full options:
neuroform create myfirst.brain --tier prototype|standard|advanced|experimental \
    --embodiment male|female|custom|mixed|non-binary|user-defined \
    --encoder handcrafted|onnx|jepa \
    --seed 42 --passphrase "your secret"
```

- **Where it will be:** exactly where you point — the path you give is the
  file. `myfirst.brain` in the command above lands in the current
  directory. Keep it anywhere; back it up like a document (it is one).
- **Tier** = capacity (how much it can hold), NOT intelligence.
- **Seed** = determinism. Same seed + same life = bit-identical file.
- **Embodiment** = hormone priors (reach, not destiny).
- **Encoder** = its perceptual birth wiring. `handcrafted` = built-in
  16-dim features (default, zero deps). `jepa` = frozen V-JEPA 2
  embeddings (needs `models/` + the export step, below). **Chosen once,
  immutable for the file's whole life.**
- **Passphrase** = Argon2id + XChaCha20-Poly1305 encryption. No phrase =
  `plain-dev` mode (fine for testing; the file says so honestly on load).

Check it any time:

```bash
neuroform verify myfirst.brain     # shards + checksums + encoder line
neuroform inspect myfirst.brain    # state snapshot (--json for machines)
```

### JEPA brains (both lineages are first-class)

```bash
python tools/export_vjepa2_onnx.py     # one-time: exports the frozen backbone
# models/vjepa2-vitl-fpc64-256 must be present (machine-side sidecar)
neuroform create jepabrain.brain --tier standard --encoder jepa --seed 7
```

Handcrafted and JEPA brains attach to the **same** adapter with zero
changes — organs send raw frames, and each file encodes with the encoder
it was born with. (A pre-encoded organ must match the file's encoder or
the adapter refuses: feature spaces are immiscible by design.)

---

## 3. Where everything lives

| Thing | Location |
|---|---|
| The brain | wherever you created it (`*.brain`), plus `.bk` backups written at births |
| Attachment registry | `tools/adapter/registry/<name>-<hash>.json` — machine-side, gitignored; safe to delete (re-attach to rebuild) |
| Encoder models | `models/` (machine-side, gitignored) |
| Writing sidecars | `writing/<brainname>/` (user data, gitignored) |
| Adapter itself | `tools/adapter/` (committed, stdlib Python) |

---

## 4. Attach organs (the 60-second tour)

```bash
cd tools/adapter            # or use full paths
python adapter.py map                                        # organ → schema table
python adapter.py attach --brain ../../myfirst.brain --driver sim_heart --id heart-1
python adapter.py attach --brain ../../myfirst.brain --driver sim_camera --id eye-l --to head.left
python adapter.py attach --brain ../../myfirst.brain --driver sim_camera --id eye-r --to head.right
python adapter.py list --brain ../../myfirst.brain
python adapter.py run --brain ../../myfirst.brain --seconds 5
python adapter.py status --brain ../../myfirst.brain         # body schema + channels
python adapter.py detach --brain ../../myfirst.brain --id eye-r
```

What happens on attach: the organ advertises its type → the adapter
resolves channel + schema coordinate (auto, or your `--to`) → the brain
opens the channel if needed (`body sense --add`) → calibration runs →
the registry records the pairing. Many organs of one kind are fine —
coordinates disambiguate (`head.front`, `head.front.2`, …).

---

## 5. The virtual car (the practice body)

```bash
python adapter.py attach --brain ../../myfirst.brain --driver sim_car --id kitt
python adapter.py run --brain ../../myfirst.brain --seconds 5
python adapter.py status --brain ../../myfirst.brain
# watch the history: motion — Transport rot ... lin ... (the drive cycle)
```

`sim_car` integrates a real kinematic bicycle model (fixed-dt,
deterministic). Default: scripted accelerate→cruise→curve→brake. With
`--params scripted=false` the car only moves when setpoints arrive — that
socket is where the brain's motor intent plugs in when the engine grows
it (VISCERA P10 / EM-1).

---

## 6. A virtual character (write your own organ in 5 minutes)

A character is the same pattern with different geometry: joint encoders
→ motion/orientation frames, contacts → touch frames. Minimal walker:

```python
# tools/adapter/organs/sim_avatar.py
from .base import OrganDriver

class Driver(OrganDriver):
    organ_type = "leg"                    # maps to motion @ chassis.leg
    provides = ["motion"]
    accepts = ["gait_setpoint"]
    default_coordinate = "avatar.legs"
    rate_hz = 60.0
    feature_space = "raw"
    direction = "both"

    def __init__(self, organ_id, params=None):
        super().__init__(organ_id, params)
        self.x = 0.0; self.heading = 0.0; self.speed = 0.0
        self._gait = float(self.params.get("gait", 1.0))  # steps/s

    def read_frames(self):
        # a fixed little walk each poll: forward + a gentle sway
        self.speed += (self._gait - self.speed) * 0.2
        self.x += self.speed * 0.01
        return [{"vx": self.speed, "wz": 0.05 * self.speed, "slip": 0.0}
                for _ in range(10)]

    def set_setpoint(self, key, value):
        if key == "gait_setpoint":
            self._gait = max(0.0, min(3.0, float(value)))
```

Attach + ride:

```bash
python adapter.py attach --brain ../../myfirst.brain --driver sim_avatar --id legs
python adapter.py run --brain ../../myfirst.brain --seconds 5 --id legs
```

Want touch? Add a second organ with `organ_type = "patch"` that emits
`{"pressure": ..., "velocity": ..., "area": ...}` frames when your avatar
bumps into things. Want it in a game engine? The driver reads the
engine's state instead of integrating its own model — next section.

---

## 7. A game character (same trick, outward-facing)

The driver contract is `read_frames() -> list[dict]` — **where the numbers
come from is the driver's business**. For a game:

1. **Get state out of the game.** Mod API (Minecraft Fabric/Forge,
   Garry's Mod, Roblox), a state file the game writes, screen-capture +
   OCR, or a memory reader. Simplest robust pattern: the game mod writes
   `state.json` (position, velocity, health, contacts) every tick.
2. **The driver polls that file** and converts to frames:

```python
def read_frames(self):
    s = json.loads(Path(self.params["state_file"]).read_text())
    return [{"vx": s["speed"], "wz": s["turn_rate"],
             "slip": 1.0 if s["on_ice"] else 0.0}]
```

3. **Vision:** have the game screenshot to a PNG; a camera-driver frame
   of `{"image_path": "shot.png"}` feeds the brain's own encoder.
4. **Control (when the engine's motor intent lands):** the same driver's
   `set_setpoint()` writes `command.json` back; the game mod reads it.
   Until then, run the game character as a *ride-along* — the brain
   watches, remembers, and forms opinions (the infant stage: senses
   first, motor later — by design).

**Rate honesty:** a 60fps game produces 60 frames/s; the adapter
aggregates — the brain receives summaries, never raw spam
("up = aggregation only"). Turn-based games can push one frame per turn.

---

## 8. A physical robot (the same file walks out into the world)

- Subclass `OrganDriver`; read the vendor SDK (Unitree Python SDK, an
  Arduino over serial, a Raspberry Pi's GPIO) and emit the same frame
  shapes the sims use.
- The robot's own balancing/motor firmware IS the 1000Hz reflex daemon —
  the brain sends setpoints, never torque loops.
- `health()` must tell the truth: cable pulled → return `False` → the
  brain gets a `sensor-failure` event at that coordinate (numbness), and
  the feed stops until recalibration. Never synthesize for a dead sensor.
- Chemical sensors (e-nose/tongue) attach as **PENDING** today — their
  channels arrive with VISCERA P4; the registry records them, unfaked.

---

## 9. Rules of the road

1. **One writer at a time.** Don't run two adapter sessions (or an
   adapter session + manual CLI writes) against the same brain file
   concurrently.
2. **Test on copies.** `copy demo.brain test.brain` — never experiment on
   a brain you care about. (The harness hash-pins this rule.)
3. **Determinism is sacred:** drivers keep dynamics wall-clock-free
   (fixed dt per frame); the same ingest stream must give the same life.
4. **Honesty over polish:** a dead organ says so. A staged channel says
   PENDING. A wrong-encoder organ is refused. No fake data, ever.

## 10. Troubleshooting

- `feature-space mismatch` — the organ emits pre-encoded features that
  aren't this brain's birth encoder. Send raw frames instead.
- `unknown channel kind` / `no schema mapping` — pass `--channel/--to`,
  or add one line to `SCHEMA_MAP` in `adapter.py`.
- `media-extract.py sidecar missing` / camera errors — run from the repo
  tree; install ffmpeg for real webcams.
- `verify` fails after anything — stop and tell someone; that should
  never happen (the harness proves integrity after every write).

## 11. Prove it yourself

```bash
python tools/adapter/verify_adapter.py     # 13 live checks, ~2 minutes
```

Green means: attach works, encoders gate correctly, aggregation is exact,
sessions land in the file, failures are honest, determinism holds, a
virtual car drives, and the originals were never touched.
