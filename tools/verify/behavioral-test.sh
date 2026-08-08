#!/usr/bin/env bash
# behavioral-test.sh — THE behavioral test protocol (docs/TESTING.md §4).
# Phase 1: comparison matrix (H16/H17/H19) — identical life for all brains.
# Phase 2: cognitive battery (H18). Results are read into docs/TESTING.md §5.
# Deterministic seeds; real media noted in the transcript.
set -uo pipefail

PROJ="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
WORK="${BEHAVIORAL_WORK:-$(mktemp -d "$LOCALAPPDATA/Temp/nf-behavioral-XXXXXX")}"
mkdir -p "$WORK"
BIN="$(cd "$PROJ" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')/release/neuroform.exe"
REPORT="$WORK/report.txt"
: > "$REPORT"

cd "$PROJ"

# --- the identical life script (every brain gets exactly this) --------------
run_life() {
  local f="$1"
  "$BIN" life "$f" --days 20 --save > /dev/null 2>&1
  "$BIN" body "$f" touch --pressure 0.35 --velocity 0.2 --duration 1200 --save > /dev/null 2>&1
  "$BIN" body "$f" motion --linear "0,0,-1" --rotational "0.2,0,0" --save > /dev/null 2>&1
  "$BIN" body "$f" interocept --energy-load 0.4 --save > /dev/null 2>&1
  "$BIN" expose "$f" --text "the old bridge spans the river and the river carries the light" --repeat 3 --save > /dev/null 2>&1
  "$BIN" physics "$f" demo --save > /dev/null 2>&1
  "$BIN" sleep "$f" --cycles 1 --save > /dev/null 2>&1
}

snapshot() {
  local f="$1" tag="$2"
  {
    echo "== $tag =="
    "$BIN" inspect "$f" 2>/dev/null | grep -E "memory:|semantic|dreams|voice|autonomy" | head -6
    "$BIN" body "$f" status 2>/dev/null | grep -E "cortex:"
    "$BIN" physics "$f" status 2>/dev/null | sed -n '2,8p'
    "$BIN" voice "$f" status 2>/dev/null | grep -iE "pitch|rate|identity" | head -3
  } >> "$REPORT"
}

echo "=== Phase 1: comparison matrix (H16/H17/H19) ==="
for spec in \
  "M1 standard male" \
  "M2 standard female" \
  "M2b standard nonbinary" \
  "M3 advanced male" \
  "M3b prototype male" \
  "M4 experimental female"; do
  set -- $spec
  tag=$1; tier=$2; preset=$3
  f="$WORK/$tag.brain"
  "$BIN" create "$f" --tier "$tier" --embodiment "$preset" --seed 42 ${ENC_EXTRA:-} > /dev/null 2>&1
  run_life "$f"
  snapshot "$f" "$tag ($tier/$preset)"
done

echo "=== Phase 1d: determinism (H19) — M1 script run twice ===" >> "$REPORT"
"$BIN" create "$WORK/d1.brain" --tier standard --embodiment male --seed 42 ${ENC_EXTRA:-} > /dev/null 2>&1
run_life "$WORK/d1.brain"
D1=$("$BIN" inspect "$WORK/d1.brain" 2>/dev/null | grep -oE "digest [0-9]+" | head -1)
D2=$("$BIN" inspect "$WORK/M1.brain" 2>/dev/null | grep -oE "digest [0-9]+" | head -1)
echo "digest d1=$D1 m1=$D2" >> "$REPORT"

echo "=== Phase 2: cognitive battery (H18) ==="
CB="$WORK/cb.brain"
"$BIN" create "$CB" --tier standard --embodiment mixed --seed 7 ${ENC_EXTRA:-} > /dev/null 2>&1
{
  echo "== battery =="
  # 1. retrieval: 10 events, query each
  for i in $(seq 1 10); do
    "$BIN" event "$CB" --text "event number $i has a unique word$i" --valence 0.1 --save > /dev/null 2>&1
  done
  for i in 1 5 10; do
    echo -n "retrieval-$i: "
    "$BIN" retrieve "$CB" --query "unique word$i" --k 1 2>/dev/null | grep -oE "ep #[0-9]+.*unique word$i" | head -1
  done
  # 2. association: repeated exposure → semantic node
  echo -n "semantic-nodes: "
  "$BIN" inspect "$CB" 2>/dev/null | grep -oE "semantic nodes, [0-9]+" | head -1
  # 3. consolidation: write then sleep
  "$BIN" doc "$CB" write --title "test" --text "the river remembers the bridge" --save > /dev/null 2>&1
  "$BIN" sleep "$CB" --cycles 1 --save > /dev/null 2>&1
  echo -n "after-sleep-strength: "
  "$BIN" inspect "$CB" 2>/dev/null | grep -oE "memory: [0-9]+ traces" | head -1
  # 5. expression: all bind with provenance
  "$BIN" draw "$CB" stroke --points "0,0,0.5;10,10,0.6;20,5,0.4" --save > /dev/null 2>&1
  echo -n "src-check: "
  "$BIN" retrieve "$CB" --query "draw" --k 1 2>/dev/null | grep -oE "src=[a-z-]+" | head -1
  # 6. interference: recency
  echo -n "oldest-trace: "
  "$BIN" retrieve "$CB" --query "unique word1" --k 3 2>/dev/null | grep -cE "unique word1"
} >> "$REPORT"

echo "report: $REPORT"
echo "$REPORT" > "$WORK/report-path.txt"
