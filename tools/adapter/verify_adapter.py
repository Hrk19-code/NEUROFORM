#!/usr/bin/env python3
"""verify_adapter.py — end-to-end acceptance for the universal organ adapter.

Runs the REAL pipeline: real neuroform CLI, real organ drivers, real ingest.
Brains used: a COPY of demo.brain (original hash-checked untouched) plus
fresh scratch brains in a temp workdir. Registry sidecars for scratch brains
are cleaned up at the end. Exit code 0 = all green.

Checks:
  1  copy brain verifies; original stays untouched (hash before/after)
  2  auto-attach: heart -> interoception @ thorax.midline-left
  3  encoder gate: mismatched pre-encoded organ refused; raw accepted
  4  multi-instance: two same-kind organs, distinct coordinates
  5  manual attach override lands exactly; brain channel switched on
  6  normalizer math (aggregation is the only thing that crosses)
  7  session ingest lands; file integrity holds after adapter writes
  8  twin determinism: identical ingest streams -> bit-identical digests
  9  failure honesty: dead organ -> sensor-failure event, feed stops
 10  car morphology: 4 wheels + front camera on one fresh brain
 11  viscera staging: olfaction organ -> PENDING, brain untouched
 12  virtual car body: kinematics deterministic; live session ingests
     motion from the sim vehicle; file integrity holds
 13  original demo.brain hash unchanged after the whole run
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ADAPTER_DIR = Path(__file__).resolve().parent
REPO_ROOT = ADAPTER_DIR.parent.parent
sys.path.insert(0, str(ADAPTER_DIR))
import adapter  # noqa: E402

PASS = 0
FAIL = 0


def note(name: str) -> None:
    print(f"{name:<58}", end="", flush=True)


def ok() -> None:
    global PASS
    print("PASS")
    PASS += 1


def bad(why: str) -> None:
    global FAIL
    print(f"FAIL: {why}")
    FAIL += 1


def sha256(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def cli(*args: str, expect_ok: bool = True) -> str:
    p = subprocess.run([sys.executable, str(ADAPTER_DIR / "adapter.py"), *args],
                       capture_output=True, text=True, timeout=600, cwd=REPO_ROOT)
    if expect_ok and p.returncode != 0:
        raise RuntimeError(f"adapter {' '.join(args)}\n{p.stderr or p.stdout}")
    if not expect_ok and p.returncode == 0:
        raise RuntimeError(f"adapter {' '.join(args)} unexpectedly succeeded")
    return (p.stdout or "") + (p.stderr or "")


def main() -> int:
    work = Path(tempfile.mkdtemp(prefix="nf-adapter-verify-",
                                 dir=os.environ["LOCALAPPDATA"] + "\\Temp"))
    demo = REPO_ROOT / "demo.brain"
    copy = work / "copy.brain"
    scratch: list[Path] = []
    try:
        h0 = sha256(demo)
        shutil.copy2(demo, copy)
        brain = adapter.BrainIO(str(copy))

        # 1 ---------------------------------------------------------------
        note("1: copy brain verifies")
        ok() if brain.verify() else bad("copy failed verify")

        # 2 ---------------------------------------------------------------
        note("2: auto-attach heart (interoception @ thorax.midline-left)")
        out = cli("attach", "--brain", str(copy), "--driver", "sim_heart",
                  "--id", "heart-1")
        if "active: heart-1" in out and "interoception" in out \
                and "thorax.midline-left" in out:
            ok()
        else:
            bad(out.strip()[:120])

        # 3 ---------------------------------------------------------------
        note("3: encoder gate (mismatch refused; raw accepted)")
        enc = brain.encoder()
        other = "jepa" if enc == "handcrafted" else "handcrafted"
        try:
            cli("attach", "--brain", str(copy), "--driver", "sim_heart",
                "--id", "heart-enc", "--params", f"feature_space={other}")
            bad("mismatched organ was NOT refused")
        except RuntimeError as e:
            fail_msg = str(e)
            out2 = cli("attach", "--brain", str(copy), "--driver", "sim_heart",
                       "--id", "heart-raw", "--params", "feature_space=raw")
            cli("detach", "--brain", str(copy), "--id", "heart-raw")
            if "feature-space mismatch" in fail_msg and "active: heart-raw" in out2:
                ok()
            else:
                bad(fail_msg.strip()[:100])

        # 4 ---------------------------------------------------------------
        note("4: multi-instance same-kind (two cameras, distinct coords)")
        cli("attach", "--brain", str(copy), "--driver", "sim_camera", "--id", "cam-l")
        cli("attach", "--brain", str(copy), "--driver", "sim_camera", "--id", "cam-r")
        reg = adapter.load_registry(str(copy))
        cams = {a["id"]: a["coordinate"] for a in reg["attachments"]
                if a["organ_type"] == "camera"}
        if cams.get("cam-l") == "head.front" and cams.get("cam-r") == "head.front.2":
            ok()
        else:
            bad(f"coords: {cams}")

        # 5 ---------------------------------------------------------------
        note("5: manual attach override (camera @ chassis.rear)")
        cli("attach", "--brain", str(copy), "--driver", "sim_camera",
            "--id", "cam-rear", "--to", "chassis.rear")
        reg = adapter.load_registry(str(copy))
        rear = [a for a in reg["attachments"] if a["id"] == "cam-rear"]
        if rear and rear[0]["coordinate"] == "chassis.rear" \
                and re.search(r'\[on\]\s+"vision"', brain.body_status()):
            ok()
        else:
            bad("coordinate or channel state wrong")

        # 6 ---------------------------------------------------------------
        note("6: normalizer aggregation math (only summaries cross)")
        n = adapter.Normalizer()
        n.push([{"x": (i % 7) / 7.0} for i in range(200)])
        s = n.summarize()
        n2 = adapter.Normalizer()
        n2.push([{"image_path": "a.png"}, {"image_path": "b.png"}])
        s2 = n2.summarize()
        exp = sum((i % 7) / 7.0 for i in range(200)) / 200
        if s and s["_frames"] == 200 and abs(s["x"] - exp) < 1e-9 \
                and s["x__min"] == 0.0 and abs(s["x__max"] - 6 / 7) < 1e-9 \
                and s2 and s2["image_path"] == "b.png" and s2["_frames"] == 2 \
                and n.summarize() is None:
            ok()
        else:
            bad(f"summary wrong: {s} / {s2}")

        # 7 ---------------------------------------------------------------
        note("7: session ingest lands; file integrity after writes")
        d1 = brain.digest()
        out = cli("run", "--brain", str(copy), "--seconds", "1.5",
                  "--id", "heart-1")
        m = re.search(r"heart-1:\s+(\d+)\s+ingests", out)
        d2 = brain.digest()
        if m and int(m.group(1)) >= 1 and d1 != d2 and brain.verify():
            ok()
        else:
            bad(f"out={out.strip()[:80]} d1==d2:{d1 == d2}")

        # 8 ---------------------------------------------------------------
        note("8: twin determinism (identical streams -> identical digests)")
        t1, t2 = work / "t1.brain", work / "t2.brain"
        for t in (t1, t2):
            subprocess.run([brain.bin, "create", str(t), "--tier", "prototype",
                            "--seed", "99"], cwd=REPO_ROOT, capture_output=True,
                           timeout=120, check=True)
        b1, b2 = adapter.BrainIO(str(t1)), adapter.BrainIO(str(t2))
        for b in (b1, b2):
            b.ingest_interoception({"force": 0.5})
            b.ingest_motion({"vx": 0.6, "wz": 0.0})
        scratch += [t1, t2]
        if b1.digest() == b2.digest():
            ok()
        else:
            bad(f"{b1.digest()} != {b2.digest()}")

        # 9 ---------------------------------------------------------------
        note("9: failure honesty (dead organ -> sensor-failure, feed stops)")
        kill = work / "heart-x.die"
        cli("attach", "--brain", str(copy), "--driver", "sim_heart",
            "--id", "heart-x", "--params", f"kill_file={json.dumps(str(kill))}")
        out1 = cli("run", "--brain", str(copy), "--seconds", "1.0", "--id", "heart-x")
        kill.touch()
        d1 = brain.digest()
        out2 = cli("run", "--brain", str(copy), "--seconds", "1.0", "--id", "heart-x")
        d2 = brain.digest()
        m1 = re.search(r"heart-x:\s+(\d+)", out1)
        m2 = re.search(r"heart-x:\s+(\d+)", out2)
        lst = cli("list", "--brain", str(copy))
        if m1 and int(m1.group(1)) >= 1 and m2 and int(m2.group(1)) == 0 \
                and d1 != d2 and "failed" in lst and "sensor-failure" in lst:
            ok()
        else:
            bad(f"live={out1.strip()[:60]} dead={out2.strip()[:60]}")

        # 10 --------------------------------------------------------------
        note("10: car morphology (4 wheels + front camera, one brain)")
        car = work / "car.brain"
        subprocess.run([brain.bin, "create", str(car), "--tier", "prototype",
                        "--seed", "7"], cwd=REPO_ROOT, capture_output=True,
                       timeout=120, check=True)
        scratch.append(car)
        for wid, coord in (("wheel-fl", "chassis.wheel.fl"),
                           ("wheel-fr", "chassis.wheel.fr"),
                           ("wheel-rl", "chassis.wheel.rl"),
                           ("wheel-rr", "chassis.wheel.rr")):
            cli("attach", "--brain", str(car), "--driver", "sim_wheel",
                "--id", wid, "--params", f"coordinate={coord}")
        cli("attach", "--brain", str(car), "--driver", "sim_camera",
            "--id", "cam-front", "--to", "chassis.front")
        out = cli("run", "--brain", str(car), "--seconds", "1.2")
        counts = dict(re.findall(r"(\S+):\s+(\d+)\s+ingests", out))
        car_io = adapter.BrainIO(str(car))
        if len(counts) == 5 and all(int(v) >= 1 for v in counts.values()) \
                and car_io.verify():
            ok()
        else:
            bad(f"counts={counts}")

        # 11 --------------------------------------------------------------
        note("11: viscera staging (olfaction -> PENDING, brain untouched)")
        d1 = brain.digest()
        out = cli("attach", "--brain", str(copy), "--driver", "sim_heart",
                  "--id", "nose-1", "--channel", "olfaction")
        d2 = brain.digest()
        if "pending: nose-1" in out and "VISCERA P4" in out and d1 == d2:
            ok()
        else:
            bad(out.strip()[:120])

        # 12 --------------------------------------------------------------
        note("12: virtual car body (deterministic sim; live session)")
        from organs import sim_car
        d_a, d_b = sim_car.Driver("a", {}), sim_car.Driver("b", {})
        for _ in range(100):              # 100 polls x 10 frames x 0.01s = 10s
            d_a.read_frames()
            d_b.read_frames()
        sa, sb = d_a.state(), d_b.state()
        model_ok = (sa == sb) and sa["x"] > 0 and sa["theta"] != 0
        vcar = work / "vcar.brain"
        subprocess.run([brain.bin, "create", str(vcar), "--tier", "prototype",
                        "--seed", "11"], cwd=REPO_ROOT, capture_output=True,
                       timeout=120, check=True)
        scratch.append(vcar)
        cli("attach", "--brain", str(vcar), "--driver", "sim_car", "--id", "car-1")
        out = cli("run", "--brain", str(vcar), "--seconds", "1.2")
        m = re.search(r"car-1:\s+(\d+)\s+ingests", out)
        vio = adapter.BrainIO(str(vcar))
        hist = vio.body_status()
        if model_ok and m and int(m.group(1)) >= 1 and vio.verify() \
                and re.search(r"\[t=\d+\][^\n]*motion", hist, re.I):
            ok()
        else:
            bad(f"model_ok={model_ok} out={out.strip()[:60]}")

        # 13 --------------------------------------------------------------
        note("13: original demo.brain untouched by the whole run")
        ok() if sha256(demo) == h0 else bad("ORIGINAL MUTATED")

    finally:
        for p in scratch + [copy]:
            rp = adapter.registry_path(str(p))
            if rp.exists():
                rp.unlink()
        shutil.rmtree(work, ignore_errors=True)

    print("-" * 62)
    print(f"RESULT: {PASS} passed, {FAIL} failed")
    return 0 if FAIL == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
