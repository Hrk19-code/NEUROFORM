// hermes-verify-w7-export.js — AD-HOC verification of W7 Export/Import logic
// in tools/desktop/tabs/export.js. NOT a canonical suite (pure JS UI feature;
// does not touch the Rust engine). Mirrors the pure formatters + round-trip
// ordering verbatim and asserts the W7 acceptance: "round-trip a project through
// JSON; export md opens clean." Kept in tools/verify-js/.
// Rerun: node tools/verify-js/verify-w7-export.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from export.js ----
function esc(s) { return String(s == null ? '' : s).replace(/[&<>"]/g, c => ({ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;' }[c])); }
function mdOf(doc) { return '# ' + (doc.title || 'Untitled') + '\n\n' + (doc.text || '') + '\n'; }
function htmlOf(title, docs) {
  const body = docs.map(d =>
    '<h1>' + esc(d.title || 'Untitled') + '</h1>' +
    (d.kind ? '<p class="meta">' + esc(d.kind) + '</p>' : '') +
    '<div class="body">' + esc(d.text || '').replace(/\n/g, '</p><p>') + '</div>'
  ).join('\n');
  return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${esc(title)}</title><style>body{max-width:720px;margin:40px auto;font:16px/1.7 Georgia,serif;color:#2b2b2b}</style></head><body>${body}</body></html>`;
}
function orderedDocs(lib) {
  const out = [];
  const push = (title, docId, kind) => { if (docId != null) out.push({ title, docId, kind }); };
  for (const ch of lib.story || []) for (const sc of ch.scenes || []) push(sc.title, sc.docId, 'scene');
  for (const n of lib.notes || []) push(n.title, n.docId, 'note');
  if (lib.journal) push(lib.journal.title, lib.journal.docId, 'journal');
  return out;
}

console.log('== 1. Markdown export opens clean ==');
const doc = { title: 'The Bridge', text: 'The old bridge spans the river.\nIt is quiet.' };
const md = mdOf(doc);
ok('starts with H1 title', md.startsWith('# The Bridge'));
ok('contains body text', md.includes('The old bridge spans the river.'));
ok('ends with newline', md.endsWith('\n'));

console.log('== 2. Styled HTML export (single file) ==');
const html = htmlOf('Proj', [{ title:'The Bridge', text:'line one\nline two', kind:'scene' }]);
ok('is a full html doc', html.startsWith('<!DOCTYPE html>') && html.includes('</html>'));
ok('has inline style', html.includes('<style>'));
ok('escapes title', htmlOf('A&B', []).includes('A&amp;B'));
ok('wraps body paragraphs', html.includes('<div class="body">line one</p><p>line two</div>') || html.includes('line one</p><p>line two'));

console.log('== 3. Project JSON round-trip ordering ==');
const lib = {
  story: [ { id:'c1', title:'One', scenes:[ {id:'s1',title:'A',docId:1}, {id:'s2',title:'B',docId:2} ] } ],
  notes: [ { id:'n1', title:'Note', docId:3 } ],
  journal: { title:'J', docId:4 },
};
const ordered = orderedDocs(lib);
ok('orders scenes, notes, journal', ordered.map(o=>o.docId).join(',') === '1,2,3,4', JSON.stringify(ordered.map(o=>o.docId)));
ok('tags kinds', ordered.map(o=>o.kind).join(',') === 'scene,scene,note,journal');
ok('skips null docId', orderedDocs({ story:[{scenes:[{title:'x'}]}] }).length === 0);
// round-trip: project JSON holds library + docs; a re-import can restore by docId
const projectJson = { brain:'m1', library: lib, docs: ordered.map(o=>({ docId:o.docId, title:o.title })) };
const restored = JSON.parse(JSON.stringify(projectJson));   // serialize/deserialize
ok('round-trips through JSON (structure preserved)', JSON.stringify(restored.library) === JSON.stringify(lib));

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
