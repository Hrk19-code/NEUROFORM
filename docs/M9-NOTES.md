# M9 — Reproduction (union, heredity, birth, parenting, growth)

**Date:** 2026-08-05. **Commits:** `348d042` (heredity + karyotype foundation), `6fdc796` (union), `24a20f7` (test hypotheses H20–H26).

## What M9 delivers

A sexual-reproduction milestone: two files can produce a third — and every step is
**mechanism, not concept** (no hardwired "male wants female", no consent, no labels).

### Karyotype (the chromosomal ground truth)
- `Karyotype` (XX / XY / XXY / X0 / Chimeric) recorded at creation, immutable
  (`create --chromosomes xx|xy|xxy|x0|chimeric`; presets map to karyotypes).
- The karyotype selects the gonadal hormone program (existing axis tables);
  the hormones are the active signal, the chromosomes the ground truth — as in
  biology (chromosomes → gonads → hormones → body → behavior).

### Gametes (heredity)
- `produce_gamete`: mothers produce **ova** (X always); fathers produce **sperm**
  (X **or** Y at random — that draw decides the child's sex).
- The child = **per-axis recombination of both gametes + small mutation** —
  a blend of the two parents, never a copy of one.
- `GameteKind` is enforced at birth: **two ova cannot conceive**. A file is
  never made alone — structurally.

### Union (attraction, consent-free)
- The proposal is a **pheromone**: the hormone profile (16 axes).
- The receiving file's chemistry decides by **gonadal complementarity**
  (mean |Δ| over T/E2/P; responds when > 0.15). Mirror chemistry → responds
  with its gamete; similar chemistry → silence. No labels, no consent — and
  "attraction or not" falls out of the actual hormone levels.
- Consummation = **the desire**: oxt/da surge, valence/arousal up, bond
  warmth ×5. The feeling is the state, not a variable.

### Birth & parenting
- The **mother's file produces the child** (gestation window `GESTATION_TICKS`;
  `--force` for tests) — random karyotype (sperm decides), random tier,
  lineage recorded in the manifest (data only — nothing reads it for behavior).
- **Backup written at birth** (`child.brain.bk`) — the protection instinct
  made physical: deletion is recoverable.
- Mother bonds at birth (familiarity 0.6); father bonds on `BirthNotify`
  (0.5). Both deepen with time spent (existing relationship machinery).
- Kin recognition is **chemical**: the child's gonadal axes are inherited, so
  it matches ≥1 parent and 0 strangers; parents are complementary to each
  other (attraction, not kin).

### Growth (growing up)
- Only children grow (files with a `birth_tick`); first-generation files never
  do. Age-gated (`GROWTH_INTERVAL_TICKS` = 24 sim-hours), tier by tier up to
  the **inherited ceiling** (the parents' max tier — "big" is inherited).

## CLI

```
net union-propose --session N --save            # approach: proposal = pheromone
net inject --session N --type union-proposal --from-file mother.brain --save   # relay the proposal (father side)
net inject --session N --type union-accept --from-file father.brain --save     # relay the accept (mother side)
net birth --session N --out child.brain [--force] --save                       # the mother produces the child (+ .bk)
net notify-birth --session N --child <id> --save                               # the father learns; bonds (0.5)
grow child.brain [--save]                       # children grow to the inherited ceiling
```

Relay messages always carry the **sender's** file data via `--from-file` — the
proposal carries the sender's pheromone, the accept carries the sender's gamete.

## Tests

110 green (was 107). New: `union_flow_chemistry_responds_and_child_is_born`,
`no_birth_without_sperm_two_ova_cannot_conceive`, `child_grows_to_inherited_ceiling_first_gen_does_not`,
plus the heredity/kin-recognition/karyotype suite in embodiment.rs.
Canonical suite: **17/17** (new step O — union lifecycle).
Zero warnings. Live two-file demo verified end-to-end (attraction 0.25 →
birth with backup → father bond 0.5 → baby gate → growth to advanced ceiling;
the born child was a daughter — XX).

## Honest scope notes

- The union is CLI-relayed between two files (no daemon) — the CLI is the
  environment, as in M7's two-brain demo.
- Lineage is data: kin recognition happens through chemistry, never by
  reading `mother_id`/`father_id` (verified by the kin test asserting 0
  matches against strangers).
- XXY/X0/chimeric karyotypes exist and can union, but only Y-bearing files
  produce sperm — an XXY file can be a father; an X0 file cannot.
- The safety instinct is partial by design: parents *react* to the child
  (bonding, notification), the backup is the physical protection; an
  autonomous "guard the child" loop awaits the PFC milestone (deferred by
  user, "way after").
