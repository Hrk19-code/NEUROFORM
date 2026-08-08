// Writing tab — W5: Version History.
// Per-document snapshots persisted as writing/<brain>/versions/doc-<id>.json
// = [{ n, text, words, when, auto }]. Manual "save version" + automatic snapshot
// every N words (configurable). Version list shows word count + time; click to
// view read-only; restore writes a version's text back to the doc; diff shows a
// simple word-level highlight between any two versions. Ties into the active doc
// published by writing.js via window.App.currentDoc.
(function () {
  const { $, p, esc } = window.App;
  let versions = [];       // newest-first? we store oldest-first, render reversed
  let autoEvery = 100;     // words between automatic snapshots
  let lastAutoWords = 0;
  let selView = null;      // version index currently being viewed (read-only)
  let diffView = null;     // {older, newer} — transient diff being shown; preserved across auto-refresh

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }
  function versFile(id) { return sidecarDir() + '/versions/doc-' + id + '.json'; }

  async function readFile(rel) {
    try {
      const r = await fetch('/api/file/read?path=' + encodeURIComponent(rel)).then(x => x.json());
      return (r && r.ok) ? r.content : null;
    } catch (e) { return null; }
  }
  async function writeFile(rel, content) {
    try {
      const r = await fetch('/api/file/write?path=' + encodeURIComponent(rel) + '&content=' + encodeURIComponent(content)).then(x => x.json());
      return !!(r && r.ok);
    } catch (e) { return false; }
  }

  // ---- active doc helpers ----
  function curDoc() { return (window.App && window.App.currentDoc) || null; }
  async function curDocText() {
    const id = curDoc();
    if (id == null) return null;
    const sc = await readFile(sidecarDir() + '/doc-' + id + '.json');
    if (!sc) return null;
    try { const o = JSON.parse(sc); return o.text != null ? o.text : ''; } catch (e) { return null; }
  }
  function wordsOf(t) { const m = (t || '').trim().match(/\S+/g); return m ? m.length : 0; }

  // ---- version CRUD ----
  let lastDoc = null;
  async function loadVersions() {
    const id = curDoc();
    versions = [];
    selView = null;
    if (id !== lastDoc) { diffView = null; lastDoc = id; }  // switching docs clears a stale diff
    if (id == null) { render(); return; }
    const raw = await readFile(versFile(id));
    if (raw) { try { const a = JSON.parse(raw); if (Array.isArray(a)) versions = a; } catch (e) {} }
    $('hActive').textContent = id != null ? ('doc #' + id) : '— no open doc —';
    render();
  }
  async function persist() { const id = curDoc(); if (id != null) await writeFile(versFile(id), JSON.stringify(versions, null, 1)); }

  async function saveVersion(auto) {
    diffView = null;
    const id = curDoc();
    const text = await curDocText();
    if (id == null || text == null) return;
    const n = versions.length + 1;
    versions.push({ n, text, words: wordsOf(text), when: new Date().toISOString(), auto: !!auto });
    lastAutoWords = wordsOf(text);
    await persist();
    render();
  }
  async function maybeAuto() {
    const id = curDoc();
    const text = await curDocText();
    if (id == null || text == null) return;
    const w = wordsOf(text);
    // auto-snapshot when a fresh word-ceiling is crossed (avoid duplicate on every keystroke)
    const ceil = Math.floor(w / autoEvery) * autoEvery;
    if (ceil > lastAutoWords && ceil >= autoEvery) {
      versions.push({ n: versions.length + 1, text, words: w, when: new Date().toISOString(), auto: true });
      lastAutoWords = w;
      await persist();
      render();
    }
  }

  function restore(n) {
    const v = versions.find(x => x.n === n);
    if (!v) return;
    const id = curDoc();
    if (id == null) return;
    // write the version's text into the sidecar AND bind into the brain
    writeFile(sidecarDir() + '/doc-' + id + '.json', JSON.stringify({ id, text: v.text, updated: new Date().toISOString(), restored: true }, null, 1));
    // rebind into brain via the CLI doc replace so the file's memory reflects the restore
    fetch('/api/run?args=' + encodeURIComponent(['doc', 'replace', p(), '--doc', String(id), '--text', v.text, '--save'].join('\u001f'))).then(r => r.json()).catch(() => {});
    // refresh the editor if writing.js is watching — it polls, so this repaints soon
    selView = null;
    diffView = null;
    render();
    $('hMsg').textContent = 'restored version #' + n + ' → doc #' + id;
  }

  // ---- rendering ----
  function renderVersions() {
    const box = $('hList');
    box.innerHTML = '';
    // newest first
    const ordered = versions.slice().reverse();
    if (!ordered.length) {
      box.innerHTML = '<div class="hint">no versions yet — open a doc and save version</div>';
      return;
    }
    for (const v of ordered) {
      const li = document.createElement('li');
      li.className = 'note' + (selView === v.n ? ' on' : '');
      li.style.cssText = 'padding:4px 6px; border-bottom:1px dashed #1a2530; display:flex; gap:8px; align-items:center; cursor:pointer';
      const tag = document.createElement('span');
      tag.textContent = v.auto ? '∘ auto' : '●';
      tag.title = v.auto ? 'automatic snapshot' : 'manual snapshot';
      tag.style.color = v.auto ? 'var(--dim)' : 'var(--acc)';
      const nm = document.createElement('span');
      const d = new Date(v.when);
      const hms = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      nm.textContent = '#' + v.n + ' · ' + v.words + ' words · ' + hms;
      nm.style.flex = '1';
      nm.onclick = () => { diffView = null; selView = (selView === v.n) ? null : v.n; loadVersions(); };
      const diffB = document.createElement('span');
      diffB.textContent = '⬌';
      diffB.title = 'diff vs previous';
      diffB.style.cursor = 'pointer';
      diffB.onclick = (e) => { e.stopPropagation(); showDiff(v.n); };
      li.append(tag, nm, diffB);
      li.onclick = () => { diffView = null; selView = (selView === v.n) ? null : v.n; loadVersions(); };
      const restoreB = document.createElement('span');
      restoreB.textContent = '↩';
      restoreB.title = 'restore this version';
      restoreB.style.cursor = 'pointer';
      restoreB.onclick = (e) => { e.stopPropagation(); restore(v.n); };
      li.append(restoreB);
      box.appendChild(li);
    }
  }
  function renderView() {
    const box = $('hView');
    box.innerHTML = '';
    // preserve an active diff across auto-refresh (render, don't clobber)
    if (diffView) {
      const older = (versions.find(v => v.n === diffView.older) || {}).text || '';
      const newer = (versions.find(v => v.n === diffView.newer) || {}).text || '';
      const parts = wordDiff(older, newer);
      const pre = document.createElement('pre');
      pre.className = 'h-diff';
      for (const [k, wd] of parts) {
        if (k === '=') { pre.appendChild(document.createTextNode(wd + ' ')); }
        else if (k === '+') { const s = document.createElement('span'); s.className = 'hd-add'; s.textContent = wd + ' '; pre.appendChild(s); }
        else { const s = document.createElement('span'); s.className = 'hd-del'; s.textContent = wd + ' '; pre.appendChild(s); }
      }
      box.appendChild(pre);
      const legend = document.createElement('div');
      legend.className = 'hint';
      legend.innerHTML = '<span style="background:#1d3a2d;color:var(--ok);padding:0 4px">added</span> <span style="background:#3a1d1d;color:var(--err);padding:0 4px">removed</span>  (#' + (diffView.older) + ' → #' + (diffView.newer) + ')';
      box.appendChild(legend);
      return;
    }
    if (selView == null) { box.innerHTML = '<div class="hint">select a version to view it read-only</div>'; return; }
    const v = versions.find(x => x.n === selView);
    if (!v) return;
    const pre = document.createElement('pre');
    pre.className = 'h-pre';
    pre.textContent = v.text || '(empty)';
    box.appendChild(pre);
  }
  function render() { renderVersions(); renderView(); }

  // ---- word-level diff (LCS over words) ----
  function wordDiff(a, b) {
    const A = (a || '').split(/\s+/).filter(Boolean);
    const B = (b || '').split(/\s+/).filter(Boolean);
    const n = A.length, m = B.length;
    // LCS length table
    const dp = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
    for (let i = n - 1; i >= 0; i--) {
      for (let j = m - 1; j >= 0; j--) {
        dp[i][j] = A[i] === B[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
      }
    }
    // walk to build marked output
    let i = 0, j = 0;
    const out = [];
    while (i < n && j < m) {
      if (A[i] === B[j]) { out.push(['=', A[i]]); i++; j++; }
      else if (dp[i + 1][j] >= dp[i][j + 1]) { out.push(['-', A[i]]); i++; }
      else { out.push(['+', B[j]]); j++; }
    }
    while (i < n) { out.push(['-', A[i]]); i++; }
    while (j < m) { out.push(['+', B[j]]); j++; }
    return out;
  }
  function showDiff(n) {
    const idx = versions.findIndex(x => x.n === n);
    if (idx <= 0) { $('hMsg').textContent = 'diff needs an earlier version'; return; }
    const older = versions[idx - 1].text, newer = versions[idx].text;
    const parts = wordDiff(older, newer);
    const box = $('hView');
    box.innerHTML = '';
    const pre = document.createElement('pre');
    pre.className = 'h-diff';
    for (const [k, wd] of parts) {
      if (k === '=') { pre.appendChild(document.createTextNode(wd + ' ')); }
      else if (k === '+') { const s = document.createElement('span'); s.className = 'hd-add'; s.textContent = wd + ' '; pre.appendChild(s); }
      else { const s = document.createElement('span'); s.className = 'hd-del'; s.textContent = wd + ' '; pre.appendChild(s); }
    }
    box.appendChild(pre);
    const legend = document.createElement('div');
    legend.className = 'hint';
    legend.innerHTML = '<span style="background:#1d3a2d;color:var(--ok);padding:0 4px">added</span> <span style="background:#3a1d1d;color:var(--err);padding:0 4px">removed</span>  (#' + (idx) + ' → #' + (idx + 1) + ')';
    box.appendChild(legend);
    // mark a transient diff view so the auto-refresh doesn't clobber it
    diffView = { older: idx, newer: idx + 1 };
    selView = null;
  }

  // ---- wiring ----
  $('hSaveManual').onclick = () => saveVersion(false);
  $('hAutoN').onchange = () => { autoEvery = Math.max(1, Number($('hAutoN').value) || 100); lastAutoWords = 0; };
  $('hAutoN').value = autoEvery;

  // auto-snapshot: when writing.js autosaves (pulse), we also check word-ceiling.
  // hook into App.runDone (called after every /api/run) and the 3s tick.
  const origRunDone = window.App.onRunDone;
  window.App.onRunDone = function () { if (origRunDone) origRunDone(); roughly(); };
  function roughly() {
    // cheap: reload versions list (so counts/time stay fresh) + maybe auto
    if (curDoc() != null) { readFile(versFile(curDoc())).then(raw => { if (raw) { try { const a = JSON.parse(raw); if (Array.isArray(a)) versions = a; render(); } catch (e) {} } }); }
    maybeAuto();
  }
  setInterval(() => { if (curDoc() != null) loadVersions(); }, 4000);

  window.addEventListener('load', () => {
    const btn = document.querySelector('nav button[data-tab="writing"]');
    if (btn) btn.addEventListener('click', loadVersions);
  });
  loadVersions();
})();
