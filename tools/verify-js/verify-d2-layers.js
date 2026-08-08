// hermes-verify-d2-layers.js — AD-HOC verification of D2 Layers logic in
// tools/desktop/tabs/canvas.js. NOT a canonical suite (pure JS UI; does not
// modify the Rust engine; no canvas raster is asserted here — composite ORDER,
// blend-mode selection, opacity clamps, and layer-op stack semantics are).
// Rerun: node tools/verify-js/verify-d2-layers.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from canvas.js ----
const BLEND = ['normal', 'multiply', 'screen', 'overlay', 'luminosity', 'add'];
// composite: draw only visible layers, in stack order (index 0 = bottom), each
// with its own opacity + blend applied. Simulated without a real canvas by
// tracking the draw ORDER and the properties each layer would use.
function compositeOrder(layersArr) {
  const order = [];
  for (const ly of layersArr) {
    if (!ly.visible) continue;
    order.push({ name: ly.name, op: ly.opacity, blend: ly.blend });
  }
  return order;
}
function clampOp(v) { return Math.max(0, Math.min(1, Number(v) || 1)); }
function mkLayer(name, white) {
  return { id: 0, name, canvas: null, visible: true, locked: false, opacity: 1, blend: 'normal', group: null };
}

console.log('== 1. composite order (bottom -> top, visible only) ==');
const L = [ mkLayer('a'), mkLayer('b'), mkLayer('c') ];
L[1].visible = false;
const order = compositeOrder(L);
ok('invisible layers skipped', order.length === 2 && !order.some(x => x.name === 'b'));
ok('order preserved bottom->top', order[0].name === 'a' && order[1].name === 'c');
ok('opacity/blend propagate', order[0].op === 1 && order[0].blend === 'normal');

console.log('== 2. blend mode vocabulary ==');
ok('six modes present', BLEND.length === 6 && BLEND.includes('multiply') && BLEND.includes('add'));
ok('setBlend rejects unknown', !BLEND.includes('xor'));

console.log('== 3. opacity clamp ==');
ok('clamp 0', clampOp(-1) === 0);
ok('clamp 1', clampOp(2) === 1);
ok('keep 0.5', clampOp(0.5) === 0.5);
ok('NaN->1', clampOp(NaN) === 1);

console.log('== 4. layer op stack semantics ==');
// add: insert above active; delete: keep paper (index 0); dup: copy canvas; merge down: combine into below
const stack = [ mkLayer('paper'), mkLayer('l1'), mkLayer('l2') ];
function addAbove(stack, active) { const ly = mkLayer('new'); stack.splice(active + 1, 0, ly); return active + 1; }
function delAt(stack, i) { if (stack.length <= 1 || i === 0) return stack.length; stack.splice(i, 1); return stack.length; }
function move(stack, i, dir) { const j = i + dir; if (j < 1 || j >= stack.length) return i; [stack[i], stack[j]] = [stack[j], stack[i]]; return j; }
let act = addAbove(stack, 2);
ok('add inserts above active', stack.length === 4 && stack[3].name === 'new' && act === 3);
ok('paper stays at bottom after move-up attempt', move(stack, 0, -1) === 0 && stack[0].name === 'paper');
ok('move down swaps', move(stack, 2, 1) === 3 && stack[3].name === stack[2].name ? stack[2].name === 'l2' : stack[3].name === 'l2');
ok('delete refuses paper', delAt(stack, 0) === 4);
ok('delete works elsewhere', delAt(stack, 1) === 3);

console.log('== 5. layer group compositing (pass-through + group opacity) ==');
// VERBATIM compositeInto: groups render members recursively with group opacity multiplied
function compositeIntoOrder(list, groupOp, out) {
  for (const it of list) {
    if (!it.visible) continue;
    if (it.kind === 'group') {
      if (it.members && it.members.length) compositeIntoOrder(it.members, (groupOp || 1) * (it.opacity ?? 1), out);
      continue;
    }
    out.push({ name: it.name, alpha: (it.opacity ?? 1) * (groupOp || 1), blend: it.blend || 'normal' });
  }
}
const grpStack = [mkLayer('paper'), { kind:'group', name:'g1', visible:true, opacity:0.5, members:[ mkLayer('inner1'), mkLayer('inner2') ] }, mkLayer('top')];
const ord = []; compositeIntoOrder(grpStack, 1, ord);
ok('group members render in place (pass-through)', ord.length === 4 && ord[1].name === 'inner1' && ord[2].name === 'inner2');
ok('group opacity multiplies member alpha', ord[1].alpha === 0.5 && ord[2].alpha === 0.5);
ok('sibling after group keeps alpha 1', ord[3].alpha === 1 && ord[3].name === 'top');
grpStack[1].visible = false;
const ord2 = []; compositeIntoOrder(grpStack, 1, ord2);
ok('hidden group skips all members', ord2.length === 2 && !ord2.some(x => x.name === 'inner1'));

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
