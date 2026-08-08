// Writing tab — W6: Analysis & Continuity.
// Style analysis (computed locally from the open doc's text): sentence-length
// histogram, repeated-word cloud, adverb density, dialogue ratio, readability
// score. Continuity: surfaces the ENGINE's real conflict flags (doc ledger) —
// property/timeline conflicts parsed from the CLI output, plus entity list.
// Acceptance: write a deliberately repetitive passage -> analysis flags it;
// contradict an entity property -> warning appears.
(function () {
  const { $, p, esc } = window.App;

  function brainName() { return (p().split(/[\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function sidecarDir() { return 'writing/' + brainName(); }
  function curDoc() { return (window.App && window.App.currentDoc) || null; }

  async function readFile(rel) {
    try {
      const r = await fetch('/api/file/read?path=' + encodeURIComponent(rel)).then(x => x.json());
      return (r && r.ok) ? r.content : null;
    } catch (e) { return null; }
  }
  async function runCli(argv) {
    try {
      const r = await fetch('/api/run?args=' + encodeURIComponent(argv.join('\u001f'))).then(x => x.json());
      return r && r.ok ? (r.stdout || '') : (r.stderr || '');
    } catch (e) { return ''; }
  }

  // ---------- style analysis (local, deterministic, testable) ----------
  function sentences(text) {
    const t = (text || '').trim();
    if (!t) return [];
    // split on sentence-ending punctuation, keep fragments as short sentences
    return t.split(/(?<=[.!?])\s+/).map(s => s.trim()).filter(Boolean);
  }
  function wordsOf(text) { const m = (text || '').trim().match(/\S+/g); return m ? m.length : 0; }
  function adverbDensity(text) {
    // common -ly adverbs (a simple heuristic; not exhaustive)
    const ad = (text || '').match(/\b\w+ly\b/g) || [];
    const w = wordsOf(text) || 1;
    return ad.length / w;
  }
  function dialogueRatio(text) {
    // dialogue = text inside quotes, ratio of dialogue words to total
    const dq = (text || '').match(/"([^"]*)"/g) || [];
    const dqWords = dq.map(q => wordsOf(q)).reduce((a, b) => a + b, 0);
    const w = wordsOf(text) || 1;
    return dqWords / w;
  }
  function sentenceLenHistogram(text) {
    const ss = sentences(text).map(wordsOf).filter(n => n > 0);
    const hist = {};            // bucket "≤3 words" etc -> count
    for (const n of ss) {
      const key = n <= 3 ? '≤3' : n <= 6 ? '4–6' : n <= 10 ? '7–10' : n <= 15 ? '11–15' : '16+';
      hist[key] = (hist[key] || 0) + 1;
    }
    const avg = ss.length ? ss.reduce((a, b) => a + b, 0) / ss.length : 0;
    return { hist, avg, count: ss.length };
  }
  function wordFreq(text) {
    const freq = {};
    (text || '').toLowerCase().split(/[^a-z']+/).filter(w => w.length > 2).forEach(w => { freq[w] = (freq[w] || 0) + 1; });
    return freq;
  }
  function repeatedWords(text, min = 3) {
    const f = wordFreq(text);
    return Object.entries(f).filter(([, c]) => c >= min).sort((a, b) => b[1] - a[1]).slice(0, 12);
  }
  function readability(text) {
    // simplified FRE formula on sentences + syllables
    const ss = sentences(text); const nSent = Math.max(1, ss.length);
    const nWord = Math.max(1, wordsOf(text));
    const syll = (text || '').match(/[aeiouy]+/gi) ? (text || '').match(/[aeiouy]+/gi).length : 0;
    const score = 206.835 - 1.015 * (nWord / nSent) - 84.6 * (syll / nWord);
    return Math.max(0, Math.min(100, Math.round(score)));
  }
  function analyze(text) {
    const sh = sentenceLenHistogram(text);
    return {
      sentences: sh.count,
      avgSentence: Math.round(sh.avg * 10) / 10,
      sentHist: sh.hist,
      words: wordsOf(text),
      adverbDensity: Math.round(adverbDensity(text) * 1000) / 1000,
      dialogueRatio: Math.round(dialogueRatio(text) * 1000) / 1000,
      repeated: repeatedWords(text),
      readability: readability(text),
    };
  }
  function renderStyle(a) {
    const box = $('anStyle');
    box.innerHTML = '';
    if (!a.words) { box.innerHTML = '<div class="hint">open a doc with text to analyze</div>'; return; }
    // sentence histogram bars
    const hist = a.sentHist;
    const hmax = Math.max(1, ...Object.values(hist));
    const ev = document.createElement('div');
    ev.className = 'an-hist';
    const labels = ['≤3', '4–6', '7–10', '11–15', '16+'];
    labels.forEach(k => {
      const col = document.createElement('div'); col.className = 'an-col';
      const bar = document.createElement('div'); bar.className = 'an-bar';
      bar.style.height = ((hist[k] || 0) / hmax * 60) + 'px';
      const lab = document.createElement('div'); lab.className = 'an-lab'; lab.textContent = k;
      const c = document.createElement('div'); c.className = 'an-cnt'; c.textContent = hist[k] || 0;
      col.append(c, bar, lab); ev.appendChild(col);
    });
    box.appendChild(ev);
    const kv = document.createElement('div');
    kv.className = 'an-kv';
    kv.innerHTML =
      '<div class="kv"><span>avg sentence</span><b>' + a.avgSentence + ' words</b></div>' +
      '<div class="kv"><span>adverb density</span><b>' + a.adverbDensity + '</b></div>' +
      '<div class="kv"><span>dialogue ratio</span><b>' + a.dialogueRatio + '</b></div>' +
      '<div class="kv"><span>readability</span><b>' + a.readability + ' /100</b></div>';
    box.appendChild(kv);
    // repeated-word cloud (flag obvious repetition)
    const cloud = document.createElement('div');
    cloud.className = 'an-cloud';
    if (a.repeated.length) {
      const maxC = a.repeated[0][1];
      a.repeated.forEach(([w, c]) => {
        const s = document.createElement('span');
        s.className = 'an-word' + (c >= 4 ? ' hot' : '');
        s.style.fontSize = (11 + (c / maxC) * 10) + 'px';
        s.textContent = w;
        s.title = c + '×';
        cloud.appendChild(s);
      });
    } else cloud.innerHTML = '<span class="hint">no repeated words</span>';
    box.appendChild(cloud);
    if (a.repeated.length >= 3 && a.repeated[0][1] >= 4) {
      const warn = document.createElement('div');
      warn.className = 'an-flag';
      warn.textContent = '⚑ repetitive: “' + a.repeated[0][0] + '” appears ' + a.repeated[0][1] + '× — consider varying word choice.';
      box.prepend(warn);
    }
  }

  // ---------- continuity (engine-backed) ----------
  async function loadContinuity() {
    const out = await runCli(['doc', 'ledger', p()]);
    const mp = $('anCont');
    // parse: continuity ledger: N entities, M flags
    const header = /continuity ledger: (\d+) entities, (\d+) flags/.exec(out);
    const flags = [];
    const reFlag = /^\s*FLAG \[(property-conflict|timeline-conflict|.+?)\] (.+?) — (.+)$/gm;
    let m;
    while ((m = reFlag.exec(out)) !== null) flags.push({ kind: m[1], entity: m[2], detail: m[3] });
    const entities = [];
    const reEnt = /entity (.+?): (\d+) mentions, first t=(\d+), last t=(\d+)/g;
    while ((m = reEnt.exec(out)) !== null) entities.push({ name: m[1], mentions: +m[2] });
    mp.innerHTML = '';
    if (header) {
      const h = document.createElement('div'); h.className = 'an-cont-head';
      h.textContent = header[2] + ' unresolved conflict' + (header[2] !== '1' ? 's' : '') + ' · ' + header[1] + ' tracked entities';
      mp.appendChild(h);
    }
    if (flags.length) {
      for (const f of flags) {
        const d = document.createElement('div');
        d.className = 'an-flag ' + (f.kind.includes('timeline') ? ' tl' : '');
        d.textContent = '⚠ ' + f.kind + ' — ' + f.entity + ': ' + f.detail;
        mp.appendChild(d);
      }
    } else {
      mp.appendChild(document.createElement('div')).className = 'hint'; // placeholder
      mp.lastChild.textContent = 'no unresolved continuity conflicts';
    }
    const el = document.createElement('div');
    el.className = 'an-ent';
    el.textContent = entities.length
      ? 'tracked: ' + entities.map(e => esc(e.name) + ' (' + e.mentions + '×)').join(', ')
      : 'no tracked entities yet (mentions appear as you write about them)';
    mp.appendChild(el);
  }

  async function refresh() {
    renderStyle(analyze($('anText').value));
    loadContinuity();
  }

  // ---------- wiring ----------
  $('btnAnRefresh').onclick = refresh;
  // when a doc is open, load its text into the analysis source
  async function loadDocText() {
    const id = curDoc();
    if (id == null) { $('anText').value = ''; $('anDoc').textContent = '— no open doc —'; return; }
    const sc = await readFile(sidecarDir() + '/doc-' + id + '.json');
    $('anDoc').textContent = 'doc #' + id;
    if (sc) { try { const o = JSON.parse(sc); $('anText').value = o.text || ''; } catch (e) {} }
    refresh();
  }
  $('anText').addEventListener('input', () => { /* analyze on demand via button; keep authoritative */ });
  window.addEventListener('load', () => {
    const btn = document.querySelector('nav button[data-tab="writing"]');
    if (btn) btn.addEventListener('click', loadDocText);
  });
  loadDocText();
})();
