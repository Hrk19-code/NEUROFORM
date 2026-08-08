// Writing tab — W8: Brain Integration Panel.
// "What the brain remembers" for the open doc, from the REAL engine:
//   live retrieval (cue = current doc text) -> ep traces + sem nodes,
//   the doc's style fingerprint vs the prior docs (radar),
//   a writing-mode selector (journal/prose/worldbuilding/lorebook) reflecting
//   how the doc binds (the engine's DocMode gates feature extraction on write).
// Acceptance: panel shows live retrieval for open doc; modes differ in binding.
(function () {
  const { $, p, esc } = window.App;

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }
  function curDoc() { return (window.App && window.App.currentDoc) || null; }

  async function readFile(rel) {
    try { const r = await fetch('/api/file/read?path=' + encodeURIComponent(rel)).then(x => x.json()); return (r && r.ok) ? r.content : null; } catch (e) { return null; }
  }
  async function runCli(argv) {
    try { const r = await fetch('/api/run?args=' + encodeURIComponent(argv.join('\u001f'))).then(x => x.json()); return r && r.ok ? (r.stdout || '') : (r.stderr || ''); } catch (e) { return ''; }
  }
  async function docText(id) {
    const sc = await readFile(sidecarDir() + '/doc-' + id + '.json');
    if (!sc) return ''; try { return JSON.parse(sc).text || ''; } catch (e) { return ''; }
  }

  // ---- live retrieval: parse engine retrieve output ----
  function parseRetrieve(out) {
    const traces = [], nodes = [];
    const ort = /^\s*ep #(\d+)\s+score=([\d.]+)\s+sal=([\d.]+)\s+str=([\d.]+)\s+src=(\S+)\s*\[(.*)\]\s*$/gm;
    let m; while ((m = ort.exec(out)) !== null) traces.push({ id: +m[1], score: +m[2], sal: +m[3], str: +m[4], src: m[5], kw: m[6] });
    const ors = /^\s*sem #(\d+)\s+score=([\d.]+)\s+belief=([\d.]+)\s*\[(.*)\]\s*$/gm;
    while ((m = ors.exec(out)) !== null) nodes.push({ id: +m[1], score: +m[2], belief: +m[3], label: m[4] });
    return { traces, nodes };
  }
  // ---- parse doc style (engine) for the radar ----
  function parseStyle(out) {
    // "  sentence len: mean .. std .. | density .. | clauses .. | dialogue .."
    const ss = /sentence len: mean ([.\d]+) std ([.\d]+) \| density ([.\d]+) \| clauses ([.\d]+) \| dialogue ([.\d]+)/.exec(out);
    const sent = /sentiment: mean ([+\-.\d]+)/.exec(out);
    return ss ? {
      sentLen: +ss[1], density: +ss[3], dialogue: +ss[5],
      sentiment: sent ? +sent[1] : null,
    } : null;
  }
  // radar axes normalized 0..1 (scaled for display; labeled honestly as relative)
  function radarValues(st) {
    if (!st) return null;
    return [
      { label: 'sentence len', v: clamp(st.sentLen / 30, 0, 1) },
      { label: 'lexical density', v: clamp(st.density, 0, 1) },
      { label: 'dialogue', v: clamp(st.dialogue * 3, 0, 1) },
      { label: 'sentiment', v: clamp((st.sentiment + 1) / 2, 0, 1) },
    ];
  }
  function clamp(x, a, b) { return Math.max(a, Math.min(b, x)); }

  // ---- mode: reflect the doc's mode (engine DocMode) ----
  async function loadMode(id) {
    // the open doc's sidecar may carry mode; else default prose
    const sc = await readFile(sidecarDir() + '/doc-' + id + '.json');
    let mode = 'prose';
    if (sc) { try { const o = JSON.parse(sc); if (o.mode) mode = o.mode; } catch (e) {} }
    const sel = $('bpMode');
    sel.value = mode;
    $('bpModeHint').textContent = 'mode ' + mode + ' — gates how writing binds (engine DocMode)';
  }

  // ---- render ----
  function renderRetrieval(pr) {
    const box = $('bpRetrieve'); box.innerHTML = '';
    if (!pr.traces.length && !pr.nodes.length) { box.innerHTML = '<div class="hint">no retrieved memory for this doc yet — write more or save</div>'; return; }
    if (pr.traces.length) {
      const h = document.createElement('div'); h.className = 'bp-h'; h.textContent = 'retrieved traces (' + pr.traces.length + ')';
      box.appendChild(h);
      for (const t of pr.traces.slice(0, 6)) {
        const d = document.createElement('div'); d.className = 'bp-trace';
        d.innerHTML = '<span class="bp-src">' + esc(t.src) + '</span> <b>#' + t.id + '</b> score ' + t.score.toFixed(2) +
          ' <span class="dim">sal ' + t.sal.toFixed(2) + '</span> <span class="kw">' + esc(t.kw) + '</span>';
        box.appendChild(d);
      }
    }
    if (pr.nodes.length) {
      const h = document.createElement('div'); h.className = 'bp-h'; h.textContent = 'semantic nodes (' + pr.nodes.length + ')';
      box.appendChild(h);
      for (const n of pr.nodes.slice(0, 6)) {
        const d = document.createElement('div'); d.className = 'bp-node';
        d.textContent = '#' + n.id + ' · ' + esc(n.label) + ' (belief ' + n.belief.toFixed(2) + ')';
        box.appendChild(d);
      }
    }
  }
  function renderRadar(stSelf, stPrior) {
    const box = $('bpRadar'); box.innerHTML = '';
    const self = radarValues(stSelf), prior = radarValues(stPrior);
    if (!self && !prior) { box.innerHTML = '<div class="hint">style fingerprint — run analyze or open a doc with text</div>'; return; }
    // a 4-axis "radar" drawn as 4 labeled bars (open doc vs prior average)
    const axes = self || prior;
    for (const ax of axes) {
      const row = document.createElement('div'); row.className = 'bp-axis';
      const lab = document.createElement('span'); lab.className = 'bp-axis-label'; lab.textContent = ax.label;
      const bars = document.createElement('span'); bars.className = 'bp-axis-bars';
      const a = document.createElement('span'); a.className = 'bp-bar self'; a.style.width = ((self ? self.find(x => x.label === ax.label).v : 0) * 70) + 'px';
      const b = document.createElement('span'); b.className = 'bp-bar prior'; b.style.width = ((prior ? prior.find(x => x.label === ax.label).v : 0) * 70) + 'px';
      bars.append(a, b);
      row.append(lab, bars);
      box.appendChild(row);
    }
    const l = document.createElement('div'); l.className = 'bp-legend';
    l.innerHTML = '<span class="sw self"></span> this doc <span class="sw prior"></span> prior docs';
    box.appendChild(l);
  }
  async function refresh() {
    const id = curDoc();
    $('bpDoc').textContent = id != null ? ('doc #' + id) : '— no open doc —';
    if (id == null) { renderRetrieval({ traces: [], nodes: [] }); renderRadar(null, null); return; }
    const text = await docText(id);
    if (!text) {
      const empty = document.getElementById('bpRetrieve'); if (empty) empty.innerHTML = '<div class="hint">open a doc with text to see what the brain retrieves</div>';
      renderRadar(null, null);
      return;
    }
    // live retrieval cued by the doc's own text
    const out = await runCli(['retrieve', p(), '--query', text, '--k', '6']);
    renderRetrieval(parseRetrieve(out));
    // style fingerprint: this doc vs a prior doc
    const stylSelf = parseStyle(await runCli(['doc', 'style', p(), '--doc', String(id)]));
    // a prior doc (first scene of chapter 1 if it differs) for comparison
    const libRaw = await readFile(sidecarDir() + '/index.json');
    let priorId = null;
    if (libRaw) { try { const lib = JSON.parse(libRaw); const scene = lib.story && lib.story[0] && lib.story[0].scenes && lib.story[0].scenes.find(s => s.docId !== id); if (scene) priorId = scene.docId; } catch (e) {} }
    let stPrior = null;
    if (priorId != null) stPrior = parseStyle(await runCli(['doc', 'style', p(), '--doc', String(priorId)]));
    renderRadar(stylSelf, stPrior);
    loadMode(id);
  }

  // ---- wiring ----
  $('btnBpRefresh').onclick = refresh;
  $('bpMode').addEventListener('change', async () => {
    const id = curDoc(); if (id == null) return;
    const mode = $('bpMode').value;
    // persist the chosen mode on the doc's sidecar (informational; engine gates real binding)
    const sc = await readFile(sidecarDir() + '/doc-' + id + '.json');
    if (sc) { try { const o = JSON.parse(sc); o.mode = mode; await fetch('/api/file/write?path=' + encodeURIComponent(sidecarDir() + '/doc-' + id + '.json') + '&content=' + encodeURIComponent(JSON.stringify(o, null, 1))).then(r => r.json()); } catch (e) {} }
    $('bpModeHint').textContent = 'mode ' + mode + ' — set on doc #' + id;
    // note: different modes bind differently in the engine (DocMode on write); test by
    // writing a block to a doc created in that mode.
  });
  window.addEventListener('load', () => {
    const btn = document.querySelector('nav button[data-tab="writing"]');
    if (btn) btn.addEventListener('click', refresh);
  });
  refresh();
})();
