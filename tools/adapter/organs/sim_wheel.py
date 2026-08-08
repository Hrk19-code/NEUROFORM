"""Simulated wheel organ — motion feedback for vehicle morphologies.

Proof that morphology is free: a car is just a chassis whose organs are
wheels. Four wheels attach as four motion-channel instances at distinct
schema coordinates (chassis.wheel.fl/.fr/.rl/.rr). Frames:
  {"vx": forward m/s, "wz": yaw rate rad/s, "slip": 0..1}

Actuator note (honest staging): receiving motion FEEDBACK works against
today's CLI (`body motion`). Driving arbitrary new joint ids is the
engine's body-map phase (BD-1 schema growth, P10) — the descriptor's
accepts[] is recorded now so the down-path needs no adapter changes then.
"""

from __future__ import annotations

import time
from pathlib import Path
from typing import Any

from .base import OrganDriver


class Driver(OrganDriver):
    organ_type = "wheel"
    provides = ["motion"]
    accepts = ["speed_setpoint", "steer_setpoint"]
    default_coordinate = "chassis.wheel"
    rate_hz = 100.0
    feature_space = "raw"
    direction = "both"

    def __init__(self, organ_id: str, params: dict[str, Any] | None = None):
        super().__init__(organ_id, params)
        self._speed = float(self.params.get("speed", 0.6))  # m/s baseline roll
        self._steer = 0.0
        self._last_poll = time.monotonic()

    def read_frames(self) -> list[dict[str, Any]]:
        now = time.monotonic()
        dt = now - self._last_poll
        self._last_poll = now
        n = max(1, min(int(dt * self.rate_hz), 500))
        return [{
            "vx": self._speed,
            "wz": self._steer * self._speed,
            "slip": 0.02,
        } for _ in range(n)]

    def set_setpoint(self, key: str, value: float) -> None:
        if key == "speed_setpoint":
            self._speed = max(0.0, float(value))
        elif key == "steer_setpoint":
            self._steer = max(-1.0, min(1.0, float(value)))

    def health(self) -> bool:
        kill = self.params.get("kill_file")
        return not (kill and Path(kill).exists())
