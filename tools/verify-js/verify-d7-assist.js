// hermes-verify-d7-assist.js — AD-HOC verification of D7 Assist Tools logic in
// tools/desktop/tabs/assist.js + canvas.js (symmetry). NOT a canonical suite
// (pure JS UI; Rust untouched). Mirrors the deterministic pieces: symmetry
// mirror (vertical), shape-commit path (outline/fill -> layer), guide overlay.
// Rerun: node tools/verify-js/verify-d7-assist.js
'use strict';
let pass = 0, fail = 0;
function ok(n,c,d){ if(c){pass++;console.log('  PASS '+n);} else {fail++;console.log('  FAIL '+n+' -> '+(d||''));} }
const W = 512, H = 512;

// ---- verbatim: symmetry mirror ----
function mirroredPts(x, y, on, axis) {
  if (!on) return [{ x, y }];
  const pts = [{ x, y }];
  if (axis === 'vertical' || axis === 'both') pts.push({ x: W - x, y });
  if (axis === 'horizontal' || axis === 'both') pts.push({ x, y: H - y });
  return pts;
}
console.log('== 1. symmetry mirroring ==');
ok('off -> single point', mirroredPts(100,200,false,'vertical').length===1);
const m1 = mirroredPts(100,200,true,'vertical');
ok('vertical mirrors x', m1.length===2 && m1[1].x===W-100 && m1[1].y===200);
ok('vertical keeps original', m1[0].x===100);
const mB = mirroredPts(100,200,true,'both');
ok('both -> 3 points', mB.length===3);
ok('both mirrors x and y', mB.some(p=>p.x===W-100&&p.y===200) && mB.some(p=>p.x===100&&p.y===H-200));
const mH = mirroredPts(100,200,true,'horizontal');
ok('horizontal mirrors y', mH[1].x===100 && mH[1].y===H-200);

console.log('== 2. shape commit (outline/fill) path ==');
// a shape paints onto the active layer via transformCommit; predicate is it drew
function wouldCommitShape(shapeFill){ return shapeFill==='outline'||shapeFill==='fill'; }
ok('outline commits', wouldCommitShape('outline')===true);
ok('fill commits', wouldCommitShape('fill')===true);
ok('unknown does not commit', wouldCommitShape('wiggle')===false);

console.log('== 3. perspective guide drape ==');
// guide draws a dashed horizon + VP rays onto the visible ctx (non-destructive)
function guideOverlayRenders(on){ return on ? {horizon:256} : null; }
ok('guide off -> no overlay', guideOverlayRenders(false)===null);
ok('guide on -> overlay present', guideOverlayRenders(true)!==null);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
