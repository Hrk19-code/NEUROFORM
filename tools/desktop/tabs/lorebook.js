// Writing tab — W3: Lorebooks & Entity Sheets.
// Lorebook entries {title, keywords[], body, enabled, tags[]}; entity sheets
// {name, kind, fields{key:val}, rels:[{to, tag?}]}. Persisted per brain in
// sidecar writing/<brain>/lorebook.json + entities.json via serve file
// endpoints, exactly like W2's doc sidecar — the brain's memory is NOT touched
// by the sheet itself (the visible entity lives here; the brain's own
// extraction pipeline in the engine keeps its separate entity records).
(function () {
  const { $, p, esc } = window.App;
  let lore = [];       // {id,title,keywords,body,enabled,tags}
  let entities = [];   // {id,name,kind,fields,rels}
  let nextId = 1;

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }

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

  async function loadAll() {
    const both = await Promise.all([
      readFile(sidecarDir() + '/lorebook.json'),
      readFile(sidecarDir() + '/entities.json'),
    ]);
    lore = []; entities = []; nextId = 1;
    let hadLore = false, hadEnt = false;
    try { const a = JSON.parse(both[0]); if (Array.isArray(a)) { lore = a; hadLore = true; a.forEach(e => nextId = Math.max(nextId, (e.id || 0) + 1)); } } catch (e) {}
    try { const b = JSON.parse(both[1]); if (Array.isArray(b)) { entities = b; hadEnt = true; b.forEach(e => nextId = Math.max(nextId, (e.id || 0) + 1)); } } catch (e) {}
    if (!hadLore && !entities.length && !hadEnt) { seed(); await saveBoth(); }
    else {
      // seed each missing collection independently, then persist so the seed
      // survives a reload (previously seed-only data vanished on refresh).
      let dirty = false;
      if (!hadLore) { seedLoreOnly(); dirty = true; }
      if (!hadEnt) { seedEntitiesOnly(); dirty = true; }
      if (dirty) await saveBoth();
    }
    renderAll();
  }
  function seed() { seedLoreOnly(); seedEntitiesOnly(); }
  function seedLoreOnly() {
    lore.push(
      { id: nextId++, title: 'The Old Bridge', keywords: ['bridge', 'spans'], body: 'A stone span over the river, remembered in every scene.', enabled: true, tags: ['location'] },
      { id: nextId++, title: 'The Valley', keywords: ['valley', 'river'], body: 'A quiet place where the river carries the light.', enabled: true, tags: ['location'] },
      { id: nextId++, title: 'Empyrean Oath', keywords: ['oath', 'swear', 'pledge'], body: 'A binding promise the characters keep at great cost.', enabled: true, tags: ['lore', 'rule'] },
      { id: nextId++, title: 'The Lantern Keeper', keywords: ['lantern', 'keeper'], body: 'Tends the light at the river crossing; asked for nothing, gave everything.', enabled: true, tags: ['character'] },
      { id: nextId++, title: 'Wintering', keywords: ['winter', 'frost', 'snow'], body: 'The long cold that shapes how everyone in the valley speaks.', enabled: true, tags: ['season'] }
    );
  }
  function seedEntitiesOnly() {
    entities.push(
      { id: nextId++, name: 'Elena', kind: 'character', fields: { hair: 'auburn', origin: 'the valley' }, rels: [] },
      { id: nextId++, name: 'Keeper', kind: 'character', fields: { role: 'tends the lantern', secret: 'keeps the oath' }, rels: [{ to: 'Elena', tag: 'guardian' }] },
      { id: nextId++, name: 'The Bridge', kind: 'place', fields: { material: 'stone', crossing: 'river' }, rels: [] }
    );
  }
  async function saveBoth() { await saveLore(); await saveEntities(); }

  // ---------- rendering ----------
  function renderLore() {
    const box = $('lbList');
    box.innerHTML = '';
    $('lbCount').textContent = lore.length + ' entries';
    for (const e of lore) {
      const li = document.createElement('li');
      li.className = 'note';
      li.style.cssText = 'padding:4px 6px; border-bottom:1px dashed #1a2530; display:flex; gap:8px; align-items:center;';
      const tg = document.createElement('span');
      tg.textContent = '◐';
      tg.style.cssText = 'cursor:pointer;color:' + (e.enabled ? 'var(--ok)' : 'var(--dim)');
      tg.title = e.enabled ? 'enabled — fires on keywords' : 'disabled — will not fire';
      tg.onclick = () => { e.enabled = !e.enabled; saveLore(); };
      const nm = document.createElement('span');
      nm.textContent = e.title;
      nm.style.cssText = 'flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;cursor:pointer';
      nm.title = 'keywords: ' + (e.keywords || []).join(', ') + ' · ' + (e.body || '');
      // click cycles: show keywords → show body → collapse
      let step = 0;
      nm.onclick = () => { step = (step + 1) % 3;
        nm.textContent = step === 1 ? '↪ ' + (e.keywords || []).join(', ')
          : step === 2 ? '“' + (e.body || '') + '”' : e.title;
      };
      const dx = document.createElement('span');
      dx.textContent = '✕';
      dx.className = 'w2x';
      dx.style.visibility = 'visible';
      dx.onclick = () => { lore = lore.filter(x => x.id !== e.id); saveLore(); };
      li.append(tg, nm, dx);
      box.appendChild(li);
    }
  }
  function renderEnt() {
    const box = $('enList');
    box.innerHTML = '';
    $('enCount').textContent = entities.length + ' entities';
    for (const e of entities) {
      const li = document.createElement('li');
      li.className = 'note';
      li.style.cssText = 'padding:4px 6px; border-bottom:1px dashed #1a2530; display:flex; gap:8px; align-items:center;';
      const nm = document.createElement('span');
      nm.textContent = e.name + ' · ' + e.kind;
      nm.style.cssText = 'flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;cursor:pointer';
      const f = Object.entries(e.fields || {}).map(([k, v]) => k + ': ' + v).join('; ');
      const rs = (e.rels || []).map(r => '→ ' + r.to + (r.tag ? ' (' + r.tag + ')' : '')).join(', ');
      nm.title = f + (rs ? '\n' + rs : '');
      let step = 0;
      nm.onclick = () => { step = (step + 1) % 3;
        nm.textContent = step === 1 ? (f || '(no fields)') : step === 2 ? (rs || '(no relationships)') : e.name + ' · ' + e.kind;
      };
      const dx = document.createElement('span');
      dx.textContent = '✕';
      dx.className = 'w2x';
      dx.style.visibility = 'visible';
      dx.onclick = () => { entities = entities.filter(x => x.id !== e.id); saveEntities(); };
      li.append(nm, dx);
      box.appendChild(li);
    }
  }
  function renderAll() { renderLore(); renderEnt(); }

  // ---------- insertion preview ----------
  // Which enabled lorebook entries fire on the probe text? Exact keyword match,
  // case-insensitive, whole-word. "fire" = at least one keyword appears.
  function preview(text) {
    const t = (text || '').toLowerCase();
    return lore.filter(e => {
      if (!e.enabled) return false;
      const kws = (e.keywords || []).map(k => String(k).toLowerCase()).filter(Boolean);
      return kws.some(k => t.includes(k));
    });
  }
  function renderPreview() {
    const hit = preview($('lbProbe').value);
    $('lbPrevR').innerHTML = hit.length
      ? 'firing: ' + hit.map(h => esc(h.title)).join(' · ')
      : 'no enabled entry fires on that text';
    $('lbPrevR').style.color = hit.length ? 'var(--ok)' : 'var(--dim)';
  }
  $('btnLbPrev').onclick = renderPreview;
  $('lbProbe').addEventListener('input', () => { /* live on demand: keep click authoritative */ });

  // ---------- add handlers ----------
  $('btnLbAdd').onclick = () => {
    const title = ($('lbTitle').value || '').trim() || 'entry ' + (lore.length + 1);
    const keywords = $('lbKeywords').value.split(',').map(s => s.trim()).filter(Boolean);
    const body = $('lbBody').value.trim();
    const tags = $('lbTags').value.split(',').map(s => s.trim()).filter(Boolean);
    lore.push({ id: nextId++, title, keywords, body, enabled: true, tags });
    saveLore();
    $('lbTitle').value = ''; $('lbKeywords').value = ''; $('lbBody').value = ''; $('lbTags').value = '';
  };
  $('btnEnAdd').onclick = () => {
    const name = ($('enName').value || '').trim() || 'entity ' + (entities.length + 1);
    const kind = ($('enKind').value || '').trim() || 'entity';
    const fields = {};
    $('enFields').value.split(';').forEach(pr => {
      const i = pr.indexOf(':');
      if (i > 0) fields[pr.slice(0, i).trim()] = pr.slice(i + 1).trim();
    });
    const relTo = $('enRelTo').value.trim();
    const rels = relTo ? [{ to: relTo, tag: 'relation' }] : [];
    entities.push({ id: nextId++, name, kind, fields, rels });
    saveEntities();
    $('enName').value = ''; $('enKind').value = 'character'; $('enFields').value = ''; $('enRelTo').value = '';
  };

  // auto-run when the tab is clicked; reload library on brain switch like W2
  let lastBrain = p();
  function tic() { if (p() !== lastBrain) { lastBrain = p(); loadAll(); } }
  setInterval(tic, 3000);
  window.addEventListener('load', () => {
    const btn = document.querySelector('nav button[data-tab="writing"]');
    if (btn) btn.addEventListener('click', loadAll);
  });

  loadAll();
})();
