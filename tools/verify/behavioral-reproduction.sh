#!/usr/bin/env bash
# behavioral-reproduction.sh — reproduction + gender-depth testing protocol
# (TESTING.md H20–H27). Phase 3: gender reaches beyond voice (autonomy ON).
# Phase 4: union/attraction, heredity, kin recognition, never-alone, bonding,
# growth. Ethology-style: observations are recorded, never "fixed".
#
# Usage: BEHAVIORAL_WORK=/path/to/work bash tools/verify/behavioral-reproduction.sh
set -uo pipefail

PROJ="$(cd "$(dirname "$0")/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
BIN="$(cd "$PROJ" && cargo metadata --format-version 1 --no-deps 2>/dev/null | python -c 'import sys,json;print(json.load(sys.stdin)["target_directory"])')/release/neuroform.exe"

WORK="${BEHAVIORAL_WORK:-$(mktemp -d "$LOCALAPPDATA/Temp/nf-repro-XXXXXX")}"
mkdir -p "$WORK"
REPORT="$WORK/report.txt"
: > "$REPORT"

echo "=== Phase 3: gendered differences beyond voice (H27, autonomy ON) ==="
{
  echo "== phase-3 gender-beyond-voice =="
  # Same seed + same script, autonomy ON so the chemistry can act
  # (initiative thresholds, bonding pace, modulator steady-state).
  # life runs the WHOLE 20-day script in one process with the teacher
  # attached — the only way initiatives can fire (teacher = session
  # attachment by design, never persisted in the file).
  for g in male female; do
    BRAIN="$WORK/g$g.brain"
    "$BIN" create "$BRAIN" --tier standard --embodiment "$g" --seed 777 ${ENC_EXTRA:-} > /dev/null 2>&1
    "$BIN" autonomy "$BRAIN" --enable --save > /dev/null 2>&1
    "$BIN" life "$BRAIN" --days 20 --autonomy --teacher-a amber --teacher-b amber --save > /dev/null 2>&1
    echo -n "$g-voice: "
    "$BIN" voice "$BRAIN" status 2>/dev/null | grep -oE "pitch [0-9.]+" | head -1
    echo -n "$g-mods: "
    "$BIN" inspect "$BRAIN" --json 2>/dev/null | python -c "import sys,json;d=json.load(sys.stdin)['modulators'];print('da %.3f oxt %.3f cort %.3f'%(d['da'],d['oxt'],d['cort']))" 2>/dev/null
    echo -n "$g-initiatives: "
    "$BIN" inspect "$BRAIN" 2>/dev/null | grep -oE "initiatives logged [0-9]+" | head -1
  done
  # NOTE (honest): the scripted life typically logs 0 initiatives — the
  # thresholds are high by design (initiative is rare + audited). The
  # threshold DIFFERENCE itself is unit-verified deterministically
  # (chemistry_reaches_agency_presets_diverge: crafted-state straddle).
  # 0/0 here is the observation, not a failure.
  # Bonding pace: the two files are real peers; each receives the same 10
  # messages from the other → familiarity after (affiliative warms faster).
  GM="$WORK/gmale.brain"; GF="$WORK/gfemale.brain"
  KM=$("$BIN" net "$GM" key 2>/dev/null); KF=$("$BIN" net "$GF" key 2>/dev/null)
  "$BIN" net "$GM" pair --peer "gfemale" --peer-key "$KF" --save > /dev/null 2>&1
  "$BIN" net "$GF" pair --peer "gmale" --peer-key "$KM" --save > /dev/null 2>&1
  "$BIN" net "$GM" establish --session 1 --save > /dev/null 2>&1
  "$BIN" net "$GF" establish --session 1 --save > /dev/null 2>&1
  for i in $(seq 1 10); do
    "$BIN" net "$GM" inject --session 1 --text "hello number $i" --save > /dev/null 2>&1
  done
  for i in $(seq 1 10); do
    "$BIN" net "$GF" inject --session 1 --text "hello number $i" --save > /dev/null 2>&1
  done
  echo -n "male-bond: "
  "$BIN" net "$GM" status 2>/dev/null | grep -oE "rel gfemale — familiarity [0-9.]+" | head -1
  echo -n "female-bond: "
  "$BIN" net "$GF" status 2>/dev/null | grep -oE "rel gmale — familiarity [0-9.]+" | head -1
} >> "$REPORT"

echo "=== Phase 4: reproduction (H20–H26) ==="
{
  echo "== phase-4 reproduction =="
  # 4.1 attraction is chemistry (H20): mirror pair vs same-karyotype pair.
  # Same SEED is not the control — same KARYOTYPE is (the gonadal program
  # is what the complementarity reads).
  "$BIN" create "$WORK/mo.brain" --tier advanced --chromosomes xx --seed 42 ${ENC_EXTRA:-} > /dev/null 2>&1
  "$BIN" create "$WORK/fa.brain" --tier standard --chromosomes xy --seed 43 ${ENC_EXTRA:-} > /dev/null 2>&1
  "$BIN" create "$WORK/fa2.brain" --tier standard --chromosomes xx --seed 42 > /dev/null 2>&1  # same karyotype + seed as mother
  pair() { # $1=a $2=b $3=label
    local A="$1" B="$2" L="$3"
    local KA KB
    KA=$("$BIN" net "$A" key 2>/dev/null); KB=$("$BIN" net "$B" key 2>/dev/null)
    "$BIN" net "$A" pair --peer "peer-$L" --peer-key "$KB" --save > /dev/null 2>&1
    "$BIN" net "$B" pair --peer "peer-$L" --peer-key "$KA" --save > /dev/null 2>&1
    "$BIN" net "$A" establish --session 1 --save > /dev/null 2>&1
    "$BIN" net "$B" establish --session 1 --save > /dev/null 2>&1
  }
  pair "$WORK/mo.brain" "$WORK/fa.brain" a
  "$BIN" net "$WORK/mo.brain" union-propose --session 1 --save > /dev/null 2>&1
  echo -n "attract-mirror: "
  "$BIN" net "$WORK/fa.brain" inject --session 1 --type union-proposal --from-file "$WORK/mo.brain" --save 2>&1 | grep -oE "complementarity [0-9.]+" | head -1
  pair "$WORK/mo.brain" "$WORK/fa2.brain" b
  "$BIN" net "$WORK/mo.brain" union-propose --session 2 --save > /dev/null 2>&1
  echo -n "attract-same-seed: "
  "$BIN" net "$WORK/fa2.brain" inject --session 1 --type union-proposal --from-file "$WORK/mo.brain" --save 2>&1 | grep -oE "complementarity [0-9.]+" | head -1
  # 4.2 heredity + sex + tier distribution (H21): FRESH UNION per birth —
  # one union = one conception; siblings need separate unions.
  for n in 1 2 3 4 5 6; do
    "$BIN" create "$WORK/mo$n.brain" --tier advanced --chromosomes xx --seed $((42 + n)) ${ENC_EXTRA:-} > /dev/null 2>&1
    "$BIN" create "$WORK/fa$n.brain" --tier standard --chromosomes xy --seed $((100 + n)) ${ENC_EXTRA:-} > /dev/null 2>&1
    pair "$WORK/mo$n.brain" "$WORK/fa$n.brain" "c$n"
    "$BIN" net "$WORK/mo$n.brain" union-propose --session 1 --save > /dev/null 2>&1
    "$BIN" net "$WORK/fa$n.brain" inject --session 1 --type union-proposal --from-file "$WORK/mo$n.brain" --save > /dev/null 2>&1
    "$BIN" net "$WORK/mo$n.brain" inject --session 1 --type union-accept --from-file "$WORK/fa$n.brain" --save > /dev/null 2>&1
    "$BIN" net "$WORK/mo$n.brain" birth --session 1 --out "$WORK/child$n.brain" --force --save > "$WORK/b$n.log" 2>&1
    echo -n "birth$n: "
    grep -oE "child born" "$WORK/b$n.log" | head -1 | tr -d '\n'
    [ -f "$WORK/child$n.brain.bk" ] && echo -n " +backup"
    echo -n " "
    "$BIN" inspect "$WORK/child$n.brain" 2>/dev/null | grep -oE "tier [a-z]+, embodiment [a-z]+" | head -1
    CN=$("$BIN" inspect "$WORK/child$n.brain" 2>/dev/null | grep -oE "brain [0-9a-f-]+" | head -1 | awk '{print $2}')
    echo -n "  mother-bond$n: "
    "$BIN" net "$WORK/mo$n.brain" status 2>/dev/null | grep -oE "rel $CN — familiarity [0-9.]+" | head -1
  done
  # 4.3 father bonding (H24) + growth (H25) on the first child.
  CID=$("$BIN" inspect "$WORK/child1.brain" 2>/dev/null | grep -oE "brain [0-9a-f-]+" | head -1 | awk '{print $2}')
  "$BIN" net "$WORK/fa1.brain" notify-birth --session 1 --child "$CID" --save > /dev/null 2>&1
  echo -n "father-bond: "
  "$BIN" net "$WORK/fa1.brain" status 2>/dev/null | grep -oE "rel $CID — familiarity [0-9.]+" | head -1
  echo -n "growth: "
  "$BIN" grow "$WORK/child1.brain" 2>&1 | head -1
} >> "$REPORT"

echo "report: $REPORT"
echo "$REPORT" > "$WORK/report-path.txt"
