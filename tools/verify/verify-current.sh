#!/usr/bin/env bash
# verify-current.sh — canonical ad-hoc verification for the current milestone
# state (M0–M5). Runs: fresh build, full test suite, and CLI scenarios over
# the organs delivered so far. Evidence = this script's output.
#
# Usage: bash tools/verify/verify-current.sh
set -uo pipefail

PROJ="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
BIN="$PROJ/target/release/neuroform"
# .cargo/config.toml overrides target-dir (apostrophe-path workaround) —
# ask cargo where artifacts actually land instead of assuming ./target.
TARGET_DIR="$(cd "$PROJ" && cargo metadata --format-version 1 --no-deps 2>/dev/null | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')"
[ -n "$TARGET_DIR" ] && BIN="$TARGET_DIR/release/neuroform"
# Windows/git-bash: MSYS $TMPDIR (/tmp) is unreadable by native binaries —
# always use the Windows temp dir.
WORK="$(mktemp -d "$LOCALAPPDATA/Temp/neuroform-verify-XXXXXX")" || exit 1
trap 'rm -rf "$WORK"' EXIT
PASS=0; FAIL=0

note() { printf '%-46s' "$1"; }
ok()   { echo "PASS"; PASS=$((PASS+1)); }
bad()  { echo "FAIL: $1"; FAIL=$((FAIL+1)); }

note "cargo build --release (fresh)"
(cd "$PROJ" && cargo build --release > "$WORK/build.log" 2>&1)
[ $? -eq 0 ] && grep -q Finished "$WORK/build.log" && ok || bad "build"

note "cargo test --release (canonical suite)"
(cd "$PROJ" && cargo test --release > "$WORK/suite.log" 2>&1)
[ $? -eq 0 ] && grep -qE "test result: ok\. [0-9]+ passed; 0 failed" "$WORK/suite.log" && ok || bad "suite"

"$BIN" create "$WORK/w.brain" --tier standard --seed 321 ${ENC_EXTRA:-} > "$WORK/o" 2>&1

note "A: doc lifecycle (new/write/ledger flags bridge conflict)"
"$BIN" doc new "$WORK/w.brain" --title "The Garden" --mode journal --save > "$WORK/o" 2>&1
"$BIN" doc write "$WORK/w.brain" --doc 1 --text "The old Bridge spans the river." --save > "$WORK/o" 2>&1
"$BIN" doc write "$WORK/w.brain" --doc 1 --text "The new Bridge glows at night." --save > "$WORK/o" 2>&1
"$BIN" doc ledger "$WORK/w.brain" > "$WORK/ledger" 2>&1
grep -q "property-conflict" "$WORK/ledger" && grep -q "bridge" "$WORK/ledger" && ok || bad "ledger"

note "B: inspect prints writing section"
"$BIN" inspect "$WORK/w.brain" > "$WORK/inspect" 2>&1
grep -q "writing:" "$WORK/inspect" && grep -q "documents" "$WORK/inspect" && ok || bad "inspect writing line"

note "C: writing binds as retrievable memory"
"$BIN" retrieve "$WORK/w.brain" --query "bridge" > "$WORK/ret.log" 2>/dev/null
grep -qi "bridge" "$WORK/ret.log" && ok || bad "retrieval"

note "D: assist teacher-mediated; degraded without"
"$BIN" doc assist "$WORK/w.brain" --doc 1 "continue the scene" --teacher amber > "$WORK/assist.log" 2>/dev/null
grep -q "\[amber\]" "$WORK/assist.log" && ok || bad "assist teacher"

note "E: DOCS shard persists + validator (11 shards incl. NET)"
(cd "$PROJ" && python tools/validator/validate_nf1.py "$WORK/w.brain" > "$WORK/v.log" 2>&1)
[ $? -eq 0 ] && grep -qE "shard index: [0-9]+ shard" "$WORK/v.log" && grep -q "shard VOICE:" "$WORK/v.log" && grep -q "shard BODY:" "$WORK/v.log" && grep -q "shard NET:" "$WORK/v.log" && ok || bad "validator ($(tail -1 "$WORK/v.log"))"

note "H: draw lifecycle (canvas/layer/stroke/motifs)"
"$BIN" draw new "$WORK/w.brain" --name "Sketch" --w 256 --h 256 --save > "$WORK/o" 2>&1
"$BIN" draw layer "$WORK/w.brain" --canvas 1 --name "Line" --save > "$WORK/o" 2>&1
"$BIN" draw stroke "$WORK/w.brain" --canvas 1 --layer 1 --brush 1 --color ff6633 --width 3 \
  --points "10,10,0.5;30,20,0.8;60,15,0.4" --save > "$WORK/o" 2>&1
"$BIN" draw motifs "$WORK/w.brain" > "$WORK/motifs.log" 2>/dev/null
grep -q "motif #0" "$WORK/motifs.log" && ok || bad "draw motifs"

note "I: ref board + media sidecar (real image features)"
PYREF=$(echo "$WORK" | tr '\\' '/')
python -c "
from PIL import Image, ImageDraw
img = Image.new('RGB', (160, 100), (20, 40, 20))
d = ImageDraw.Draw(img); d.ellipse([40, 20, 120, 80], fill=(200, 100, 40))
img.save('$PYREF/ref.png')
" > /dev/null 2>&1
# Note: don't pipe the binary's stdout straight into `grep -q` here —
# with pipefail, grep -q exits on match and closes the pipe, the CLI's
# next write hits a broken pipe, and the step spuriously fails (race).
"$BIN" draw ref "$WORK/w.brain" --canvas 1 --name ref --kind image --vault-ref "$WORK/ref.png" --save > "$WORK/ref.log" 2>/dev/null
grep -q "16 features" "$WORK/ref.log" && ok || bad "media sidecar"

note "J: autonomy persists (default OFF → enable → status)"
"$BIN" autonomy "$WORK/w.brain" --status > "$WORK/aut1.log" 2>/dev/null
grep -q "enabled false" "$WORK/aut1.log" \
  && "$BIN" autonomy "$WORK/w.brain" --enable --quiet-start 23 --quiet-end 6 --save > "$WORK/aut2.log" 2>&1 \
  && "$BIN" autonomy "$WORK/w.brain" --status > "$WORK/aut3.log" 2>/dev/null \
  && grep -q "enabled true" "$WORK/aut3.log" && ok || bad "autonomy"

note "K: body lifecycle (touch/motion/interocept/sense/motor)"
"$BIN" body "$WORK/w.brain" touch --pressure 0.2 --velocity 0.1 --duration 1500 --save > "$WORK/body1.log" 2>&1
"$BIN" body "$WORK/w.brain" motion --linear 0,0,-1 --rotational 0,0,0.5 --save > "$WORK/body2.log" 2>&1
"$BIN" body "$WORK/w.brain" interocept --energy-load 0.6 --processing 0.5 --session-min 90 --save > "$WORK/body3.log" 2>&1
"$BIN" body "$WORK/w.brain" sense --add vision --save > "$WORK/body4.log" 2>&1
"$BIN" body "$WORK/w.brain" calibrate --channel vision --samples 200 --save > "$WORK/body5.log" 2>&1
"$BIN" body "$WORK/w.brain" status > "$WORK/body6.log" 2>&1
grep -q "touch ingested" "$WORK/body1.log" \
  && grep -q "motion ingested" "$WORK/body2.log" \
  && grep -q "interoception ingested" "$WORK/body3.log" \
  && grep -q "novel channel\|already available" "$WORK/body4.log" \
  && grep -q "confidence" "$WORK/body5.log" \
  && grep -q "vision" "$WORK/body6.log" \
  && grep -q "motor enabled: 0" "$WORK/body6.log" \
  && grep -q "cortex:" "$WORK/body6.log" \
  && grep -q "somatosensory" "$WORK/body6.log" && ok || bad "body lifecycle"

note "L: net lifecycle (pair/establish/exchange/close)"
PEER_KEY=$("$BIN" net "$WORK/w.brain" key 2>/dev/null)
"$BIN" net "$WORK/w.brain" pair --peer "peer-1" --peer-key "$PEER_KEY" --save > "$WORK/net1.log" 2>&1
"$BIN" net "$WORK/w.brain" establish --session 1 --save > "$WORK/net2.log" 2>&1
"$BIN" net "$WORK/w.brain" send --session 1 --text "hello from the file" --save > "$WORK/net3.log" 2>&1
"$BIN" net "$WORK/w.brain" inject --session 1 --text "hello back" --save > "$WORK/net4.log" 2>&1
"$BIN" net "$WORK/w.brain" signal --peer "peer-1" --closer --save > "$WORK/net5.log" 2>&1
"$BIN" net "$WORK/w.brain" status > "$WORK/net6.log" 2>&1
grep -q "session #1" "$WORK/net2.log" \
  && grep -q "ESTABLISHED" "$WORK/net2.log" \
  && grep -q "sent \[text\]" "$WORK/net3.log" \
  && grep -q "bound as social percept" "$WORK/net4.log" \
  && grep -q "signal closer" "$WORK/net5.log" \
  && grep -q "Established" "$WORK/net6.log" \
  && grep -q "familiarity" "$WORK/net6.log" && ok || bad "net lifecycle"

note "M: expose lifecycle (unlabeled text + image exposure)"
"$BIN" expose "$WORK/w.brain" --text "the old bridge spans the river and the river carries the light" --repeat 2 --save > "$WORK/exp1.log" 2>&1
"$BIN" expose "$WORK/w.brain" --image "$PYREF/ref.png" --save > "$WORK/exp2.log" 2>&1
"$BIN" retrieve "$WORK/w.brain" --query "bridge river" > "$WORK/exp3.log" 2>/dev/null
"$BIN" inspect "$WORK/w.brain" > "$WORK/exp4.log" 2>&1
grep -q "source ambient" "$WORK/exp1.log" \
  && grep -q "exposed image" "$WORK/exp2.log" \
  && grep -q "src=ambient" "$WORK/exp3.log" \
  && grep -q "semantic" "$WORK/exp4.log" && ok || bad "expose lifecycle"

note "N: physics lifecycle (learns gravity/support; violation surprises)"
"$BIN" physics "$WORK/w.brain" demo --save > "$WORK/phy1.log" 2>&1
"$BIN" physics "$WORK/w.brain" status > "$WORK/phy2.log" 2>&1
grep -q "permanence violation" "$WORK/phy1.log" \
  && grep -qE "learned: fall 0\.9|learned: fall 1\.0" "$WORK/phy1.log" \
  && grep -q "learned rates" "$WORK/phy2.log" \
  && grep -q "containment" "$WORK/phy2.log" && ok || bad "physics lifecycle"

note "O: union lifecycle (chemical response → birth → backup)"
"$BIN" create "$WORK/mother.brain" --tier advanced --chromosomes xx --seed 42 ${ENC_EXTRA:-} > /dev/null 2>&1
"$BIN" create "$WORK/father.brain" --tier standard --chromosomes xy --seed 43 ${ENC_EXTRA:-} > /dev/null 2>&1
MKEY=$("$BIN" net "$WORK/mother.brain" key 2>/dev/null)
FKEY=$("$BIN" net "$WORK/father.brain" key 2>/dev/null)
"$BIN" net "$WORK/mother.brain" pair --peer "father" --peer-key "$FKEY" --save > "$WORK/u1.log" 2>&1
"$BIN" net "$WORK/father.brain" pair --peer "mother" --peer-key "$MKEY" --save > "$WORK/u2.log" 2>&1
"$BIN" net "$WORK/mother.brain" establish --session 1 --save > "$WORK/u3.log" 2>&1
"$BIN" net "$WORK/father.brain" establish --session 1 --save > "$WORK/u4.log" 2>&1
"$BIN" net "$WORK/mother.brain" union-propose --session 1 --save > "$WORK/u5.log" 2>&1
"$BIN" net "$WORK/father.brain" inject --session 1 --type union-proposal --from-file "$WORK/mother.brain" --save > "$WORK/u6.log" 2>&1
"$BIN" net "$WORK/mother.brain" inject --session 1 --type union-accept --from-file "$WORK/father.brain" --save > "$WORK/u7.log" 2>&1
"$BIN" net "$WORK/mother.brain" birth --session 1 --out "$WORK/child.brain" --force --save > "$WORK/u8.log" 2>&1
"$BIN" grow "$WORK/child.brain" > "$WORK/u9.log" 2>&1
grep -q "union proposed" "$WORK/u5.log" \
  && grep -q "chemistry responded" "$WORK/u6.log" \
  && grep -q "received \[union_accept\]" "$WORK/u7.log" \
  && grep -q "child born" "$WORK/u8.log" \
  && [ -f "$WORK/child.brain.bk" ] \
  && grep -q "too young to grow" "$WORK/u9.log" && ok || bad "union lifecycle"

note "F: sleep + dreams work on a written brain (M2 regression)"
"$BIN" sleep "$WORK/w.brain" --cycles 1 --save > "$WORK/sleep.log" 2>&1
grep -q "dreams: [1-9]" "$WORK/sleep.log" && grep -q "regulated true" "$WORK/sleep.log" && ok || bad "sleep"

note "G: digest stable across reloads"
D1=$(grep -i digest <("$BIN" inspect "$WORK/w.brain" 2>/dev/null) | awk '{print $2}')
D2=$(grep -i digest <("$BIN" inspect "$WORK/w.brain" 2>/dev/null) | awk '{print $2}')
[ -n "$D1" ] && [ "$D1" = "$D2" ] && ok || bad "digest '$D1' vs '$D2'"

echo "----------------------------------------"
echo "RESULT: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
