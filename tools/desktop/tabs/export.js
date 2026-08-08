// Writing tab — W7: Export / Import.
// Export the current doc or the whole project to Markdown (.md), plain text
// (.txt), single-file styled HTML, and full project JSON. Import: .md/.txt ->
// a new doc; project JSON -> restore the library structure + each doc's text.
// Acceptance: round-trip a project through JSON; export md opens clean.
// Source of truth = the sidecar (writing/<brain>/index.json library + the
// per-doc doc-<id>.json texts) served through /api/file/read.
(function () {
  const { $, p } = window.App;

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }

  async function readFile(rel) {
    try { const r = await fetch('/api/file/read?path=' + encodeURIComponent(rel)).then(x => x.json()); return (r && r.ok) ? r.content : null; } catch (e) { return null; }
  }
  async function readIndex() {
    const raw = await readFile(sidecarDir() + '/index.json');
    if (!raw) return null;
    try { return JSON.parse(raw); } catch (e) { return null; }
  }
  async function readDoc(id) {
    const raw = await readFile(sidecarDir() + '/doc-' + id + '.json');
    if (!raw) return null;
    try { return JSON.parse(raw); } catch (e) { return null; }
  }
  function esc(s) { return String(s == null ? '' : s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c])); }

  // collect ordered docs from the library (story chapters/scenes, notes, journal)
  function orderedDocs(lib) {
    const out = [];
    const push = (title, docId, kind) => { if (docId != null) out.push({ title, docId, kind }); };
    for (const ch of lib.story || []) for (const sc of ch.scenes || []) push(sc.title, sc.docId, 'scene');
    for (const n of lib.notes || []) push(n.title, n.docId, 'note');
    if (lib.journal) push(lib.journal.title, lib.journal.docId, 'journal');
    return out;
  }

  // ---------- exporting ----------
  function mdOf(doc) { return '# ' + (doc.title || 'Untitled') + '\n\n' + (doc.text || '') + '\n'; }
  function txtOf(doc) { return (doc.title || 'Untitled') + '\n' + '='.repeat((doc.title || 'U').length) + '\n\n' + (doc.text || '') + '\n'; }
  function htmlOf(title, docs) {
    const body = docs.map(d =>
      '<h1>' + esc(d.title || 'Untitled') + '</h1>' +
      (d.kind ? '<p class="meta">' + esc(d.kind) + '</p>' : '') +
      '<div class="body">' + esc(d.text || '').replace(/\n/g, '</p><p>') + '</div>'
    ).join('\n');
    return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${esc(title)}</title>
<style>body{max-width:720px;margin:40px auto;padding:0 20px;font:16px/1.7 Georgia,serif;color:#2b2b2b;background:#fafafa}
h1{font-size:26px;border-bottom:2px solid #333;padding-bottom:6px;margin-top:40px}
.meta{font:12px/1.4 sans-serif;color:#888;text-transform:uppercase;letter-spacing:1px}
.body{white-space:pre-wrap}</style></head><body>${body}</body></html>`;
  }
  function projectJson(lib, docs) {
    return { brain: brainName(), exported: new Date().toISOString(), library: lib, docs };
  }

  async function doExportDoc() {
    const id = (window.App && window.App.currentDoc);
    const fmt = $('exFmt').value;
    if (id == null) { $('exMsg').textContent = 'open a doc first'; return; }
    const doc = await readDoc(id);
    if (!doc) { $('exMsg').textContent = 'doc sidecar missing'; return; }
    download(fmt === 'md' ? mdOf(doc) : txtOf(doc), 'doc-' + id + '.' + fmt, fmt === 'md' ? 'text/markdown' : 'text/plain');
    $('exMsg').textContent = 'exported doc #' + id + ' as.' + fmt;
  }
  async function doExportAll() {
    const fmt = $('exFmt').value;
    const lib = await readIndex();
    if (!lib) { $('exMsg').textContent = 'no library to export'; return; }
    const docs = [];
    for (const o of orderedDocs(lib)) { const d = await readDoc(o.docId); if (d) docs.push({ ...d, kind: o.kind }); }
    const title = brainName();
    if (fmt === 'md') { download(docs.map(mdOf).join('\n\n---\n\n'), title + '.md', 'text/markdown'); }
    else if (fmt === 'txt') { download(docs.map(txtOf).join('\n\n'), title + '.txt', 'text/plain'); }
    else if (fmt === 'html') { download(htmlOf(title, docs), title + '.html', 'text/html'); }
    else { download(JSON.stringify(projectJson(lib, docs), null, 1), title + '.project.json', 'application/json'); }
    $('exMsg').textContent = 'exported project (' + docs.length + ' docs) as.' + fmt;
  }
  function download(content, name, type) {
    const blob = new Blob([content], { type: type || 'text/plain' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = name;
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }

  // ---------- importing ----------
  function onImport(file) {
    const kind = $('exImportKind').value;      // 'text' | 'project'
    const reader = new FileReader();
    reader.onload = async () => {
      const content = String(reader.result || '');
      if (kind === 'project') await importProject(content);
      else await importText(content, file.name);
    };
    reader.readAsText(file);
  }
  async function importText(content, name) {
    // title = filename sans extension; body = the whole text
    const title = (name || 'imported').replace(/\.[^.]+$/, '') || 'imported';
    const id = await newDoc(title);
    if (id == null) { $('exMsg').textContent = 'import failed: could not create doc'; return; }
    // write the full text into the doc
    const ok = await writeReplace(id, content);
    $('exMsg').textContent = ok ? 'imported "' + title + '" → doc #' + id : 'import failed writing text';
  }
  async function newDoc(title) {
    try {
      const r = await fetch('/api/run?args=' + encodeURIComponent(['doc', 'new', p(), '--title', title, '--save'].join('\u001f'))).then(x => x.json());
      const m = r && /#(\d+)/.exec(r.stdout || '');
      return m ? Number(m[1]) : null;
    } catch (e) { return null; }
  }
  async function writeReplace(id, text) {
    try {
      const r = await fetch('/api/run?args=' + encodeURIComponent(['doc', 'replace', p(), '--doc', String(id), '--text', text, '--save'].join('\u001f'))).then(x => x.json());
      return !!(r && r.ok);
    } catch (e) { return false; }
  }
  async function importProject(content) {
    let data;
    try { data = JSON.parse(content); } catch (e) { $('exMsg').textContent = 'import failed: not valid project JSON'; return; }
    if (!data || !data.library) { $('exMsg').textContent = 'import failed: missing library field'; return; }
    // write the library index back, then re-create each doc (rebinding into brain)
    await writeFile(sidecarDir() + '/index.json', JSON.stringify(data.library, null, 1));
    let n = 0;
    for (const d of data.docs || []) {
      const id = await newDoc(d.title || 'Untitled');
      if (id == null) continue;
      await writeReplace(id, d.text || '');
      n++;
    }
    $('exMsg').textContent = 'restored project: ' + n + ' docs recreated (' + (data.docs || []).length + ' in file)';
    // tell the writing tree to reload
    if (window.App && window.App.onRunDone) window.App.onRunDone();
  }
  async function writeFile(rel, content) {
    try { const r = await fetch('/api/file/write?path=' + encodeURIComponent(rel) + '&content=' + encodeURIComponent(content)).then(x => x.json()); return !!(r && r.ok); } catch (e) { return false; }
  }

  // ---------- wiring ----------
  $('btnExDoc').onclick = doExportDoc;
  $('btnExAll').onclick = doExportAll;
  $('exImportFile').addEventListener('change', (e) => { const f = e.target.files && e.target.files[0]; if (f) onImport(f); e.target.value = ''; });
})();
