"""Simulated pump/heart organ — interoceptive beat source.

Stands in for the §38 `physical-heart.py` sidecar: a real pump driver would
read RPM + vibration sensors and emit the identical frame shape. Frames:
  {"bpm": float, "force": float 0..1, "regularity": float 0..1}

Test hook: if the file named by params["kill_file"] exists, health() is
False — the adapter must report sensor-failure, never fake beats.
"""

from __future__ import annotations

import math
import os
import time
from typing import Any

from .base import OrganDriver


class Driver(OrganDriver):
    organ_type = "heart"
    provides = ["interoception"]
    accepts = ["arousal_setpoint"]  # down-path: sympathetic/parasympathetic tone
    default_coordinate = "thorax.midline-left"
    rate_hz = 50.0  # native frame rate; the adapter never forwards raw
    feature_space = "raw"
    direction = "both"

    def __init__(self, organ_id: str, params: dict[str, Any] | None = None):
        super().__init__(organ_id, params)
        self._base_bpm = float(self.params.get("bpm", 64.0))
        self._arousal = 0.0  # setpoint: 0 rest .. 1 exertion
        self._t0 = time.monotonic()
        self._last_poll = self._t0
        self._phase = 0.0

    def read_frames(self) -> list[dict[str, Any]]:
        now = time.monotonic()
        dt = now - self._last_poll
        self._last_poll = now
        n = max(1, min(int(dt * self.rate_hz), 400))  # cap burst replay
        bpm = self._base_bpm * (1.0 + 0.8 * self._arousal)
        period = 60.0 / max(bpm, 1.0)
        frames = []
        for _ in range(n):
            self._phase += period / self.rate_hz * self.rate_hz * 0.02
            # systolic peak shape via rectified sine
            s = abs(math.sin(self._phase * math.pi))
            frames.append({
                "bpm": bpm,
                "force": round(0.35 + 0.5 * s + 0.3 * self._arousal, 4),
                "regularity": 0.98,
            })
        return frames

    def set_setpoint(self, key: str, value: float) -> None:
        if key == "arousal_setpoint":
            self._arousal = max(0.0, min(1.0, float(value)))

    def health(self) -> bool:
        kill = self.params.get("kill_file")
        return not (kill and os.path.exists(kill))
