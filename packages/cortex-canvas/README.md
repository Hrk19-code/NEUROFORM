# Cortex Canvas — scaffold

Static-snapshot renderer for the brain visualization (DESIGN.md §3.5). M0 stage:
loads a state snapshot JSON (from `neuroform inspect <file> --json --out snapshot.json`)
and renders the two hemispheres with region nodes whose brightness is bound to
live state fields — but only at snapshot granularity for now.

**M1 contract:** the renderer will bind to the 10 Hz cognitive-bus `state` channel
(§17.9) so every node tracks live metrics. The rule from §3.5 already holds here:
*no visual element may render a metric that does not exist in the state schema* —
this scaffold only reads fields that `Brain::snapshot_json()` actually emits.

## Run

```bash
cargo run -p neuroform-cli -- tick demo.brain --ticks 36000 --save --snapshot packages/cortex-canvas/snapshot.json
# then open index.html in a browser, or use tools/make_preview.py to inline the
# snapshot into a single self-contained preview.html (file:// fetch is blocked
# in some browsers):
python tools/make_preview.py packages/cortex-canvas
```

## Region → metric bindings (M0 subset)

| Region | Bound metric |
|---|---|
| Prefrontal cluster | vigilance.energy, vigilance.attentionFocus |
| Limbic cluster | affect (valence → hue, arousal → brightness) |
| Somatosensory strip | embodied.bodyComfort |
| Insula region | embodied.interoceptiveLoad |
| Vestibular cluster | embodied.motionComfort |
| Development region | development.posture, curiosity |
| Modulator ring | 8 axes → ring segment colors |
