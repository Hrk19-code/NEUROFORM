"""Simulated camera organ — emits RAW frames for the brain's own encoder.

Universality rule (both encoder lineages): this driver NEVER encodes
features. It writes a real PNG the brain ingests via `expose --image`;
the FILE encodes it with whatever encoder it was born with (handcrafted
16-dim or frozen V-JEPA 2 — chosen at creation, immutable). Pre-encoded
organs (feature_space != "raw") are refused at attach time unless the
space matches the brain's birth encoder exactly.

The PNG writer is stdlib-only (zlib + struct), deterministic per frame
index: a moving block on a gradient — enough structure for the media
extractor to see a changing scene.
"""

from __future__ import annotations

import struct
import tempfile
import time
import zlib
from pathlib import Path
from typing import Any

from .base import OrganDriver


def _png_chunk(tag: bytes, data: bytes) -> bytes:
    body = tag + data
    return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)


def write_png(path: str | Path, width: int, height: int, rows: list[bytes]) -> None:
    """rows: height byte-strings of width*3 (RGB8), no filter."""
    raw = b"".join(b"\x00" + r for r in rows)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + _png_chunk(b"IDAT", zlib.compress(raw))
        + _png_chunk(b"IEND", b"")
    )
    Path(path).write_bytes(png)


class Driver(OrganDriver):
    organ_type = "camera"
    provides = ["vision"]
    accepts = ["gaze_setpoint"]
    default_coordinate = "head.front"
    rate_hz = 30.0
    feature_space = "raw"
    direction = "both"

    W = 64
    H = 64

    def __init__(self, organ_id: str, params: dict[str, Any] | None = None):
        super().__init__(organ_id, params)
        self._frame_idx = 0
        self._last_poll = time.monotonic()
        self._workdir = Path(self.params.get(
            "workdir", Path(tempfile.gettempdir()) / "nf-adapter-frames"))
        self._workdir.mkdir(parents=True, exist_ok=True)

    def _render(self, idx: int) -> Path:
        w, h = self.W, self.H
        bx, by = (idx * 7) % (w - 12), (idx * 5) % (h - 12)
        rows = []
        for y in range(h):
            row = bytearray()
            for x in range(w):
                g = (x * 4 + y * 2) % 256
                if bx <= x < bx + 12 and by <= y < by + 12:
                    row += bytes((255, 80, 40))  # moving warm block
                else:
                    row += bytes((g, g, min(255, g + 30)))
            rows.append(bytes(row))
        path = self._workdir / f"{self.organ_id}-{idx:06d}.png"
        write_png(path, w, h, rows)
        return path

    def read_frames(self) -> list[dict[str, Any]]:
        now = time.monotonic()
        dt = now - self._last_poll
        self._last_poll = now
        n = max(1, min(int(dt * self.rate_hz), 90))
        for _ in range(n):
            self._frame_idx += 1
        # Aggregation rule for vision: only the LATEST frame is kept —
        # the normalizer collapses the rest (count is preserved).
        return [{"image_path": str(self._render(self._frame_idx)),
                 "frame": self._frame_idx, "frames_seen": n}]

    def health(self) -> bool:
        kill = self.params.get("kill_file")
        return not (kill and Path(kill).exists())
