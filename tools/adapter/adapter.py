#!/usr/bin/env python3
"""neuroform universal organ adapter — the machine-side bridge.

DESIGN.md §38/§29 contract, implemented:
  * The .brain file stays hardware-agnostic. This adapter is machine-side
    (like the models/ sidecar); organs attach to the brain's channel stubs
    through the EXISTING CLI verbs — no engine changes.
  * Any morphology: biped, quadruped, car. Morphology = which organ
    instances attach at which schema coordinates. Unlimited same-kind
    organs (two eyes, four wheels) — the registry keys instances, and
    auto-attach disambiguates coordinates.
  * Universal both encoder ways: organs that emit RAW frames attach to any
    brain (the file encodes with its birth encoder — handcrafted 16-dim or
    frozen V-JEPA 2, immutable, manifest-recorded). Organs emitting
    PRE-ENCODED features must match the brain's birth encoder exactly;
    mismatch is refused honestly (feature spaces are immiscible by design).
  * Up = aggregation only (any organ rate -> one summary per ingest).
    Down = setpoints only.
  * Honesty: a dead organ => sensor-failure event at its coordinate and the
    feed stops. Never synthesize data for a dead organ.
  * VISCERA-forward: chemical-sense organs (olfaction/gustation) register
    as PENDING until the P4 channels exist — recorded, never faked.

CLI:
  adapter.py attach --brain B --driver sim_heart [--id X] [--to coord]
                    [--channel C] [--params k=v,...]
  adapter.py detach --brain B --id X        adapter.py list --brain B
  adapter.py status --brain B               adapter.py run --brain B [--seconds N] [--id X]
  adapter.py fail  --brain B --id X         (test hook: simulate organ death)
  adapter.py map                            (print the organ->schema map)
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

ADAPTER_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(ADAPTER_DIR))
ORGANS_PKG = "organs"
REGISTRY_DIR = ADAPTER_DIR / "registry"
REGISTRY_DIR.mkdir(exist_ok=True)
REPO_ROOT = ADAPTER_DIR.parent.parent

# --- the organ->schema map (extend freely; unknown types are refused) ------
# organ_type -> (channel kind, default schema coordinate)
SCHEMA_MAP: dict[str, tuple[str, str]] = {
    "heart": ("interoception", "thorax.midline-left"),
    "pump": ("interoception", "thorax.midline-left"),
    "camera": ("vision", "head.front"),
    "eye": ("vision", "head.front"),
    "microphone": ("audition", "head.left"),
    "ear": ("audition", "head.left"),
    "touch": ("touch", "skin.dorsal"),
    "patch": ("touch", "skin.dorsal"),
    "imu": ("orientation", "torso.core"),
    "wheel": ("motion", "chassis.wheel"),
    "track": ("motion", "chassis.track"),
    "car": ("motion", "chassis.body"),
    "leg": ("motion", "chassis.leg"),
    "joint": ("motion", "chassis.joint"),
    # VISCERA-forward (channels arrive with P4; honest pending until then):
    "nose": ("olfaction", "face.nasal"),
    "e-nose": ("olfaction", "face.nasal"),
    "tongue": ("gustation", "mouth.dorsal"),
}
CHANNEL_KINDS = {"touch", "motion", "orientation", "vision",
                 "audition", "interoception", "ui"}
STAGED_KINDS = {"olfaction": "VISCERA P4", "gustation": "VISCERA P4"}
ADDABLE_KINDS = {"vision", "audition"}  # body sense --add supports these


# ===========================================================================
# Brain I/O — every write goes through the real CLI; one in flight at a time
# ===========================================================================
class BrainIO:
    def __init__(self, brain_path: str):
        self.brain = str(Path(brain_path).resolve())
        self.bin = self._resolve_binary()

    @staticmethod
    def _resolve_binary() -> str:
        env = os.environ.get("NEUROFORM_BIN")
        if env and Path(env).exists():
            return env
        try:
            out = subprocess.run(["cargo", "metadata", "--format-version", "1",
                                  "--no-deps"], cwd=REPO_ROOT, capture_output=True,
                                 text=True, timeout=60)
            td = json.loads(out.stdout)["target_directory"]
            for cand in (Path(td) / "release" / "neuroform.exe",
                         Path(td) / "release" / "neuroform"):
                if cand.exists():
                    return str(cand)
        except Exception:
            pass
        for cand in (REPO_ROOT / "neuroform.exe", REPO_ROOT / "neuroform"):
            if cand.exists():
                return str(cand)
        raise RuntimeError("neuroform binary not found (set NEUROFORM_BIN)")

    def _run(self, args: list[str], timeout: int = 300) -> str:
        cmd = [self.bin] + [a.replace("{brain}", self.brain) for a in args]
        p = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True,
                           timeout=timeout)
        if p.returncode != 0:
            raise RuntimeError(f"CLI failed: {' '.join(cmd)}\n{p.stderr or p.stdout}")
        return p.stdout

    # -- reads -------------------------------------------------------------
    def inspect(self) -> dict[str, Any]:
        out = self._run(["inspect", "{brain}", "--json"])
        m = re.search(r"\{.*\}", out, re.S)
        return json.loads(m.group(0)) if m else {}

    def encoder(self) -> str:
        # Birth encoder is manifest-level: `verify` prints it. (CLI inspect
        # --json does NOT carry it — checked against a live file.)
        m = re.search(r"encoder:\s+(\S+)", self._run(["verify", "{brain}"]))
        return m.group(1) if m else "handcrafted"

    def digest(self) -> str:
        # Read-only digest probe: 1 tick, NO --save (file untouched; the
        # 3000-tick autosave can't fire). "digest before" = current digest.
        out = self._run(["tick", "{brain}", "--ticks", "1"])
        m = re.search(r"digest before:\s+([0-9a-f]+)", out)
        if not m:
            raise RuntimeError("could not read digest from tick output")
        return m.group(1)

    # NOTE: the body command family is `body <path> <sub>` — path FIRST,
    # subcommand second (cmd_body reads positional[1] as the sub).
    def body_status(self) -> str:
        return self._run(["body", "{brain}", "status"])

    def channels_available(self) -> set[str]:
        # status prints `{:?}` on the kind: [on] "touch" (quoted); a degraded
        # channel is still an attached channel.
        return set(re.findall(r'^\s+\[(?:on|degraded)\]\s+"?(\w+)"?',
                              self.body_status(), re.M))

    def channels_unavailable(self) -> dict[str, str]:
        return {k: r for k, r in
                re.findall(r"^\s+\[off\]\s+(\w+)\s+\(([^)]+)\)", self.body_status(), re.M)}

    def verify(self) -> bool:
        try:
            self._run(["verify", "{brain}"])
            return True
        except RuntimeError:
            return False

    # -- writes (each verb runs the 310-tick bind internally; --save) ------
    def sense_add(self, channel: str) -> str:
        return self._run(["body", "{brain}", "sense", "--add", channel, "--save"])

    def calibrate(self, channel: str, samples: int = 100) -> str:
        return self._run(["body", "{brain}", "calibrate", "--channel", channel,
                          "--samples", str(samples), "--save"])

    def ingest_interoception(self, s: dict[str, float]) -> str:
        return self._run(["body", "{brain}", "interocept",
                          "--energy-load", f"{s['force']:.3f}",
                          "--processing", "0.3",
                          "--memory-pressure", "0.2",
                          "--session-min", "30",
                          "--interaction", "0.2", "--save"])

    def ingest_motion(self, s: dict[str, float]) -> str:
        return self._run(["body", "{brain}", "motion",
                          "--linear", f"{s['vx']:.3f},0,0",
                          "--rotational", f"0,0,{s['wz']:.3f}", "--save"])

    def ingest_touch(self, s: dict[str, float]) -> str:
        return self._run(["body", "{brain}", "touch",
                          "--pressure", f"{s.get('pressure', 0.4):.3f}",
                          "--velocity", f"{s.get('velocity', 0.3):.3f}",
                          "--area", f"{s.get('area', 0.4):.3f}",
                          "--duration", f"{s.get('duration', 800):.1f}",
                          "--contacts", f"{s.get('contacts', 1):.1f}", "--save"])

    def ingest_vision(self, image_path: str) -> str:
        # RAW frame in; the FILE's birth encoder does the features.
        return self._run(["expose", "{brain}", "--image", image_path], timeout=600)

    def ingest_event(self, text: str, source: str = "system") -> str:
        return self._run(["event", "{brain}", "--text", text,
                          "--source", source, "--save"])


# ===========================================================================
# Normalizer — up = aggregation only. Any organ rate collapses to one
# summary per ingest window. Vision keeps the latest raw frame.
# ===========================================================================
class Normalizer:
    def __init__(self) -> None:
        self._buf: list[dict[str, Any]] = []

    def push(self, frames: list[dict[str, Any]]) -> None:
        self._buf.extend(frames)

    def summarize(self) -> dict[str, Any] | None:
        if not self._buf:
            return None
        frames, self._buf = self._buf, []
        out: dict[str, Any] = {"_frames": len(frames)}
        image = next((f["image_path"] for f in reversed(frames)
                      if "image_path" in f), None)
        if image:
            out["image_path"] = image
            return out
        keys = sorted({k for f in frames for k, v in f.items()
                       if isinstance(v, (int, float))})
        for k in keys:
            vals = [float(f[k]) for f in frames if k in f]
            out[k] = sum(vals) / len(vals)
            out[f"{k}__min"] = min(vals)
            out[f"{k}__max"] = max(vals)
        return out


# ===========================================================================
# Registry — durable per-brain attachment record (machine-side sidecar)
# ===========================================================================
def registry_path(brain: str) -> Path:
    stem = Path(brain).stem.replace(".brain", "")
    h = hashlib.sha1(str(Path(brain).resolve()).encode()).hexdigest()[:8]
    return REGISTRY_DIR / f"{stem}-{h}.json"


def load_registry(brain: str) -> dict[str, Any]:
    p = registry_path(brain)
    if p.exists():
        return json.loads(p.read_text())
    return {"brain": str(Path(brain).resolve()), "attachments": []}


def save_registry(reg: dict[str, Any]) -> None:
    registry_path(reg["brain"]).write_text(json.dumps(reg, indent=1))


# ===========================================================================
# Attachment engine
# ===========================================================================
class AttachError(RuntimeError):
    pass


def _load_driver(name: str, organ_id: str, params: dict[str, Any]):
    try:
        mod = importlib.import_module(f"{ORGANS_PKG}.{name}")
    except ModuleNotFoundError:
        raise AttachError(f"unknown driver '{name}' (looked in tools/adapter/organs/)")
    return mod.Driver(organ_id, params)


def attach(brain: BrainIO, driver_name: str, organ_id: str | None,
           to: str | None, channel: str | None,
           params: dict[str, Any]) -> dict[str, Any]:
    drv = _load_driver(driver_name, organ_id or driver_name, params)
    desc = drv.advertise()
    organ_id = desc["id"]

    reg = load_registry(brain.brain)
    if any(a["id"] == organ_id and a["state"] != "detached"
           for a in reg["attachments"]):
        raise AttachError(f"organ id '{organ_id}' already attached")

    # -- encoder gate (universal both ways) ---------------------------------
    brain_enc = brain.encoder()
    fs = desc.get("feature_space", "raw")
    if fs not in ("raw", brain_enc):
        raise AttachError(
            f"feature-space mismatch: organ emits pre-encoded '{fs}' features but "
            f"this brain was born '{brain_enc}' — spaces are immiscible by design "
            f"(encoder is chosen at creation, immutable). Send raw frames instead.")

    # -- channel + coordinate resolution ------------------------------------
    organ_type = desc["organ_type"]
    if channel:
        ch = channel
        default_coord = desc["coordinate"]
    elif organ_type in SCHEMA_MAP:
        ch, default_coord = SCHEMA_MAP[organ_type]
    else:
        raise AttachError(
            f"unknown organ_type '{organ_type}': no schema mapping — pass "
            f"--channel/--to manually or extend SCHEMA_MAP (adapter.py)")
    coord = to or params.get("coordinate") or default_coord

    # -- staged (VISCERA-forward) kinds: register pending, never fake -------
    if ch in STAGED_KINDS:
        entry = _entry(desc, ch, coord, "pending",
                       f"channel '{ch}' unknown to this brain — staged for "
                       f"{STAGED_KINDS[ch]}; recorded, not faked")
        reg["attachments"].append(entry)
        save_registry(reg)
        return entry

    if ch not in CHANNEL_KINDS:
        raise AttachError(f"unknown channel kind '{ch}'")

    # -- multi-instance disambiguation (many limbs of one kind) -------------
    taken = {a["coordinate"] for a in reg["attachments"]
             if a["state"] in ("active", "pending")}
    if coord in taken:
        i = 2
        while f"{coord}.{i}" in taken:
            i += 1
        coord = f"{coord}.{i}"

    # -- brain-side channel attach ------------------------------------------
    if ch not in brain.channels_available():
        if ch in ADDABLE_KINDS:
            brain.sense_add(ch)       # novel channel -> integration sequence
        else:
            raise AttachError(
                f"channel '{ch}' unavailable on this brain and not addable "
                f"via body sense --add")
    brain.calibrate(ch)               # calibration sequence (schema stub)

    entry = _entry(desc, ch, coord, "active", "attached + calibrating")
    entry["stats"] = {"frames": 0, "ingests": 0}
    reg["attachments"].append(entry)
    save_registry(reg)
    return entry


def _entry(desc: dict[str, Any], ch: str, coord: str,
           state: str, note: str) -> dict[str, Any]:
    return {"id": desc["id"], "organ_type": desc["organ_type"],
            "driver": desc["driver"], "channel": ch, "coordinate": coord,
            "direction": desc["direction"], "feature_space": desc["feature_space"],
            "rate_hz": desc["rate_hz"], "params": desc["params"],
            "state": state, "note": note, "attached_at": time.strftime("%Y-%m-%dT%H:%M:%S")}


def detach(brain: BrainIO, organ_id: str) -> dict[str, Any]:
    reg = load_registry(brain.brain)
    for a in reg["attachments"]:
        if a["id"] == organ_id and a["state"] != "detached":
            a["state"] = "detached"
            a["note"] = "detached by user; channel history stays in the file"
            save_registry(reg)
            return a
    raise AttachError(f"organ '{organ_id}' not attached")


def mark_failed(brain: BrainIO, organ_id: str, reason: str) -> dict[str, Any]:
    """Honesty path: the brain is TOLD the sensor failed. No fake data."""
    reg = load_registry(brain.brain)
    for a in reg["attachments"]:
        if a["id"] == organ_id and a["state"] == "active":
            a["state"] = "failed"
            a["note"] = f"sensor-failure: {reason}"
            brain.ingest_event(
                f"sensor-failure at {a['coordinate']} ({a['id']}): {reason}",
                source="system")
            save_registry(reg)
            return a
    raise AttachError(f"organ '{organ_id}' not active")


# ===========================================================================
# Session — poll organs, aggregate, ingest one summary per organ per pass
# ===========================================================================
INGESTERS = {"interoception": "ingest_interoception",
             "motion": "ingest_motion",
             "touch": "ingest_touch"}


def run_session(brain: BrainIO, seconds: float, only_id: str | None = None) -> dict[str, int]:
    reg = load_registry(brain.brain)
    actives = [a for a in reg["attachments"] if a["state"] == "active"
               and (only_id is None or a["id"] == only_id)]
    if not actives:
        raise AttachError("no active organs to run")
    drivers = {a["id"]: _load_driver(a["driver"], a["id"], a.get("params", {}))
               for a in actives}
    norms = {a["id"]: Normalizer() for a in actives}
    stats = {a["id"]: 0 for a in actives}
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        for a in list(actives):  # copy: failure path removes mid-loop
            oid = a["id"]
            drv = drivers[oid]
            if not drv.health():
                # Honesty path: mutate the LIVE registry entry (a is part of
                # reg), tell the brain, persist immediately. Never fake data.
                a["state"] = "failed"
                a["note"] = ("sensor-failure: driver health() is False "
                             "(organ died/unplugged)")
                brain.ingest_event(
                    f"sensor-failure at {a['coordinate']} ({oid}): "
                    f"organ died/unplugged", source="system")
                actives.remove(a)
                save_registry(reg)
                continue
            frames = drv.read_frames()
            if not frames:
                continue
            a["stats"]["frames"] += len(frames)
            norms[oid].push(frames)
            summary = norms[oid].summarize()
            if summary is None:
                continue
            ch = a["channel"]
            if ch == "vision" and "image_path" in summary:
                brain.ingest_vision(summary["image_path"])
            elif ch in INGESTERS:
                getattr(brain, INGESTERS[ch])(summary)
            else:
                continue  # orientation folds into motion (documented); staged kinds skip
            a["stats"]["ingests"] += 1
            stats[oid] += 1
        save_registry(reg)
        time.sleep(0.05)
    for d in drivers.values():
        d.close()
    return stats


# ===========================================================================
# CLI
# ===========================================================================
def _parse_params(raw: str | None) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for kv in (raw or "").split(","):
        if not kv:
            continue
        k, _, v = kv.partition("=")
        try:
            out[k.strip()] = json.loads(v)
        except json.JSONDecodeError:
            out[k.strip()] = v
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="neuroform universal organ adapter")
    ap.add_argument("cmd", choices=["attach", "detach", "list", "status",
                                    "run", "fail", "map"])
    ap.add_argument("--brain")
    ap.add_argument("--driver")
    ap.add_argument("--id")
    ap.add_argument("--to")
    ap.add_argument("--channel")
    ap.add_argument("--params")
    ap.add_argument("--seconds", type=float, default=3.0)
    args = ap.parse_args()

    if args.cmd == "map":
        for t, (ch, coord) in sorted(SCHEMA_MAP.items()):
            staged = f"  [staged: {STAGED_KINDS[ch]}]" if ch in STAGED_KINDS else ""
            print(f"  {t:<12} -> {ch:<14} @ {coord}{staged}")
        return 0

    if not args.brain:
        ap.error("--brain required")
    brain = BrainIO(args.brain)

    if args.cmd == "attach":
        if not args.driver:
            ap.error("attach requires --driver")
        e = attach(brain, args.driver, args.id, args.to, args.channel,
                   _parse_params(args.params))
        print(f"{e['state']}: {e['id']} ({e['organ_type']}) -> {e['channel']} @ "
              f"{e['coordinate']} — {e['note']}")
    elif args.cmd == "detach":
        e = detach(brain, args.id)
        print(f"detached: {e['id']} — {e['note']}")
    elif args.cmd == "fail":
        e = mark_failed(brain, args.id, "fail hook invoked by user")
        print(f"failed: {e['id']} — brain informed (sensor-failure event)")
    elif args.cmd == "list":
        reg = load_registry(brain.brain)
        atts = reg["attachments"]
        if not atts:
            print("no organs attached")
        for a in atts:
            s = a.get("stats", {})
            print(f"  [{a['state']:<8}] {a['id']:<14} {a['organ_type']:<10} "
                  f"{a['channel']:<14} @ {a['coordinate']:<22} "
                  f"frames={s.get('frames', 0)} ingests={s.get('ingests', 0)}")
            if a["state"] in ("pending", "failed"):
                print(f"             note: {a['note']}")
    elif args.cmd == "status":
        print(f"brain encoder: {brain.encoder()}")
        print(brain.body_status())
    elif args.cmd == "run":
        stats = run_session(brain, args.seconds, args.id)
        for oid, n in stats.items():
            print(f"  {oid}: {n} ingests")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AttachError, RuntimeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
