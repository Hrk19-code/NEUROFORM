#!/usr/bin/env python3
"""audio-extract.py — deterministic, local voice-feature extraction for the
Voice organ's heard-voice patterns (DESIGN.md §5.8, §7; M5).

The brain never stores raw audio; this sidecar produces the extracted
summary the Brain File actually hears: a 16-dim feature vector over
{pitch, energy, brightness, dynamics, articulation, instability}. No
network, no learned models, no hidden uploads.

Usage: python audio-extract.py <audio-file>
  WAV is read natively. Other formats (mp3, m4a, or a video file whose
  audio track you want) are transcoded through ffmpeg — if ffmpeg is on
  PATH — to 16 kHz mono PCM WAV first. That is the video-track bridge:
  `voice hear --audio some_video.mp4` extracts the speaker's envelope
  without ever storing the media.

Output: JSON {"duration": s, "features": [16 dims]} or {"error": "..."}

Feature contract (keep in lockstep with packages/brain-core/src/voice.rs):
  0 pitch_mean, 1 pitch_std, 2 rms_mean, 3 rms_std, 4 zcr_mean, 5 zcr_std,
  6 attack_mean, 7 decay_mean, 8 energy_trend, 9 gap_mean, 10 voice_ratio,
  11 seg_rate, 12 jitter, 13 shimmer, 14 duration_log, 15 crispness
All dims normalized to 0..1.
"""
import json
import math
import os
import struct
import subprocess
import sys
import tempfile
import wave

SR = 16000
FRAME = int(SR * 0.02)   # 20 ms analysis window
HOP = int(SR * 0.01)     # 10 ms hop
VOICE_RMS = 0.015        # voiced-frame energy floor
F0_MIN, F0_MAX = 60.0, 420.0


def _transcode_to_wav(path: str) -> str:
    """ffmpeg → 16 kHz mono WAV in a temp dir. Returns the temp path."""
    fd, tmp = tempfile.mkstemp(suffix=".wav")
    os.close(fd)
    try:
        r = subprocess.run(
            ["ffmpeg", "-y", "-i", path, "-ar", str(SR), "-ac", "1", "-f", "wav", tmp],
            capture_output=True, timeout=120,
        )
        if r.returncode != 0:
            raise ValueError("ffmpeg transcode failed: " + (r.stderr or b"").decode("utf-8", "replace")[-300:])
        return tmp
    except Exception:
        os.remove(tmp)
        raise


def read_wav(path: str):
    """Return (samples: list[float] -1..1 mono, duration_s)."""
    if not os.path.exists(path):
        raise ValueError(f"no such file: {path}")
    tmp = None
    try:
        with open(path, "rb") as fh:
            is_riff = fh.read(4) == b"RIFF"
        if not is_riff:
            tmp = _transcode_to_wav(path)
            path = tmp
        with wave.open(path, "rb") as w:
            nch, sw, sr, n = w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes()
            raw = w.readframes(n)
        fmt = {1: "<B", 2: "<h", 4: "<i"}[sw]
        # Repeat only the format code, not the endian prefix: '<h'*N would
        # produce '<h<h<h...' (invalid — struct.error: bad char in format).
        samples = struct.unpack(fmt[0] + fmt[1:] * (len(raw) // sw), raw)
        if sw == 1:
            samples = [(s - 128.0) / 128.0 for s in samples]
        else:
            scale = 1.0 / (1 << (8 * sw - 1))
            samples = [s * scale for s in samples]
        if nch > 1:
            samples = [sum(samples[i * nch:(i + 1) * nch]) / nch for i in range(n // nch)]
        return samples, n / float(sr)
    finally:
        if tmp:
            os.remove(tmp)


def autocorr_pitch(frame, sr):
    """Normalized autocorrelation pitch (Hz) for a voiced frame; None if
    unvoiced. Deterministic."""
    n = len(frame)
    mean = sum(frame) / n
    x = [v - mean for v in frame]
    energy = sum(v * v for v in x)
    if energy < 1e-9:
        return None
    lo = int(sr / F0_MAX)
    hi = int(sr / F0_MIN)
    best_lag, best_r = lo, 0.0
    for lag in range(lo, min(hi, n)):
        num = sum(x[i] * x[i + lag] for i in range(n - lag))
        den = energy
        if den > 0:
            r = num / den
            if r > best_r:
                best_r, best_lag = r, lag
    if best_r < 0.3:
        return None
    return sr / best_lag


def main() -> int:
    if len(sys.argv) < 2:
        print(json.dumps({"error": "usage: audio-extract.py <audio-file>"}))
        return 2
    path = sys.argv[1]
    try:
        samples, dur = read_wav(path)
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"error": f"cannot read audio: {exc}"}))
        return 4

    # Frame statistics.
    n_frames = max(1, (len(samples) - FRAME) // HOP)
    rms = []
    zcr = []
    pitch = []   # Hz, only voiced frames
    voiced = []  # bool per frame
    for i in range(n_frames):
        f = samples[i * HOP:i * HOP + FRAME]
        e = sum(v * v for v in f) / max(1, len(f))
        r = math.sqrt(e)
        z = sum(1 for k in range(1, len(f)) if (f[k - 1] < 0) != (f[k] < 0)) / max(1, len(f))
        rms.append(r)
        zcr.append(z)
        if r > VOICE_RMS:
            p = autocorr_pitch(f, SR)
            if p is not None:
                pitch.append(p)
                voiced.append(True)
            else:
                voiced.append(False)
        else:
            voiced.append(False)

    # Voiced segments (contiguous runs).
    segs = []
    start = None
    for i, v in enumerate(voiced + [False]):
        if v and start is None:
            start = i
        elif not v and start is not None:
            segs.append((start, i - 1))
            start = None

    # Segment-level dynamics: attack/decay times, gaps.
    attacks, decays, gaps = [], [], []
    for si, (a, b) in enumerate(segs):
        seg_rms = rms[a:b + 1]
        peak = max(seg_rms)
        if peak <= 0:
            continue
        pi = seg_rms.index(peak)
        attacks.append(pi / 50.0)                      # rise frames → 0..1
        decays.append((len(seg_rms) - 1 - pi) / 50.0)  # fall frames → 0..1
        if si > 0:
            gaps.append((a - segs[si - 1][1] - 1) * HOP / SR / 2.0)  # seconds → 0..1
    def mean(xs, default):
        return sum(xs) / len(xs) if xs else default
    attack = mean(attacks, 0.3)
    decay = mean(decays, 0.3)
    gap = mean(gaps, 0.3)

    # Instability measures (only over voiced frames).
    jitter, shimmer = 0.0, 0.0
    vp = [p for p, v in zip(pitch, voiced) if v]
    vr = [r for r, v in zip(rms, voiced) if v]
    if len(vp) > 1 and mean(vp, 0) > 0:
        jitter = mean([abs(vp[k] - vp[k - 1]) for k in range(1, len(vp))], 0) / mean(vp, 1)
    if len(vr) > 1 and mean(vr, 0) > 0:
        shimmer = mean([abs(vr[k] - vr[k - 1]) for k in range(1, len(vr))], 0) / mean(vr, 1)

    # Energy trend: slope of frame RMS over time, normalized to 0..1.
    trend = 0.5
    if n_frames > 2:
        t = list(range(n_frames))
        tm = sum(t) / len(t)
        rm = sum(rms) / len(rms)
        num = sum((t[i] - tm) * (rms[i] - rm) for i in range(n_frames))
        den = sum((t[i] - tm) ** 2 for i in range(n_frames))
        if den > 0:
            slope = num / den * 1e4  # per-frame → scaled
            trend = min(1.0, max(0.0, (slope + 1.0) / 2.0))

    pm = mean(vp, 200.0) / F0_MAX
    ps = (mean([abs(p - mean(vp, 200.0)) for p in vp], 0) / F0_MAX) if vp else 0.0
    rm_ = mean(rms, 0.05) * 4.0
    rs = (mean([abs(r - mean(rms, 0.05)) for r in rms], 0) * 4.0) if rms else 0.0
    zm = mean(zcr, 0.05) / 0.4
    zs = (mean([abs(z - mean(zcr, 0.05)) for z in zcr], 0) / 0.4) if zcr else 0.0
    vratio = sum(voiced) / n_frames
    seg_rate = (len(segs) / dur / 6.0) if dur > 0 else 0.0
    crisp = mean([z for z, v in zip(zcr, voiced) if v], 0.06) / 0.3
    durlog = math.log(dur + 1.0) / math.log(601.0) if dur > 0 else 0.0

    features = [
        min(1.0, pm), min(1.0, ps), min(1.0, rm_), min(1.0, rs),
        min(1.0, zm), min(1.0, zs), min(1.0, attack), min(1.0, decay),
        trend, min(1.0, gap), min(1.0, vratio), min(1.0, seg_rate),
        min(1.0, jitter), min(1.0, shimmer), min(1.0, durlog), min(1.0, crisp),
    ]
    print(json.dumps({
        "duration": round(dur, 3),
        "features": [round(v, 6) for v in features],
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
