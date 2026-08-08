// Writing tab — W2: Library & Persistence.
// Sidebar tree: Story → {Chapters → Scenes}, Notes, Journal. Sidecar storage
// writing/<brain>/ (index.json + one JSON per doc) via serve file endpoints;
// every autosave also binds into the brain (doc replace --save) with a pulse.
// TWO NAMED CURSORS: user caret ("you") vs brain's named caret (<file>),
// synced from the state snapshot; stale decoration cleared on doc switch.
(function () {
  const { $, p, v, run, runJson, esc } = window.App;
  let editor = null, currentDoc = null, blocks = [], saveTimer = null;
  let focusOn = false, typewriterOn = false;
  let lib = { story: [], notes: [], journal: null, nextId: 1 };
  let open = [];                       // open doc ids, tab order
  const counts = {};                   // docId -> word count (tree badges)
  const titles = {};                   // docId -> title (from sidecar)

  // editor.js is an ES module — wait for window.Writer.
  function whenReady(fn, n) {
    if (window.Writer) fn();
    else if ((n || 0) < 200) setTimeout(() => whenReady(fn, (n || 0) + 1), 100);
  }

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }
  function docFile(id) { return sidecarDir() + '/doc-' + id + '.json'; }
  function mdMode() { return $('mdModeSel').value === 'md'; }
  function wordsOf(t) { const m = t.trim().match(/\S+/g); return m ? m.length : 0; }

  // ---------- sidecar I/O through the serve file endpoints ----------
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
  async function loadIndex() {
    const raw = await readFile(sidecarDir() + '/index.json');
    if (raw) {
      try { lib = JSON.parse(raw); } catch (e) { lib = { story: [], notes: [], journal: null, nextId: 1 }; }
    } else {
      await migrateIndex();
    }
    renderTree();
  }
  async function saveIndex() { return writeFile(sidecarDir() + '/index.json', JSON.stringify(lib, null, 1)); }
  // first run for this brain: turn whatever docs the brain already has into
  // a Chapter 1 of scenes so nothing is hidden.
  async function migrateIndex() {
    const docs = await runJson(['doc', 'list', p(), '--json']);
    lib = { story: [], notes: [], journal: null, nextId: 1 };
    if (docs && docs.length) {
      const ch = { id: 'c' + (lib.nextId++), title: 'Chapter 1', scenes: [] };
      for (const d of docs) {
        ch.scenes.push({ id: 's' + (lib.nextId++), title: d.title || ('Scene ' + d.id), docId: d.id });
        titles[d.id] = d.title || ('Scene ' + d.id);
      }
      lib.story.push(ch);
    }
    await saveIndex();
  }

  // ---------- tree rendering ----------
  function mk(liCls, text, onClick) {
    const li = document.createElement('li');
    li.className = liCls;
    const t = document.createElement('span');
    t.className = 'w2t';
    t.textContent = text;
    const w = document.createElement('span');
    w.className = 'w2w';
    const x = document.createElement('span');
    x.className = 'w2x';
    x.textContent = '✕';
    x.title = 'remove from library';
    li.append(t, w, x);
    t.addEventListener('click', onClick);
    li.addEventListener('dblclick', (e) => { if (e.target === t) startRename(li, t); });
    return { li, t, w, x };
  }

  function chapterOf(sceneId) { return lib.story.find(ch => ch.scenes.some(s => s.id === sceneId)); }

  function renderTree() {
    const st = $('wStory'), nt = $('wNotes'), jn = $('wJournal');
    st.innerHTML = ''; nt.innerHTML = ''; jn.innerHTML = '';
    for (const ch of lib.story) {
      const c = mk('chapter', ch.title, () => {});
      c.li.dataset.id = ch.id;
      c.li.draggable = true;
      bindDrag(c.li, 'chapter', ch.id);
      const add = document.createElement('span');
      add.className = 'w2a';
      add.textContent = '+';
      add.title = 'add scene';
      add.addEventListener('click', (e) => { e.stopPropagation(); addScene(ch.id); });
      c.li.insertBefore(add, c.x);
      c.x.addEventListener('click', (e) => { e.stopPropagation(); deleteChapter(ch.id); });
      st.appendChild(c.li);
      for (const sc of ch.scenes) {
        const s = mk('scene', sc.title, () => openDoc(sc.docId));
        s.w.textContent = counts[sc.docId] != null ? counts[sc.docId] + 'w' : '';
        s.li.dataset.id = sc.id; s.li.dataset.doc = sc.docId;
        s.li.draggable = true;
        bindDrag(s.li, 'scene', sc.id);
        if (currentDoc === sc.docId) s.li.classList.add('on');
        s.x.addEventListener('click', (e) => { e.stopPropagation(); deleteScene(ch.id, sc.id); });
        st.appendChild(s.li);
      }
    }
    for (const n of lib.notes) {
      const nn = mk('note', n.title, () => openDoc(n.docId));
      nn.w.textContent = counts[n.docId] != null ? counts[n.docId] + 'w' : '';
      nn.li.dataset.id = n.id; nn.li.dataset.doc = n.docId;
      nn.li.draggable = true;
      bindDrag(nn.li, 'note', n.id);
      if (currentDoc === n.docId) nn.li.classList.add('on');
      nn.x.addEventListener('click', (e) => { e.stopPropagation(); deleteNote(n.id); });
      nt.appendChild(nn.li);
    }
    if (lib.journal) {
      const j = mk('journal', lib.journal.title, () => openDoc(lib.journal.docId));
      j.w.textContent = counts[lib.journal.docId] != null ? counts[lib.journal.docId] + 'w' : '';
      j.li.dataset.doc = lib.journal.docId;
      if (currentDoc === lib.journal.docId) j.li.classList.add('on');
      j.x.addEventListener('click', (e) => { e.stopPropagation(); deleteJournal(); });
      jn.appendChild(j.li);
    }
    renderTabs();
  }

  function startRename(li, t) {
    const inp = document.createElement('input');
    inp.className = 'w2edit';
    inp.value = t.textContent;
    li.replaceChild(inp, t);
    inp.focus(); inp.select();
    const done = async () => {
      const val = inp.value.trim() || 'untitled';
      const id = li.dataset.id, doc = li.dataset.doc;
      if (id && id[0] === 'c') { const ch = lib.story.find(x => x.id === id); if (ch) ch.title = val; }
      else if (id && id[0] === 's') { const ch = chapterOf(id); const sc = ch && ch.scenes.find(x => x.id === id); if (sc) sc.title = val; }
      else if (id && id[0] === 'n') { const n = lib.notes.find(x => x.id === id); if (n) n.title = val; }
      else if (doc) { if (lib.journal && lib.journal.docId === Number(doc)) lib.journal.title = val; }
      if (doc) titles[Number(doc)] = val;
      await saveIndex();
      renderTree();
    };
    inp.addEventListener('keydown', (e) => { if (e.key === 'Enter') inp.blur(); if (e.key === 'Escape') { li.replaceChild(t, inp); } });
    inp.addEventListener('blur', done);
  }

  // ---------- drag reorder (HTML5 dnd) ----------
  let dragInfo = null;
  function bindDrag(li, kind, id) {
    li.addEventListener('dragstart', (e) => {
      dragInfo = { kind, id };
      li.classList.add('drag');
      if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
    });
    li.addEventListener('dragend', () => { dragInfo = null; document.querySelectorAll('.w2list li').forEach(x => x.classList.remove('drag', 'drop')); });
    li.addEventListener('dragover', (e) => {
      if (!dragInfo || dragInfo.id === id) return;
      e.preventDefault(); e.dataTransfer.dropEffect = 'move';
      document.querySelectorAll('.w2list li').forEach(x => x.classList.remove('drop'));
      li.classList.add('drop');
    });
    li.addEventListener('dragleave', () => li.classList.remove('drop'));
    li.addEventListener('drop', (e) => { e.preventDefault(); reorder(dragInfo, kind, id); });
  }

  async function reorder(src, dstKind, dstId) {
    const { kind, id } = src;
    if (kind === 'chapter' && dstKind === 'chapter' && id !== dstId) {
      const a = lib.story.findIndex(x => x.id === id), b = lib.story.findIndex(x => x.id === dstId);
      if (a >= 0 && b >= 0) { lib.story.splice(b, 0, lib.story.splice(a, 1)[0]); await saveIndex(); renderTree(); }
    } else if (kind === 'note' && dstKind === 'note' && id !== dstId) {
      const a = lib.notes.findIndex(x => x.id === id), b = lib.notes.findIndex(x => x.id === dstId);
      if (a >= 0 && b >= 0) { lib.notes.splice(b, 0, lib.notes.splice(a, 1)[0]); await saveIndex(); renderTree(); }
    } else if (kind === 'scene') {
      const srcCh = chapterOf(id), dstCh = dstKind === 'chapter' ? lib.story.find(x => x.id === dstId) : chapterOf(dstId);
      if (!srcCh || !dstCh) return;
      const sc = srcCh.scenes.find(x => x.id === id);
      if (!sc) return;
      srcCh.scenes = srcCh.scenes.filter(x => x.id !== id);
      if (dstKind === 'scene') {
        const b = dstCh.scenes.findIndex(x => x.id === dstId);
        dstCh.scenes.splice(b, 0, sc);
      } else dstCh.scenes.push(sc);
      await saveIndex(); renderTree();
    }
  }

  // ---------- library CRUD ----------
  async function newDoc(title) {
    const r = await fetch('/api/run?args=' + encodeURIComponent(['doc', 'new', p(), '--title', title, '--save'].join('\u001f'))).then(x => x.json()).catch(() => null);
    const m = r && /#(\d+)/.exec(r.stdout || '');
    if (m) window.App.refreshState();
    return m ? Number(m[1]) : null;
  }
  async function addChapter() {
    lib.story.push({ id: 'c' + (lib.nextId++), title: 'Chapter ' + (lib.story.length + 1), scenes: [] });
    await saveIndex(); renderTree();
  }
  async function addNote() {
    const title = 'Note ' + (lib.notes.length + 1);
    const docId = await newDoc(title);
    if (docId == null) return;
    lib.notes.push({ id: 'n' + (lib.nextId++), title, docId });
    titles[docId] = title;
    await saveIndex(); renderTree();
    openDoc(docId);
  }
  async function addScene(chId) {
    const ch = lib.story.find(x => x.id === chId);
    if (!ch) return;
    const title = 'Scene ' + (ch.scenes.length + 1);
    const docId = await newDoc(title);
    if (docId == null) return;
    ch.scenes.push({ id: 's' + (lib.nextId++), title, docId });
    titles[docId] = title;
    await saveIndex(); renderTree();
    openDoc(docId);
  }
  async function addJournal() {
    if (lib.journal) { openDoc(lib.journal.docId); return; }
    const title = 'Journal';
    const docId = await newDoc(title);
    if (docId == null) return;
    lib.journal = { title, docId };
    titles[docId] = title;
    await saveIndex(); renderTree();
    openDoc(docId);
  }
  async function deleteChapter(id) {
    const ch = lib.story.find(x => x.id === id);
    if (!ch || !confirm('Remove chapter "' + ch.title + '" from the library? (the brain keeps its memory of these docs)')) return;
    ch.scenes.forEach(s => closeDocTab(s.docId, true));
    lib.story = lib.story.filter(x => x.id !== id);
    await saveIndex(); renderTree();
  }
  async function deleteScene(chId, id) {
    const ch = lib.story.find(x => x.id === chId);
    const sc = ch && ch.scenes.find(x => x.id === id);
    if (!sc || !confirm('Remove scene "' + sc.title + '" from the library?')) return;
    closeDocTab(sc.docId, true);
    ch.scenes = ch.scenes.filter(x => x.id !== id);
    await saveIndex(); renderTree();
  }
  async function deleteNote(id) {
    const n = lib.notes.find(x => x.id === id);
    if (!n || !confirm('Remove note "' + n.title + '" from the library?')) return;
    closeDocTab(n.docId, true);
    lib.notes = lib.notes.filter(x => x.id !== id);
    await saveIndex(); renderTree();
  }
  async function deleteJournal() {
    if (!lib.journal || !confirm('Remove the journal from the library?')) return;
    closeDocTab(lib.journal.docId, true);
    lib.journal = null;
    await saveIndex(); renderTree();
  }

  // ---------- tabs ----------
  function renderTabs() {
    const bar = $('docTabs');
    bar.innerHTML = '';
    for (const id of open) {
      const t = document.createElement('span');
      t.className = 'dtab' + (id === currentDoc ? ' on' : '');
      const nm = document.createElement('span');
      nm.textContent = '#' + id + ' ' + (titles[id] || 'doc');
      const x = document.createElement('span');
      x.className = 'dtx'; x.textContent = '✕';
      x.addEventListener('click', (e) => { e.stopPropagation(); closeDocTab(id); });
      t.append(nm, x);
      t.addEventListener('click', () => { if (id !== currentDoc) switchTo(id); });
      bar.appendChild(t);
    }
  }
  function closeDocTab(id, silent) {
    if (currentDoc === id) {
      flushSave();
      currentDoc = null;
      if (window.App) window.App.currentDoc = null;
      const rest = open.filter(x => x !== id);
      open = rest;
      if (rest.length) switchTo(rest[rest.length - 1]);
      else { editor.setDoc('', mdMode()); editor.setBrainCursor(null); $('docPill').textContent = 'no document'; renderTabs(); }
    } else {
      open = open.filter(x => x !== id);
      renderTabs();
    }
    if (!silent) renderTree();
  }
  function switchTo(id) { openDoc(id, true); }

  // ---------- document load / save ----------
  async function openDoc(id, fromTab) {
    if (currentDoc != null && currentDoc !== id) flushSave();
    const data = await runJson(['doc', 'read', p(), '--doc', String(id), '--json']);
    if (!data) { $('docPill').textContent = 'doc #' + id + ' missing in brain'; return; }
    blocks = Array.isArray(data) ? data : (data.blocks || []);
    currentDoc = id;
    if (window.App) window.App.currentDoc = id;
    if (!open.includes(id)) open.push(id);
    const text = blocks.map(b => b.text).join('\n\n');
    // sidecar copy holds caret + title
    const sc = await readFile(docFile(id));
    let caret = null, title = null;
    if (sc) { try { const o = JSON.parse(sc); caret = o.caret; title = o.title; } catch (e) {} }
    if (title) titles[id] = title;
    if (editor) {
      editor.setBrainCursor(null);
      editor.setDoc(text, mdMode());
      editor.setPos(caret != null ? caret : (Number(localStorage.getItem('nf-caret-' + p() + '-' + id)) || 0));
    }
    counts[id] = wordsOf(text);
    $('docPill').textContent = 'doc #' + id + ' · ' + blocks.length + ' blocks';
    renderTree();
  }

  function scheduleSave() {
    if (!editor || currentDoc == null) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, 2000);
  }

  async function flushSave() {
    if (!editor || currentDoc == null) return;
    const id = currentDoc;
    const text = editor.view.state.doc.toString();
    clearTimeout(saveTimer);
    // 1) sidecar: one JSON per doc (library copy, served through file endpoints)
    const sc = { id, title: titles[id] || ('doc ' + id), mode: mdMode() ? 'md' : 'prose', text, caret: editor.getPos(), words: wordsOf(text), updated: new Date().toISOString() };
    await writeFile(docFile(id), JSON.stringify(sc, null, 1));
    // 2) bind into the brain (doc replace --save) — the memory side
    run(['doc', 'replace', p(), '--doc', String(id), '--text', text, '--save']);
    counts[id] = wordsOf(text);
    localStorage.setItem('nf-caret-' + p() + '-' + id, String(editor.getPos()));
    // only touch the pill while this doc is still the open one (closing the
    // last tab mid-save must not clobber the 'no document' state)
    if (currentDoc === id) {
      const pill = $('docPill');
      const base = 'doc #' + id + ' · ' + blocks.length + ' blocks';
      pill.textContent = base + ' · bound to memory…';
      setTimeout(() => { if (currentDoc === id) pill.textContent = base; }, 1800);
    }
    renderTree();
  }

  // ---------- editor extras ----------
  function appendReply(who, text) {
    const box = $('wResponse');
    const div = document.createElement('div');
    div.className = 'part';
    div.innerHTML = '<span class="who ' + (who === 'you' ? 'you' : 'brain') + '">' + esc(who) + '</span><div>' + esc(text) + '</div>';
    box.appendChild(div);
    box.scrollTop = box.scrollHeight;
  }
  // NovelAI-style: "continue from here" asks the brain (attached LLM) to continue.
  // The reply is shown above AND written back into the doc so the brain's own
  // cursor advances to where it actually wrote (writing-organ stub wiring).
  async function onContinue() {
    const id = currentDoc;
    if (id == null) { $('wBrainTxt2').textContent = 'open a doc first'; return; }
    const text = editor.view.state.doc.toString();
    appendReply('you', text.slice(-300));   // show the tail you handed off
    $('btnResp').disabled = true;
    try {
      const r = await fetch('/api/run?args=' + encodeURIComponent(['doc', 'assist', p(), '--doc', String(id), 'continue the scene from the current text', '--teacher', 'active'].join('\u001f'))).then(x => x.json());
      const raw = (r && r.ok) ? (r.stdout || '') : ((r && r.stderr) || 'no teacher attached — brain will reflect in its own words');
      const out = raw.replace(/^\s*\[[a-z]+\].*?\n/, '').trim() || raw.trim() || '' ;
      appendReply('brain', out || '(no reply)');
      if (out) {
        // bind the continuation into the doc (doc write --save) so the file's
        // cursor advances: the snapshot re-publishes cursorDoc/Block -> editor.
        await fetch('/api/run?args=' + encodeURIComponent(['doc', 'write', p(), '--doc', String(id), '--text', out, '--save'].join('\u001f'))).then(x => x.json()).catch(() => {});
        // refresh state so the brain cursor moves; also reload the editor text
        window.App.refreshState();
        openDoc(id, true);
      }
      $('wBrainTxt2').textContent = out ? 'brain continued — its cursor now sits at the new block' : 'no teacher attached; nothing written';
    } catch (e) {
      appendReply('brain', 'request failed: ' + e);
    }
    $('btnResp').disabled = false;
  }

  function onCount(s) {
    $('docCounts').textContent = s.words + ' words · ' + s.chars + ' chars · ' + s.lines + ' lines';
    $('docCaret').textContent = 'pos ' + s.pos;
    const goal = Number(v('goalN')) || 1;
    $('goalBar').style.width = Math.min(100, (s.words / goal) * 100) + '%';
  }

  // brain cursor: snapshot → this doc's last written block → editor position
  function onState(d) {
    if (!editor || currentDoc == null) return;
    const w = d.writing || {};
    if (w.cursorDoc === currentDoc && blocks.length) {
      const idx = Math.min(w.cursorBlock ?? 0, blocks.length - 1);
      let pos = 0;
      for (let i = 0; i <= idx; i++) pos += blocks[i].text.length + (i < idx ? 2 : 0);
      editor.setBrainCursor(pos, brainName());
      $('wBrainTxt2').textContent = "the file's cursor sits at block #" + idx + ': "' + (blocks[idx].text.slice(0, 40)) + '…”';
    } else if (w.cursorDoc != null) {
      editor.setBrainCursor(null);
      $('wBrainTxt2').textContent = "the file's cursor is in doc #" + w.cursorDoc + ' — open it to see it.';
    }
  }

  whenReady(() => {
    editor = window.Writer.createEditor($('editorHost'), {
      doc: '',
      onCount,
      onDocChange: scheduleSave,
    });
    loadIndex();
    // library actions
    $('btnAddChapter').onclick = addChapter;
    $('btnAddNote').onclick = addNote;
    $('btnAddJournal').onclick = addJournal;
    // modes & aids
    $('mdModeSel').onchange = () => { if (editor && currentDoc != null) { const t = editor.view.state.doc.toString(); editor.setDoc(t, mdMode()); } };
    $('btnFocus').onclick = () => { focusOn = !focusOn; editor.setFocusMode(focusOn); $('btnFocus').classList.toggle('pri', focusOn); };
    $('btnTypewriter').onclick = () => { typewriterOn = !typewriterOn; editor.setTypewriter(typewriterOn); $('btnTypewriter').classList.toggle('pri', typewriterOn); };
    $('btnResp').onclick = onContinue;
    // brain keyboard: the brain can ALWAYS write via its writing organ; this
    // button only toggles whether YOU peek at that channel (hidden from you by
    // default, never hidden from the brain).
    const bkArea = $('wBkArea');
    $('btnBrainKbd').onclick = () => {
      const on = bkArea.style.display !== 'none';
      bkArea.style.display = on ? 'none' : '';
      $('btnBrainKbd').classList.toggle('pri', !on);
      $('bkStatus').textContent = on ? 'peeked off — the brain still writes; only this view is hidden' : 'peeked on — see the brain key as it writes';
    };
    // drive one brain write through the organ path (same route the brain uses
    // on its own: doc write --save). For testing/peek; the brain's own writes
    // share this exact path automatically.
    $('btnBkSend').onclick = async () => {
      const id = currentDoc;
      const text = $('wBkInput').value.trim();
      if (id == null) { $('bkMsg').textContent = 'open a doc first'; return; }
      if (!text) { $('bkMsg').textContent = 'nothing to write yet'; return; }
      appendReply('brain', text);
      await fetch('/api/run?args=' + encodeURIComponent(['doc', 'write', p(), '--doc', String(id), '--text', text, '--save'].join('\u001f'))).then(x => x.json()).catch(() => {});
      $('wBkInput').value = '';
      $('bkMsg').textContent = 'walked one write through the organ → doc #' + id + ' (src=writing, no tokens)';
      window.App.refreshState();
      openDoc(id, true);
    };
    window.App.onState = onState;
    // brain-file switch: reload the library when the path changes
    let lastBrain = p();
    setInterval(async () => {
      if (p() !== lastBrain) {
        lastBrain = p();
        open = []; currentDoc = null;
        editor.setDoc('', mdMode()); editor.setBrainCursor(null);
        $('docPill').textContent = '—';
        await loadIndex();
      }
    }, 3000);
  });
})();
