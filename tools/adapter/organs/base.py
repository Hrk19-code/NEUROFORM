"""Organ driver base — the contract every device/simulated organ implements.

DESIGN anchors (DESIGN.md §38, §29):
- The brain file is hardware-agnostic: it emits schema-level setpoints and
  consumes calibrated, *summarized* ingest. It never sees raw high-rate noise.
- Up = aggregation only. Drivers may produce frames at any rate; the adapter
  normalizes to the brain's cadence. Raw 1000Hz chatter never crosses.
- Down = setpoints only. The brain sends goals; the driver/daemon computes
  the actuator-level realization.
- Honesty: a dead organ reports failure; the adapter marks the brain channel
  with src=sensor-failure. Never synthesize data for a dead organ.
"""

from __future__ import annotations
from typing import Any


class OrganDriver:
    """Base class for organ drivers (simulated or physical).

    Subclasses MUST set class attribute `organ_type` and MAY set
    `provides`, `accepts`, `default_coordinate`, `rate_hz`,
    `feature_space` ("raw" | "handcrafted" | "jepa"), `direction`.
    """

    organ_type: str = "generic"
    provides: list[str] = ["interoception"]
    accepts: list[str] = []
    default_coordinate: str = "chassis.core"
    rate_hz: float = 10.0
    feature_space: str = "raw"  # raw = the brain's own birth-encoder encodes
    direction: str = "sensor"   # sensor | actuator | both

    def __init__(self, organ_id: str, params: dict[str, Any] | None = None):
        self.organ_id = organ_id
        self.params = params or {}

    # -- identity ---------------------------------------------------------
    def advertise(self) -> dict[str, Any]:
        """Return the organ descriptor the registry stores."""
        return {
            "id": self.organ_id,
            "organ_type": self.organ_type,
            "direction": self.direction,
            "provides": list(self.provides),
            "accepts": list(self.accepts),
            "coordinate": self.params.get("coordinate", self.default_coordinate),
            "rate_hz": float(self.params.get("rate_hz", self.rate_hz)),
            "feature_space": self.params.get("feature_space", self.feature_space),
            "driver": type(self).__module__.rsplit(".", 1)[-1],
            "params": dict(self.params),
        }

    # -- data -------------------------------------------------------------
    def read_frames(self) -> list[dict[str, Any]]:
        """Return frames produced since the last call (may be empty).

        A frame is a flat dict. Numeric values are aggregated by the
        normalizer; a frame with key "image_path" carries a raw visual
        frame for the brain's own encoder (never pre-encoded here).
        """
        return []

    def set_setpoint(self, key: str, value: float) -> None:
        """Receive a down-path setpoint (goal, not state)."""

    # -- health -----------------------------------------------------------
    def health(self) -> bool:
        """False => the organ is dead/unplugged. The adapter will mark
        sensor-failure and stop feeding the channel until recovery +
        recalibration. Never lie here."""
        return True

    def close(self) -> None:
        """Release device handles. Sim organs may no-op."""
