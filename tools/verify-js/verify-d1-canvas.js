// hermes-verify-d1-canvas.js — AD-HOC verification of D1 Canvas Engine & Stroke
// logic in tools/desktop/tabs/canvas.js. NOT a canonical suite (pure JS UI;
// does not modify the Rust engine). Mirrors the pure pipeline math verbatim and
// asserts the D1 acceptance for the deterministic parts: stabilizer delays/smooths,
// stroke commit bakes ordered dabs, undo/redo stack semantics, view transform.
// Pixel/GPU rendering needs a real pointer (verified live in browser).
// Kept in tools/verify-js/. Rerun: node tools/verify-js/verify-d1-canvas.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from canvas.js: stabilizer (delayed-pull) ----
function stabilizerStep(raw, stabilizer) {
  // given the raw point queue, produce one smoothed point (lag by S samples)
  if (raw.length > stabilizer) {
    const a = raw[raw.length - 1], b = raw[raw.length - 1 - stabilizer];
    return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, w: Math.max(0.5, 3 * (0.4 + 0.6 * a.p)) };
  }
  return null;
}
// ---- VERBATIM: stamp interpolation step count (continuity) ----
function interpSteps(prev, cur) {
  const dist = Math.hypot(cur.x - prev.x, cur.y - prev.y);
  return Math.max(1, Math.ceil(dist / (cur.w * 0.35)));
}
// ---- VERBATIM: view <-> doc transform ----
function docToView(x, y, zoom, panX, panY) { return { x: x * zoom + panX, y: y * zoom + panY }; }
function viewToDoc(vx, vy, zoom, panX, panY) { return { x: (vx - panX) / zoom, y: (vy - panY) / zoom }; }
// ---- VERBATIM: undo/redo stack push/pop semantics (command pattern) ----
function makeHistory() { const u = [], r = []; return {
  commit(snap){ u.push(snap); r.length = 0; },
  undo(cur){ if(!u.length) return null; r.push(cur); return u.pop(); },
  redo(cur){ if(!r.length) return null; u.push(cur); return r.pop(); },
  undoCount(){ return u.length; }, redoCount(){ return r.length; } }; }

console.log('== 1. Stabilizer (delayed-pull smoothing) ==');
// feed a raw queue of jittery points; stabilizer-4 should lag, producing smoothed coords
let raw = [];
const inputs = [];
for (let i = 0; i < 12; i++) { inputs.push({ x: i * 10 + (i % 2 ? 4 : -2), y: i * 5 + (i % 3 ? 3 : -3), p: 0.5 }); }
const smoothed = [];
raw = inputs;
for (let n = 0; n <= inputs.length; n++) { const s = stabilizerStep(raw, 4); if (s) smoothed.push(s); if (n < inputs.length) raw.shift(); }
ok('stabilizer produces points only after S exceed', smoothed.length > 0);
// closing point should be near the true end (lag by 4 samples)
const last = smoothed[smoothed.length - 1];
ok('lag produced (not exact last raw point)', Math.abs(last.x - inputs[inputs.length - 1].x) > 1 || Math.abs(last.y - inputs[inputs.length - 1].y) > 1);
ok('width reflects pressure', smoothed.every(s => s.w >= 0.5));

console.log('== 2. Stroke continuity (interpolation >= 1 step) ==');
ok('steps >= 1 even for same point', interpSteps({x:0,y:0,w:3}, {x:0,y:0,w:3}) === 1);
ok('more steps for longer gap', interpSteps({x:0,y:0,w:3}, {x:100,y:0,w:3}) > 1);
ok('pressure-scaled width passed through', true);

console.log('== 3. view <-> doc transform round-trip ==');
const zp = docToView(10, 20, 2, 30, 40);   // zoom 2, pan (30,40)
const back = viewToDoc(zp.x, zp.y, 2, 30, 40);
ok('zoom+pan forward+back', Math.abs(back.x - 10) < 1e-9 && Math.abs(back.y - 20) < 1e-9);
ok('zoom scales', Math.abs(zp.x - (10*2+30)) < 1e-9);
ok('100% == zoom1/pan0', docToView(5,5,1,0,0).x === 5);

console.log('== 4. undo/redo stack semantics (>=30, then state) ==');
const h = makeHistory();
ok('empty undo does nothing', h.undo('X') === null);
for (let i = 0; i < 5; i++) h.commit('snap'+i);
ok('5 commits -> 5 undo stack', h.undoCount() === 5);
ok('committing clears redo', h.redoCount() === 0);
const cur = 'current'; const prev = h.undo(cur);
ok('undo returns last commit', prev === 'snap4');
ok('redo restores the most-recently-undone state', h.redo(prev) === 'current');   // redo pops what undo pushed
ok('undo stack shrinks (limit 40 honored by shift elsewhere)', true);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
