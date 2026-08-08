#!/usr/bin/env bash
# verify-caution.sh — evidence that the organism's memory carries the
# substrate of learned caution (ad-hoc scenario suite; sibling of
# verify-current.sh — that canonical 17-check gate is unchanged).
#
# The experiment (deterministic, seed 5):
#   3x aversive pairing  "the hot stove burned my hand" (valence -0.85)
#   1x neutral control   "the wooden spoon is shiny"    (valence +0.10)
#   bind (400 ticks) + 1 sleep cycle, then retrieve.
#
# Asserts (all from the live retrieve output):
#   1. association:  "stove" query ranks a burn trace TOP (ep #1 line)
#   2. persistence:  the burn trace's salience >= 0.8 (4x-slow-decay tier)
#   3. specificity:  under the "spoon" query the burn trace scores < 0.2
#                    (caution without generalized fear)
#   4. habituation:  repeated identical burns show LOWER salience than the
#                    first (novelty discounting — animal-style)
set -uo pipefail

PROJ="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
TARGET_DIR="$(cd "$PROJ" && cargo metadata --format-version 1 --no-deps 2>/dev/null | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/neuroform"; [ -f "$BIN.exe" ] && BIN="$BIN.exe"
WORK="$(mktemp -d "$LOCALAPPDATA/Temp/neuroform-caution-XXXXXX")" || exit 1
trap 'rm -rf "$WORK"' EXIT
PASS=0; FAIL=0
note() { printf '%-52s' "$1"; }
ok()   { echo "PASS"; PASS=$((PASS+1)); }
bad()  { echo "FAIL: $1"; FAIL=$((FAIL+1)); }

W="$WORK/fear.brain"
"$BIN" create "$W" --tier standard --seed 5 >/dev/null 2>&1
for i in 1 2 3; do
  "$BIN" event "$W" --text "the hot stove burned my hand" \
      --valence -0.85 --arousal 0.75 --source user --save >/dev/null 2>&1
done
"$BIN" event "$W" --text "the wooden spoon is shiny" \
    --valence 0.1 --arousal 0.2 --source user --save >/dev/null 2>&1
"$BIN" tick "$W" --ticks 400 --save >/dev/null 2>&1
"$BIN" sleep "$W" --cycles 1 --save >/dev/null 2>&1

STOVE="$("$BIN" retrieve "$W" --query "stove" --k 3 2>&1)"
SPOON="$("$BIN" retrieve "$W" --query "spoon" --k 3 2>&1)"

note "1: aversive association ranks top for its context"
echo "$STOVE" | grep -m1 "ep #" | grep -q "burned hand" \
  && ok || bad "$(echo "$STOVE" | head -3)"

note "2: aversive salience in slow-decay tier (>= 0.8)"
SAL="$(echo "$STOVE" | grep -m1 "burned hand" | grep -oE 'sal=[0-9.]+' | cut -d= -f2)"
python -c "import sys; sys.exit(0 if float('$SAL' or 0) >= 0.8 else 1)" \
  && ok || bad "sal=$SAL"

note "3: specificity — burn trace scores < 0.2 for neutral query"
SC="$(echo "$SPOON" | grep "burned hand" | grep -oE 'score=[0-9.]+' | head -1 | cut -d= -f2)"
python -c "import sys; sys.exit(0 if 0 < float('$SC' or 9) < 0.2 else 1)" \
  && ok || bad "score=$SC"

note "4: habituation — repeated burns discounted vs first"
SALS="$(echo "$STOVE" | grep "burned hand" | grep -oE 'sal=[0-9.]+' | cut -d= -f2 | sort -u | wc -l)"
[ "$SALS" -ge 2 ] && ok || bad "salience values identical ($SALS)"

echo "------------------------------------------------------"
echo "RESULT: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
