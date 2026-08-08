#!/usr/bin/env bash
# neuroform play — interactive chat loop against a brain file (read-only; chat never saves).
# usage: bash tools/play.sh [brain-file] [--teacher name]
#   brain-file  default: m1.brain (the grown demo brain)
#   --teacher   attach a mock teacher for this session (e.g. --teacher amber)
#   special commands: /memory /dreams /sleep /inspect /quit
set -u
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

BIN="$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')/release/neuroform.exe"
[ -f "$BIN" ] || { echo "building (first run takes a minute)..."; cargo build --release >/dev/null 2>&1 || { echo "build failed"; exit 1; }; }

BRAIN="${1:-m1.brain}"; shift || true
TEACHER=""
if [ "${1:-}" = "--teacher" ]; then TEACHER="${2:-amber}"; fi
[ -f "$BRAIN" ] || { echo "no such brain: $BRAIN"; exit 1; }

echo "=== NEUROFORM PLAY ==="
echo "brain: $BRAIN   teacher: ${TEACHER:-none (speaks from memory alone)}"
echo "type a line to talk. /memory /dreams /sleep /inspect /quit"
echo
while IFS= read -r -p "you> " line; do
  case "$line" in
    /quit|/exit|q) break ;;
    /memory) "$BIN" retrieve "$BRAIN" --query "" --k 5 2>&1 | head -8 ;;
    /dreams) "$BIN" dreams "$BRAIN" --top 3 2>&1 | head -5 ;;
    /sleep)  "$BIN" inspect "$BRAIN" 2>&1 | grep -E "sleep|pressure" ;;
    /inspect) "$BIN" inspect "$BRAIN" 2>&1 | head -9 ;;
    "") ;;
    *)
      if [ -n "$TEACHER" ]; then
        "$BIN" chat "$BRAIN" "$line" --teacher "mock:$TEACHER" 2>&1 | grep -E "^(user|file):"
      else
        "$BIN" chat "$BRAIN" "$line" 2>&1 | grep -E "^(user|file):"
      fi ;;
  esac
done
echo "bye."
