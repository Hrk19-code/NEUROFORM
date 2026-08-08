// hermes-verify-w4-structure.js — AD-HOC verification of W4 Structure Tools
// logic in tools/desktop/tabs/structure.js. NOT a canonical suite.
// Mirrors the pure functions verbatim (timeline ordering + drag-reorder
// mutation + normalization) and asserts the W4 acceptance. KEPT permanently
// in tools/verify-js/. Rerun: node tools/verify-js/verify-w4-structure.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM: timeline ordering ----
function timelineSorted(lib) {
  const items = [];
  lib.story.forEach((ch, ci) => {
    ch.scenes.forEach(scRaw => {
      const sc = { ...scRaw, synopsis: scRaw.synopsis || '', status: scRaw.status || 'draft', date: scRaw.date != null ? scRaw.date : null };
      items.push({ scene: sc, chapterTitle: ch.title, order: sc.date != null ? String(sc.date) : ci + '.' + (scRaw.id || 0) });
    });
  });
  const dated = items.filter(i => i.scene.date != null);
  const undated = items.filter(i => i.scene.date == null);
  dated.sort((a, b) => String(a.scene.date).localeCompare(String(b.scene.date), undefined, { numeric: true }));
  return dated.concat(undated);
}
// ---- VERBATIM: drag-reorder mutation ----
function reorder(lib, srcChapterId, srcSceneId, dstChapterId, dstSceneId) {
  const srcCh = lib.story.find(c => c.id === srcChapterId);
  const sc = srcCh && srcCh.scenes.find(s => s.id === srcSceneId);
  if (!sc) return false;
  srcCh.scenes = srcCh.scenes.filter(s => s.id !== srcSceneId);
  const dstCh = lib.story.find(c => c.id === dstChapterId);
  const b = dstCh.scenes.findIndex(s => s.id === dstSceneId);
  dstCh.scenes.splice(b >= 0 ? b : dstCh.scenes.length, 0, sc);
  return true;
}
// ---- VERBATIM: normalization ----
const norm = (sc) => ({ ...sc, synopsis: sc.synopsis || '', status: sc.status || 'draft', date: sc.date != null ? sc.date : null });

console.log('== 1. Timeline ordering ==');
const lib = { story: [
  { id: 'c1', scenes: [ { id: 's1', title: 'A', date: 'Day 10' }, { id: 's2', title: 'B', date: 'Day 02' }, { id: 's3', title: 'C', date: null } ] },
  { id: 'c2', scenes: [ { id: 's4', title: 'D', date: 'Day 05' } ] },
] };
const ord = timelineSorted(lib).map(i => i.scene.title);
ok('dated numeric first (B,D,A)', ord[0]==='B' && ord[1]==='D' && ord[2]==='A', ord.join(','));
ok('undated trailing (C)', ord[3]==='C', ord.join(','));

console.log('== 2. Reorder mutation ==');
const lib2 = { story: [
  { id: 'c1', scenes: [ { id: 's1', title: 'A' }, { id: 's2', title: 'B' } ] },
  { id: 'c2', scenes: [ { id: 's3', title: 'C' } ] },
] };
ok('drop s1 on s3', reorder(lib2, 'c1', 's1', 'c2', 's3') === true);
ok('c1 = [B]', lib2.story[0].scenes.map(s=>s.id).join(',') === 's2');
ok('c2 = [A,C] (splice-at-target)', lib2.story[1].scenes.map(s=>s.id).join(',') === 's1,s3');

console.log('== 3. Normalization ==');
ok('defaults', norm({id:'x'}).status==='draft' && norm({id:'x'}).synopsis==='' && norm({id:'x'}).date===null);
ok('preserves explicit', norm({id:'x',status:'final',synopsis:'s',date:'Day 1'}).status==='final' && norm({id:'x',status:'final',synopsis:'s',date:'Day 1'}).date==='Day 1');

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
