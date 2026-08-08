#!/usr/bin/env python3
"""media-extract.py — deterministic, local image feature extraction for the
Drawing organ's reference board (DESIGN.md §5.8, §9).

Two encoders, chosen at BRAIN CREATION and immutable per file:

  handcrafted (default): the ORIGINAL extractor, unchanged — a 16-dim feature
    vector over {hue histogram, saturation, value, aspect, size, edge density,
    warmth} plus dimensions. Pure PIL, no learned models, no network.

  jepa: frozen V-JEPA 2 backbone (facebook/vjepa2-vitl-fpc64-256, exported to
    ONNX by tools/export_vjepa2_onnx.py) -> 1024-dim embedding. Preprocessing
    matches the official VJEPA2VideoProcessor: resize shortest edge to 292,
    center-crop 256x256, /255, ImageNet mean/std. The model's tubelet_size is
    2, so the single frame is repeated twice into a (1,2,3,256,256) clip.
    ONNX Runtime is pinned to 1 thread for bit-exact determinism.

  onnx: frozen DINOv2-small (onnx-community/dinov2-small, fp32) -> 384-dim
    embedding. Preprocessing matches the official DINOv2 processor: resize
    shortest edge to 224, center-crop 224x224, /255, ImageNet mean/std.
    Same determinism contract (1 thread).

Usage: python media-extract.py <image-path> [--encoder handcrafted|onnx|jepa]
Output: JSON {"width":.., "height":.., "features":[...]} or {"error": ".."}
"""
import json
import math
import sys
from pathlib import Path


def handcrafted(img):
    """Original 16-dim extractor — verbatim, do not modify."""
    width, height = img.size
    small = img.resize((64, 64))
    px = small.load()
    hue_hist = [0.0] * 8
    sat_sum = 0.0
    val_sum = 0.0
    edge = 0.0
    r_minus_b = 0.0
    b_minus_y = 0.0
    n = 0
    for y in range(64):
        for x in range(64):
            r, g, b = px[x, y]
            rf, gf, bf = r / 255.0, g / 255.0, b / 255.0
            mx, mn = max(rf, gf, bf), min(rf, gf, bf)
            delta = mx - mn
            if delta < 1e-4:
                h = 0.0
            elif mx == rf:
                h = 60.0 * (((gf - bf) / delta) % 6.0)
            elif mx == gf:
                h = 60.0 * ((bf - rf) / delta + 2.0)
            else:
                h = 60.0 * ((rf - gf) / delta + 4.0)
            hue_hist[int((h / 360.0) * 8) % 8] += 1.0
            sat_sum += 0.0 if mx == 0 else delta / mx
            val_sum += mx
            r_minus_b += rf - bf
            b_minus_y += bf - (rf + gf) / 2.0
            # edge estimate: neighbor luminance delta
            if x < 63:
                l1 = 0.299 * r + 0.587 * g + 0.114 * b
                r2, g2, b2 = px[x + 1, y]
                l2 = 0.299 * r2 + 0.587 * g2 + 0.114 * b2
                edge += abs(l1 - l2)
            n += 1
    total = float(n)
    hue = [c / total for c in hue_hist]
    sat = sat_sum / total
    val = val_sum / total
    edge_density = min(1.0, edge / (64.0 * 63.0 * 255.0) * 4.0)
    aspect = math.log(max(width, 1) / max(height, 1))
    features = [
        *hue,
        sat,
        val,
        edge_density,
        min(1.0, math.log(width * height) / 24.0),
        (r_minus_b / total + 1.0) / 2.0,  # warmth 0..1
        (b_minus_y / total + 1.0) / 2.0,  # blue-yellow 0..1
        math.log(max(width, 1)) / 12.0,
        math.log(max(height, 1)) / 12.0,
    ]
    # 8 + 8 = 16 dims
    return width, height, [round(v, 6) for v in features]


def jepa(img):
    """Frozen V-JEPA 2 backbone (ONNX) -> 1024-dim embedding, L2-normalized."""
    try:
        import numpy as np
        import onnxruntime as ort
    except ImportError as exc:
        return None, None, {"error": f"jepa venv missing a dependency: {exc}"}

    model_path = (
        Path(__file__).resolve().parent.parent
        / "models" / "vjepa2-vitl-fpc64-256" / "vjepa2_backbone.onnx"
    )
    if not model_path.exists():
        return None, None, {
            "error": f"jepa backbone missing: {model_path} — run tools/export_vjepa2_onnx.py"
        }

    # Bit-exact determinism: single thread, no logging.
    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.log_severity_level = 3
    sess = ort.InferenceSession(str(model_path), so, providers=["CPUExecutionProvider"])

    # Official VJEPA2VideoProcessor pipeline:
    # resize shortest edge -> 292 (bilinear), center-crop 256x256,
    # rescale /255, normalize with ImageNet mean/std.
    width, height = img.size
    img = img.convert("RGB")
    short = min(width, height)
    scale = 292.0 / short
    nw, nh = max(1, round(width * scale)), max(1, round(height * scale))
    img = img.resize((nw, nh), 2)  # 2 = bilinear
    left = (nw - 256) // 2
    top = (nh - 256) // 2
    img = img.crop((left, top, left + 256, top + 256))
    arr = np.asarray(img, dtype=np.float32) / 255.0
    mean = np.array([0.485, 0.456, 0.406], dtype=np.float32)
    std = np.array([0.229, 0.224, 0.225], dtype=np.float32)
    arr = (arr - mean) / std
    arr = arr.transpose(2, 0, 1)  # CHW
    # tubelet_size = 2: repeat the single frame -> (1, 2, 3, 256, 256)
    clip = np.stack([arr, arr], axis=0)[None, ...]
    emb = sess.run(None, {sess.get_inputs()[0].name: clip})[0]
    vec = emb.reshape(-1)
    norm = float(np.linalg.norm(vec))
    if norm > 1e-9:
        vec = vec / norm
    # Full precision — no rounding (1024 floats; rounding would cost quality).
    return width, height, [float(v) for v in vec]


def onnx_mode(img):
    """Frozen DINOv2-small (ONNX) -> 384-dim embedding, L2-normalized."""
    try:
        import numpy as np
        import onnxruntime as ort
    except ImportError as exc:
        return None, None, {"error": f"jepa venv missing a dependency: {exc}"}

    model_path = (
        Path(__file__).resolve().parent.parent
        / "models" / "dinov2-small" / "onnx" / "model.onnx"
    )
    if not model_path.exists():
        return None, None, {
            "error": f"dinov2 model missing: {model_path} — download it (BUILD-THE-BODY P0)"
        }

    so = ort.SessionOptions()
    so.intra_op_num_threads = 1
    so.inter_op_num_threads = 1
    so.log_severity_level = 3
    sess = ort.InferenceSession(str(model_path), so, providers=["CPUExecutionProvider"])

    # DINOv2 processor pipeline (official config): resize shortest edge ->
    # 256 (bicubic), center-crop 224x224, /255, ImageNet mean/std.
    width, height = img.size
    img = img.convert("RGB")
    short = min(width, height)
    scale = 256.0 / short
    nw, nh = max(1, round(width * scale)), max(1, round(height * scale))
    img = img.resize((nw, nh), 3)  # 3 = bicubic
    left = (nw - 224) // 2
    top = (nh - 224) // 2
    img = img.crop((left, top, left + 224, top + 224))
    arr = np.asarray(img, dtype=np.float32) / 255.0
    mean = np.array([0.485, 0.456, 0.406], dtype=np.float32)
    std = np.array([0.229, 0.224, 0.225], dtype=np.float32)
    arr = (arr - mean) / std
    arr = arr.transpose(2, 0, 1)[None, ...]  # (1, 3, 224, 224)
    out = sess.run(None, {sess.get_inputs()[0].name: arr})[0]
    # The export may return (1, D) pooled or (1, N, D) per-token — mean-pool
    # tokens in the latter case (consistent with the jepa pooling choice).
    if out.ndim == 3:
        out = out.mean(axis=1)
    vec = out.reshape(-1)
    norm = float(np.linalg.norm(vec))
    if norm > 1e-9:
        vec = vec / norm
    return width, height, [float(v) for v in vec]


def main() -> int:
    args = sys.argv[1:]
    encoder = "handcrafted"
    if "--encoder" in args:
        i = args.index("--encoder")
        if i + 1 >= len(args):
            print(json.dumps({"error": "--encoder requires a value (handcrafted|jepa)"}))
            return 2
        encoder = args[i + 1]
        del args[i : i + 2]
    if not args:
        print(json.dumps({"error": "usage: media-extract.py <image-path> [--encoder handcrafted|jepa]"}))
        return 2
    path = args[0]
    try:
        from PIL import Image
    except ImportError:
        print(json.dumps({"error": "Pillow is required: pip install pillow"}))
        return 3
    try:
        img = Image.open(path).convert("RGB")
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"error": f"cannot open image: {exc}"}))
        return 4
    if encoder == "handcrafted":
        width, height, features = handcrafted(img)
    elif encoder == "jepa":
        width, height, features = jepa(img)
        if isinstance(features, dict):
            print(json.dumps(features))
            return 5
    elif encoder == "onnx":
        width, height, features = onnx_mode(img)
        if isinstance(features, dict):
            print(json.dumps(features))
            return 5
    else:
        print(json.dumps({"error": f"unknown encoder: {encoder} (handcrafted|onnx|jepa)"}))
        return 2
    print(json.dumps({"width": width, "height": height, "features": features}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
