#!/usr/bin/env python3
"""Inline snapshot.json into a self-contained preview.html for the Cortex Canvas.

file:// fetch of snapshot.json is blocked in some browsers; this produces a
single HTML file with the snapshot embedded as a JS constant.

Usage: python tools/make_preview.py packages/cortex-canvas
"""

import json
import pathlib
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: python tools/make_preview.py <cortex-canvas dir>")
        return 2
    root = pathlib.Path(sys.argv[1])
    snap = root / "snapshot.json"
    if not snap.exists():
        print(f"no snapshot.json in {root} — run: neuroform inspect <file> --json --out {snap}")
        return 1
    data = json.loads(snap.read_text())
    index = (root / "index.html").read_text()
    js = (root / "main.js").read_text()
    # Replace the fetch-based loader with an embedded constant.
    js = js.replace(
        "fetch('snapshot.json').then(r => r.json()).then(apply).catch(() => {",
        "/* embedded by make_preview.py */\nconst EMBEDDED = " + json.dumps(data) + ";\napply(EMBEDDED);\nif (false) { //",
    )
    js += "\n}"
    html = index.replace('</body>', '<script>' + js + '</script></body>').replace('<script src="main.js"></script>', "")
    out = root / "preview.html"
    out.write_text(html)
    print(f"wrote {out} ({len(html)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
