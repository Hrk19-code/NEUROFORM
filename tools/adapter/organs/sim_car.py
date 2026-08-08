"""Simulated virtual car — a whole vehicle body as ONE organ.

The practice body: the adapter (and therefore the brain) cannot tell this
from a CAN bus. A real vehicle driver emits the identical frame shape from
wheel encoders / IMU; this one integrates a kinematic bicycle model in
pure stdlib Python.

Model (deterministic — fixed dt, fixed frames per poll, no wall-clock in
the dynamics; wall time only decides WHEN a poll happens, never WHAT the
car does):
    x'  = v·cos(θ)          y' = v·sin(θ)
    θ'  = v/L · tan(δ)      (L = wheelbase)
Frames: {"vx": forward m/s, "wz": yaw rate rad/s, "speed": m/s, "slip": 0..1}

Two drive modes:
  scripted (default): accelerate 3s → cruise 3s → curve 3s → brake 2s →
                      rest. A full sensory story for the brain to bind.
  setpoint-driven (params scripted=false): v and δ come ONLY from
                      set_setpoint() — the down-path demo. Nothing moves
                      unless a setpoint arrives (motor intent is the staged
                      engine milestone; this mode is its test socket).

organ_type "car" -> motion @ chassis.body (one organ = one channel; a full
vehicle build is the composite pattern — 4×sim_wheel + sim_camera, check 10).
"""

from __future__ import annotations

import math
import time
from pathlib import Path
from typing import Any

from .base import OrganDriver

DT = 0.01          # s of sim-time per frame
WHEELBASE = 2.7    # m


class Driver(OrganDriver):
    organ_type = "car"
    provides = ["motion"]
    accepts = ["speed_setpoint", "steer_setpoint"]
    default_coordinate = "chassis.body"
    rate_hz = 100.0
    feature_space = "raw"
    direction = "both"

    def __init__(self, organ_id: str, params: dict[str, Any] | None = None):
        super().__init__(organ_id, params)
        self._scripted = bool(self.params.get("scripted", True))
        self._frames_per_poll = int(self.params.get("frames_per_poll", 10))
        # state
        self.x = 0.0
        self.y = 0.0
        self.theta = 0.0
        self.v = 0.0
        self.steer = 0.0
        self.t = 0.0
        # setpoint mode targets
        self._v_target = 0.0
        self._steer_target = 0.0

    # -- dynamics ----------------------------------------------------------
    def _script(self, t: float) -> tuple[float, float]:
        """(v_target, steer) for the scripted drive cycle."""
        if t < 3.0:
            return (8.0 * (t / 3.0), 0.0)          # accelerate to 8 m/s
        if t < 6.0:
            return (8.0, 0.0)                       # cruise straight
        if t < 9.0:
            return (8.0, 0.30)                      # gentle curve
        if t < 11.0:
            return (max(0.0, 8.0 * (1.0 - (t - 9.0) / 2.0)), 0.0)  # brake
        return (0.0, 0.0)                           # parked

    def _step(self) -> dict[str, float]:
        if self._scripted:
            self._v_target, self._steer_target = self._script(self.t)
        # first-order approach to targets (actuator lag)
        self.v += (self._v_target - self.v) * 0.25
        self.steer += (self._steer_target - self.steer) * 0.3
        wz = self.v / WHEELBASE * math.tan(self.steer)
        self.x += self.v * math.cos(self.theta) * DT
        self.y += self.v * math.sin(self.theta) * DT
        self.theta += wz * DT
        self.t += DT
        return {"vx": round(self.v, 4), "wz": round(wz, 5),
                "speed": round(self.v, 4), "slip": 0.01}

    # -- contract ----------------------------------------------------------
    def read_frames(self) -> list[dict[str, Any]]:
        # Deterministic: a fixed number of frames per poll, each DT long.
        # (No wall-clock in the dynamics — same polls => same trajectory.)
        time.monotonic()  # poll cadence is the adapter's business only
        return [self._step() for _ in range(self._frames_per_poll)]

    def set_setpoint(self, key: str, value: float) -> None:
        if key == "speed_setpoint":
            self._v_target = max(0.0, float(value))
        elif key == "steer_setpoint":
            self._steer_target = max(-0.6, min(0.6, float(value)))

    def state(self) -> dict[str, float]:
        return {"x": round(self.x, 3), "y": round(self.y, 3),
                "theta": round(self.theta, 4), "v": round(self.v, 3),
                "t": round(self.t, 2)}

    def health(self) -> bool:
        kill = self.params.get("kill_file")
        return not (kill and Path(kill).exists())
