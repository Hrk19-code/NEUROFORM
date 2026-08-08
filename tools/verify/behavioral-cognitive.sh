#!/usr/bin/env bash
# behavioral-cognitive.sh — H16 capacity stress + H18 cognitive battery re-run
# (TESTING.md). Phase 5: same load across all four tiers → capacity, not
# cognition. Phase 6: the battery with the fixed probes (exposure before
# association; src-check queries; recency by rank).
#
# Usage: BEHAVIORAL_WORK=/path/to/work bash tools/verify/behavioral-cognitive.sh
set -uo pipefail

PROJ="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
BIN="$(cd "$PROJ" && cargo metadata --format-version 1 --no-deps 2>/dev/null | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')/release/neuroform.exe"

WORK="${BEHAVIORAL_WORK:-$(mktemp -d "$LOCALAPPDATA/Temp/nf-cognitive-XXXXXX")}"
mkdir -p "$WORK"
REPORT="$WORK/report.txt"
: > "$REPORT"

echo "=== Phase 5: H16 capacity stress (same load, four tiers) ==="
{
  echo "== phase-5 capacity-stress =="
  LOAD="the old bridge spans the river and the river carries the light"
  for tier in prototype standard advanced experimental; do
    F="$WORK/t-$tier.brain"
    "$BIN" create "$F" --tier "$tier" --seed 321 ${ENC_EXTRA:-} > /dev/null 2>&1
    "$BIN" expose "$F" --text "$LOAD" --repeat 6200 --save > /dev/null 2>&1
    B1=$("$BIN" inspect "$F" --json 2>/dev/null | python -c "import sys,json;d=json.load(sys.stdin);print(d['memory']['traces'] if isinstance(d['memory'],dict) else '?')" 2>/dev/null)
    S1=$("$BIN" inspect "$F" --json 2>/dev/null | python -c "import sys,json;d=json.load(sys.stdin);print(d['memory'].get('prunedTraces',0) if isinstance(d['memory'],dict) else '?')" 2>/dev/null)
    "$BIN" sleep "$F" --cycles 2 --save > /dev/null 2>&1
    B2=$("$BIN" inspect "$F" --json 2>/dev/null | python -c "import sys,json;d=json.load(sys.stdin);print(d['memory']['traces'] if isinstance(d['memory'],dict) else '?')" 2>/dev/null)
    S2=$("$BIN" inspect "$F" --json 2>/dev/null | python -c "import sys,json;d=json.load(sys.stdin);print(d['memory'].get('prunedTraces',0) if isinstance(d['memory'],dict) else '?')" 2>/dev/null)
    NODES=$("$BIN" inspect "$F" 2>/dev/null | grep -oE "[0-9]+ semantic nodes" | head -1)
    echo "$tier: traces $B1 → $B2 (pruned $S1 → $S2), $NODES"
  done
} >> "$REPORT"

echo "=== Phase 6: H18 cognitive battery (fixed probes) ==="
{
  echo "== phase-6 battery =="
  CB="$WORK/cb.brain"
  "$BIN" create "$CB" --tier standard --embodiment mixed --seed 7 ${ENC_EXTRA:-} > /dev/null 2>&1
  # 1. retrieval: 10 events, query each
  for i in $(seq 1 10); do
    "$BIN" event "$CB" --text "event number $i has a unique word$i" --valence 0.1 --save > /dev/null 2>&1
  done
  for i in 1 5 10; do
    echo -n "retrieval-$i: "
    "$BIN" retrieve "$CB" --query "unique word$i" --k 1 2>/dev/null | grep -oE "ep #[0-9]+" | head -1
  done
  # 2. association (FIXED: exposure BEFORE the check, enough repetition) → gist
  "$BIN" expose "$CB" --text "the river remembers the bridge and the bridge remembers the river" --repeat 20 --save > /dev/null 2>&1
  "$BIN" sleep "$CB" --cycles 2 --save > /dev/null 2>&1
  echo -n "semantic-nodes: "
  "$BIN" inspect "$CB" 2>/dev/null | grep -oE "[0-9]+ semantic nodes" | head -1
  # 3. consolidation: write then sleep (FIXED: doc new first, doc write with --doc)
  "$BIN" doc new "$CB" --title "test" --save > /dev/null 2>&1
  "$BIN" doc write "$CB" --doc 1 --text "the garden grows in the light" --save > /dev/null 2>&1
  "$BIN" sleep "$CB" --cycles 1 --save > "$WORK/sleep.log" 2>&1
  echo -n "dreams: "
  grep -oE "dreams: [0-9]+" "$WORK/sleep.log" | head -1
  echo -n "traces-after: "
  "$BIN" inspect "$CB" --json 2>/dev/null | python -c "import sys,json;print(json.load(sys.stdin)['memory']['traces'])" 2>/dev/null
  # 4. prediction: physics demo → learned rates + violation surprise
  "$BIN" physics "$CB" demo --save > "$WORK/phy.log" 2>&1
  echo -n "physics: "
  grep -oE "learned: fall [0-9.]+" "$WORK/phy.log" | head -1
  echo -n "violation: "
  grep -iE "violation" "$WORK/phy.log" | head -1
  # 5. expression (FIXED: proper draw/doc syntax, src-specific queries)
  "$BIN" draw new "$CB" --name "Sketch" --w 256 --h 256 --save > /dev/null 2>&1
  "$BIN" draw layer "$CB" --canvas 1 --name "Line" --save > /dev/null 2>&1
  "$BIN" draw stroke "$CB" --canvas 1 --layer 1 --brush 1 --color ff6633 --width 3 \
    --points "10,10,0.5;30,20,0.8;60,15,0.4" --save > /dev/null 2>&1
  echo -n "src-writing: "
  "$BIN" retrieve "$CB" --query "garden grows light" --k 5 2>/dev/null | grep -oE "src=[a-z-]+" | sort | uniq -c | tr '\n' ' '; echo
  echo -n "src-drawing: "
  "$BIN" retrieve "$CB" --query "draw stroke" --k 5 2>/dev/null | grep -oE "src=[a-z-]+" | sort | uniq -c | tr '\n' ' '; echo
  # 6. recency/interference (FIXED: rank comparison old vs new)
  for i in $(seq 11 40); do
    "$BIN" event "$CB" --text "later event number $i has its own marker$i" --valence 0.1 --save > /dev/null 2>&1
  done
  echo -n "old-word1: "
  "$BIN" retrieve "$CB" --query "unique word1" --k 5 2>/dev/null | grep -oE "ep #[0-9]+" | head -1
  echo -n "new-word40: "
  "$BIN" retrieve "$CB" --query "marker40" --k 5 2>/dev/null | grep -oE "ep #[0-9]+" | head -1
} >> "$REPORT"

echo "report: $REPORT"
echo "$REPORT" > "$WORK/report-path.txt"
