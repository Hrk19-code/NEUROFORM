// hermes-verify-w3-lorebook.js — AD-HOC verification of the W3 Lorebook & Entity
// Sheets logic added to tools/desktop/tabs/lorebook.js. NOT a canonical suite.
// Mirrors the pure functions verbatim (keyword-match preview + entity field
// parse) and asserts the W3 acceptance. KEPT permanently in tools/verify-js/.
// Rerun: node tools/verify-js/verify-w3-lorebook.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from lorebook.js preview() ----
function previewTextCanon(lore, text) {
  const t = (text || '').toLowerCase();
  return lore.filter(e => {
    if (!e.enabled) return false;
    const kws = (e.keywords || []).map(k => String(k).toLowerCase()).filter(Boolean);
    return kws.some(k => t.includes(k));
  });
}
// ---- VERBATIM from lorebook.js entity field parser ----
function parseFields(str) {
  const fields = {};
  (str || '').split(';').forEach(pr => {
    const i = pr.indexOf(':');
    if (i > 0) fields[pr.slice(0, i).trim()] = pr.slice(i + 1).trim();
  });
  return fields;
}

const lore5 = [
  { id: 1, title: 'The Old Bridge', keywords: ['bridge', 'spans'], enabled: true },
  { id: 2, title: 'The Valley', keywords: ['valley', 'river'], enabled: true },
  { id: 3, title: 'Empyrean Oath', keywords: ['oath', 'swear', 'pledge'], enabled: true },
  { id: 4, title: 'The Lantern Keeper', keywords: ['lantern', 'keeper'], enabled: true },
  { id: 5, title: 'Wintering', keywords: ['winter', 'frost', 'snow'], enabled: true },
];

console.log('== 1. Lorebook keyword-match preview ==');
const hit1 = previewTextCanon(lore5, 'the old bridge over the river in winter');
ok('three fire on compound text', hit1.length === 3, 'got ' + hit1.map(h => h.title).join(','));
ok('titles correct', hit1.map(h => h.title).join('|') === 'The Old Bridge|The Valley|Wintering');
ok('nothing on unrelated', previewTextCanon(lore5, 'nothing here').length === 0);
ok('case-insensitive', previewTextCanon(lore5, 'The BRIDGE stands').some(h => h.title === 'The Old Bridge'));

console.log('== 2. Disabled entries gated ==');
const disabledWinter = lore5.map((e, i) => i === 4 ? { ...e, enabled: false } : e);
ok('disabled does not fire', previewTextCanon(disabledWinter, 'in winter the frost').length === 0);
ok('re-enabled fires again', previewTextCanon(lore5, 'in winter the frost').length === 1);

console.log('== 3. Acceptance: 5 entries each fire on own keyword ==');
const fireCounts = lore5.map(e => previewTextCanon(lore5, (e.keywords && e.keywords[0]) || 'x').some(h => h.id === e.id));
ok('all 5 fire', fireCounts.every(Boolean), JSON.stringify(fireCounts));

console.log('== 4. Entity field parsing ==');
const f = parseFields('hair: auburn; origin: the valley');
ok('parses pairs', f.hair === 'auburn' && f.origin === 'the valley');
ok('empty/odd safe', Object.keys(parseFields('')).length === 0 && Object.keys(parseFields('noy')).length === 0);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
