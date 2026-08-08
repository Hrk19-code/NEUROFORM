// Writing tab — W4: Structure Tools (Outline beat sheet + Timeline).
// Outline: one card per scene — synopsis + word count + status
// (draft/revised/final); drag a card to reorder within/across chapters, which
// mutates the same library index.json that W2's sidebar tree reads (so the
// tree updates too). Timeline: scenes on a horizontal axis by in-story date
// (fallback: chapter order), zoomable. Persisted in writing/<brain>/index.json
// via serve file endpoints, exactly like W2/W3.
(function () {
  const { $, p, esc } = window.App;
  let lib = { story: [], notes: [], journal: null, nextId: 1 };
  let mode = 'outline';        // 'outline' | 'timeline'
  let zoom = 1;

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }
  function indexFile() { return sidecarDir() + '/index.json'; }

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
    const raw = await readFile(indexFile());
    if (raw) {
      try {
        const o = JSON.parse(raw);
        lib = { story: [], notes: [], journal: null, nextId: 1, ...o };
      } catch (e) { lib = { story: [], notes: [], journal: null, nextId: 1 }; }
    }
    render();
  }
  async function saveIndex() {
    await writeFile(indexFile(), JSON.stringify(lib, null, 1));
    render();
  }

  // word counts: read from each scene's doc via doc read (cheap enough, cached)
  const wcCache = {};          // docId -> word count
  async function wordsFor(docId) {
    if (wcCache[docId] != null) return wcCache[docId];
    try {
      const r = await fetch('/api/run?args=' + encodeURIComponent(['doc', 'read', p(), '--doc', String(docId), '--json'].join('\u001f'))).then(x => x.json());
      const blocks = (r && r.ok) ? (Array.isArray(r.stdout) ? r.stdout : []) : [];
      const txt = blocks.map(b => b.text || '').join(' ');
      const m = txt.trim().match(/\S+/g);
      wcCache[docId] = m ? m.length : 0;
      return wcCache[docId];
    } catch (e) { return 0; }
  }

  // normalize a scene: give it synopsis/status/date defaults if absent (old data compat)
  function norm(scene) {
    return {
      ...scene,
      synopsis: scene.synopsis || '',
      status: scene.status || 'draft',          // draft | revised | final
      date: scene.date != null ? scene.date : null,  // in-story date/order, optional
    };
  }

  // ---------- outline (beat sheet) ----------
  function renderOutline() {
    const host = $('stOutline');
    host.innerHTML = '';
    for (const ch of lib.story) {
      const chEl = document.createElement('div');
      chEl.className = 'st-chapter';
      const h = document.createElement('div');
      h.className = 'st-chapter-head';
      h.textContent = '▸ ' + ch.title + '  (' + ch.scenes.length + ' scenes)';
      chEl.appendChild(h);
      const list = document.createElement('div');
      list.className = 'st-cards';
      list.dataset.chapter = ch.id;
      for (const scRaw of ch.scenes) {
        const sc = norm(scRaw);
        const card = document.createElement('div');
        card.className = 'st-card' + (sc.status === 'final' ? ' final' : sc.status === 'revised' ? ' revised' : '');
        card.dataset.scene = sc.id;
        card.draggable = true;
        const top = document.createElement('div');
        top.className = 'st-card-top';
        const title = document.createElement('span');
        title.className = 'st-card-title';
        title.textContent = esc(sc.title || 'Scene');
        const st = document.createElement('span');
        st.className = 'st-card-status';
        st.textContent = sc.status;
        top.append(title, st);
        const syn = document.createElement('div');
        syn.className = 'st-card-syn';
        syn.textContent = 'synopsis: ' + (sc.synopsis || '—');
        const meta = document.createElement('div');
        meta.className = 'st-card-meta';
        wordsFor(sc.docId).then(w => { meta.textContent = w + ' words' + (sc.date != null ? ' · in-story ' + sc.date : ''); });
        card.append(top, syn, meta);
        // drag reorder
        card.addEventListener('dragstart', (e) => {
          dragInfo = { sceneId: sc.id, chapterId: ch.id };
          card.classList.add('drag');
          if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
        });
        card.addEventListener('dragend', () => { dragInfo = null; document.querySelectorAll('.st-card').forEach(c => c.classList.remove('drag', 'drop')); });
        card.addEventListener('dragover', (e) => {
          if (!dragInfo || dragInfo.sceneId === sc.id) return;
          e.preventDefault();
          if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
          document.querySelectorAll('.st-card').forEach(c => c.classList.remove('drop'));
          card.classList.add('drop');
        });
        card.addEventListener('drop', (e) => { e.preventDefault(); doReorder(dragInfo, ch.id, sc.id); });
        // click to open the doc in the editor if a doc is attached
        card.addEventListener('dblclick', () => { if (sc.docId != null) window.App && window.App.openDoc && window.App.openDoc(sc.docId); });
        // edit synopsis/status/date inline
        card.addEventListener('dblclick', (ev) => {
          ev.stopPropagation();
          const panel = document.createElement('div');
          panel.className = 'st-edit';
          panel.innerHTML =
            '<div class="row"><label>synopsis</label><input type="text" class="st-syn" value="' + esc(sc.synopsis || '') + '"></div>' +
            '<div class="row"><label>status</label><select class="st-status">' +
            '<option value="draft"' + (sc.status === 'draft' ? ' selected' : '') + '>draft</option>' +
            '<option value="revised"' + (sc.status === 'revised' ? ' selected' : '') + '>revised</option>' +
            '<option value="final"' + (sc.status === 'final' ? ' selected' : '') + '>final</option></select>' +
            '<label>in-story date</label><input type="text" class="st-date" value="' + esc(sc.date != null ? sc.date : '') + '" placeholder="optional"></div>' +
            '<div class="row"><button class="btn pri st-ok">save</button><button class="btn st-cancel">cancel</button></div>';
          card.replaceWith(panel);
          const apply = () => {
            const syn = panel.querySelector('.st-syn').value.trim();
            const status = panel.querySelector('.st-status').value;
            const dateRaw = panel.querySelector('.st-date').value.trim();
            const date = dateRaw === '' ? null : dateRaw;
            // find the scene (by id) in the library and mutate + persist
            for (const c of lib.story) {
              const s = c.scenes.find(x => x.id === sc.id);
              if (s) { s.synopsis = syn; s.status = status; s.date = date; }
            }
            saveIndex();
          };
          panel.querySelector('.st-ok').onclick = apply;
          panel.querySelector('.st-cancel').onclick = () => { saveIndex(); };
          // Enter in the synopsis field saves
          panel.querySelector('.st-syn').addEventListener('keydown', (e) => { if (e.key === 'Enter') apply(); });
        });
        list.appendChild(card);
      }
      chEl.appendChild(list);
      host.appendChild(chEl);
    }
  }
  let dragInfo = null;
  function doReorder(src, dstChapterId, dstSceneId) {
    if (!src) return;
    const srcCh = lib.story.find(c => c.id === src.chapterId);
    const sc = srcCh && srcCh.scenes.find(s => s.id === src.sceneId);
    if (!sc) return;
    srcCh.scenes = srcCh.scenes.filter(s => s.id !== src.sceneId);
    const dstCh = lib.story.find(c => c.id === dstChapterId);
    const b = dstCh.scenes.findIndex(s => s.id === dstSceneId);
    dstCh.scenes.splice(b >= 0 ? b : dstCh.scenes.length, 0, sc);
    saveIndex();
  }

  // ---------- timeline ----------
  function renderTimeline() {
    const host = $('stTimeline');
    host.innerHTML = '';
    // collect all scenes with an ordering key: date if present, else chapter order index
    const items = [];
    lib.story.forEach((ch, ci) => {
      ch.scenes.forEach(scRaw => {
        const sc = norm(scRaw);
        items.push({ scene: sc, chapterTitle: ch.title, order: sc.date != null ? String(sc.date) : ci + '.' + (scRaw.id || 0) });
      });
    });
    // sort: dated scenes by their string date (simple), undated after, stable by chapter order
    const dated = items.filter(i => i.scene.date != null);
    const undated = items.filter(i => i.scene.date == null);
    dated.sort((a, b) => String(a.scene.date).localeCompare(String(b.scene.date), undefined, { numeric: true }));
    const sorted = dated.concat(undated);
    const track = document.createElement('div');
    track.className = 'st-track';
    // zoom: each scene block width = base * zoom
    const baseW = 150;
    for (const it of sorted) {
      const blk = document.createElement('div');
      blk.className = 'st-block';
      blk.style.width = (baseW * zoom) + 'px';
      const t = document.createElement('div');
      t.className = 'st-block-title';
      t.textContent = esc(it.scene.title || 'Scene');
      const d = document.createElement('div');
      d.className = 'st-block-date';
      d.textContent = it.scene.date != null ? ('◷ ' + esc(it.scene.date)) : ('(' + it.chapterTitle + ')');
      blk.append(t, d);
      blk.title = (it.scene.synopsis || 'no synopsis') + ' · ' + it.chapterTitle;
      track.appendChild(blk);
    }
    host.appendChild(track);
    const hint = document.createElement('div');
    hint.className = 'st-hint';
    hint.textContent = sorted.length + ' scenes — dated first by in-story date, then chapter order (zoom: ' + zoom + '×)';
    host.appendChild(hint);
  }

  function render() {
    if (mode === 'outline') { $('stOutline').style.display = ''; $('stTimeline').style.display = 'none'; renderOutline(); }
    else { $('stOutline').style.display = 'none'; $('stTimeline').style.display = ''; renderTimeline(); }
  }

  // ---------- wiring ----------
  $('stModeOutline').onclick = () => { mode = 'outline'; $('stModeTimeline').classList.remove('pri'); $('stModeOutline').classList.add('pri'); render(); };
  $('stModeTimeline').onclick = () => { mode = 'timeline'; $('stModeOutline').classList.remove('pri'); $('stModeTimeline').classList.add('pri'); render(); };
  $('stZoomIn').onclick = () => { zoom = Math.min(4, zoom + 0.5); renderTimeline(); };
  $('stZoomOut').onclick = () => { zoom = Math.max(0.5, zoom - 0.5); renderTimeline(); };

  // reload when the writing tab is opened (so it picks up W2/W3 edits to index.json)
  window.addEventListener('load', () => {
    const btn = document.querySelector('nav button[data-tab="writing"]');
    if (btn) btn.addEventListener('click', loadIndex);
  });
  loadIndex();
})();
