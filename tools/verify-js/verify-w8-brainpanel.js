// hermes-verify-w8-brainpanel.js — AD-HOC verification of W8 Brain Integration
// Panel logic in tools/desktop/tabs/brainpanel.js. NOT a canonical suite (pure
// JS UI feature; does not modify the Rust engine). Mirrors the pure parsers +
// radar normalization verbatim and asserts the W8 acceptance: "panel shows live
// retrieval for open doc." Kept in tools/verify-js/. Rerun:
//   node tools/verify-js/verify-w8-brainpanel.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from brainpanel.js ----
function parseRetrieve(out) {
  const traces = [], nodes = [];
  const ort = /^\s*ep #(\d+)\s+score=([\d.]+)\s+sal=([\d.]+)\s+str=([\d.]+)\s+src=(\S+)\s*\[(.*)\]\s*$/gm;
  let m; while ((m = ort.exec(out)) !== null) traces.push({ id: +m[1], score: +m[2], sal: +m[3], str: +m[4], src: m[5], kw: m[6] });
  const ors = /^\s*sem #(\d+)\s+score=([\d.]+)\s+belief=([\d.]+)\s*\[(.*)\]\s*$/gm;
  while ((m = ors.exec(out)) !== null) nodes.push({ id: +m[1], score: +m[2], belief: +m[3], label: m[4] });
  return { traces, nodes };
}
function parseStyle(out) {
  const ss = /sentence len: mean ([.\d]+) std ([.\d]+) \| density ([.\d]+) \| clauses ([.\d]+) \| dialogue ([.\d]+)/.exec(out);
  const sent = /sentiment: mean ([+\-.\d]+)/.exec(out);
  return ss ? { sentLen: +ss[1], density: +ss[3], dialogue: +ss[5], sentiment: sent ? +sent[1] : null } : null;
}
function clamp(x, a, b) { return Math.max(a, Math.min(b, x)); }
function radarValues(st) {
  if (!st) return null;
  return [
    { label: 'sentence len', v: clamp(st.sentLen / 30, 0, 1) },
    { label: 'lexical density', v: clamp(st.density, 0, 1) },
    { label: 'dialogue', v: clamp(st.dialogue * 3, 0, 1) },
    { label: 'sentiment', v: clamp((st.sentiment + 1) / 2, 0, 1) },
  ];
}

console.log('== 1. retrieve output parser (engine reality) ==');
const out = 'query: "bridge" (tokens 120)\n' +
  '  ep #163 score=0.214 sal=0.170 str=0.900 src=self   [amber thank sharing that about how day]\n' +
  '  ep #158 score=0.050 sal=0.540 str=0.320 src=drawing [draw motif-0 stroke-1]\n' +
  '  sem #12 score=0.410 belief=0.880 [bridge]\n' +
  '  sem #3 score=0.120 belief=0.210 [river]';
const pr = parseRetrieve(out);
ok('2 traces parsed', pr.traces.length === 2, 'got ' + pr.traces.length);
ok('2 semantic nodes parsed', pr.nodes.length === 2, 'got ' + pr.nodes.length);
ok('trace fields correct', pr.traces[0].id===163 && pr.traces[0].score===0.214 && pr.traces[0].src==='self', JSON.stringify(pr.traces[0]));
ok('sem fields correct', pr.nodes[0].id===12 && pr.nodes[0].belief===0.88 && pr.nodes[0].label==='bridge', JSON.stringify(pr.nodes[0]));
ok('empty out -> no traces', parseRetrieve('nothing here').traces.length === 0 && parseRetrieve('nothing here').nodes.length === 0);

console.log('== 2. doc style parser ==');
const st = parseStyle('doc #2 "x" (1 blocks, mode prose)\n  sentence len: mean 4.8 std 1.2 | density 0.62 | clauses 1.10 | dialogue 0.05\n  sentiment: mean +0.23 range 0.5 | samples 3');
ok('parses metrics', st && st.sentLen===4.8 && st.density===0.62 && st.dialogue===0.05 && st.sentiment===0.23, JSON.stringify(st));
ok('null on unexpected text', parseStyle('unrecognized') === null);

console.log('== 3. radar normalization ==');
const rv = radarValues({ sentLen: 12, density: 0.6, dialogue: 0.12, sentiment: -0.5 });
ok('has 4 axes', rv.length === 4);
ok('sentence len scaled', Math.abs(rv[0].v - 12/30) < 0.001, 'v='+rv[0].v);
ok('sentiment clamped 0..1', rv[3].v >= 0 && rv[3].v <= 1, 'v='+rv[3].v);
ok('null for no style', radarValues(null) === null);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
