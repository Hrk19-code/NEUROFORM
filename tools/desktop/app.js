// neuroform desktop — app.js (shell)
// The UI is a thin shell over the real, tested CLI. /api/run spawns the exe;
// /api/state reads brain state in-process (no spawn) for the live dashboard.

const $ = id => document.getElementById(id);
const p = () => $('path').value.trim();
const v = id => $(id).value.trim();
const esc = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const pill = (t, cls) => { const e = $('pill'); e.textContent = t; e.className = 'pill' + (cls ? ' ' + cls : ''); };
// The brain's name: its file name (the two-cursor labels use it).
const brainName = () => (p().split(/[\\/]/).pop() || 'brain');

async function run(argv) {
  const log = $('log');
  log.innerHTML += `<div class="cmd">$ ${esc(argv.join(' '))}</div>`;
  log.scrollTop = log.scrollHeight;
  pill('working…');
  try {
    const r = await fetch('/api/run?args=' + encodeURIComponent(argv.join('\u001f')));
    const j = await r.json();
    const out = (j.stdout || '') + (j.stderr ? '\n[stderr] ' + j.stderr : '');
    log.innerHTML += `<div class="${j.ok ? 'ok' : 'err'}">${esc(out || (j.ok ? '(no output)' : 'code ' + j.code))}</div>`;
    pill(j.ok ? 'ok' : 'err ' + j.code, j.ok ? 'live' : '');
  } catch (e) {
    log.innerHTML += `<div class="err">request failed: ${esc(e)}</div>`;
    pill('offline');
  }
  log.scrollTop = log.scrollHeight;
  refreshState();
  if (window.App && window.App.onRunDone) { window.App.onRunDone(); }
}

// ---------- state dashboard ----------
let lastState = null;
async function refreshState() {
  try {
    const r = await fetch('/api/state?path=' + encodeURIComponent(p()));
    const j = await r.json();
    if (!j.ok) { $('pill').textContent = 'no brain'; return; }
    render(j.state);
    pill('live', 'live');
  } catch (e) { /* server restarting */ }
}

function render(d) {
  lastState = d;
  const aff = d.affect || {}, mods = d.modulators || {}, mem = d.memory || {}, emb = d.embodiment || {};
  const f = (x, n = 2) => (typeof x === 'number' ? x.toFixed(n) : '—');
  $('vVal').textContent = f(aff.valence);  $('vBar').style.width = ((aff.valence + 1) / 2 * 100) + '%';
  $('aVal').textContent = f(aff.arousal);  $('aBar').style.width = (aff.arousal * 100) + '%';
  $('wVal').textContent = f(aff.warmth);   $('sVal').textContent = f(aff.safety); $('lVal').textContent = f(aff.loneliness);
  $('tVal').textContent = f(aff.valence);
  $('mTraces').textContent = mem.traces ?? '—';
  $('mNodes').textContent = mem.semanticNodes ?? mem.nodes ?? '—';
  $('mPruned').textContent = mem.prunedTraces ?? '—';
  $('mSleep').textContent = d.sleep ? f(d.sleep.pressure) : '—';
  $('mDreams').textContent = d.sleep ? d.sleep.dreams ?? '—' : '—';
  $('fId').textContent = d.brainId || '—';
  $('fTier').textContent = (d.tier || '—') + ' / ' + (d.seed ?? '—');
  $('fTime').textContent = (d.simTime ?? '—') + ' ticks';
  $('fKary').textContent = emb.karyotype ?? '—';
  $('fPreset').textContent = emb.preset ?? '—';
  $('fInit').textContent = d.autonomy ? (d.autonomy.total ?? d.autonomy.initiatives ?? '—') : '—';
  $('fEnc').textContent = d.encoder ?? '—';
  const order = ['da', 'ne', '5ht', 'cort', 'oxt', 'avp', 'ach', 'ecb'];
  const labels = { da: 'dopamine', ne: 'noradrenaline', '5ht': 'serotonin', cort: 'cortisol', oxt: 'oxytocin', avp: 'vasopressin', ach: 'acetylcholine', ecb: 'endocannabinoid' };
  $('mods').innerHTML = order.map(k => mods[k] != null
    ? `<div class="kv"><span>${labels[k]}</span><b>${f(mods[k])}</b></div><div class="bar"><i class="${k === 'oxt' ? 'ok' : ''}" style="width:${mods[k] * 100}%"></i></div>`
    : '').join('');
  const ctx = (d.body && d.body.cortex) || null;
  if (ctx && Array.isArray(ctx)) {
    $('cortex').innerHTML = ctx.map(r => `<span class="chip ${(r.level || 0) > 0.3 ? 'hot' : ''}"><i></i>${esc(r.region || r.name || '?')} ${f(r.level)}</span>`).join('');
  } else if (ctx && typeof ctx === 'object') {
    $('cortex').innerHTML = Object.entries(ctx).map(([k, x]) => `<span class="chip ${x > 0.3 ? 'hot' : ''}"><i></i>${esc(k)} ${f(x)}</span>`).join('');
  } else { $('cortex').innerHTML = '<span class="hint">cortex data appears with body activity</span>'; }
  if (d.physics && d.physics.rates) {
    $('physics').innerHTML = Object.entries(d.physics.rates).map(([k, x]) => `<div class="kv"><span>${esc(k)}</span><b>${f(x)}</b></div>`).join('');
  }
  if (d.voice) {
    $('voPitch').textContent = d.voice.pitch != null ? f(d.voice.pitch) : '—';
    $('voRange').textContent = d.voice.range != null ? f(d.voice.range) : '—';
    $('voMaturity').textContent = d.voice.maturity != null ? f(d.voice.maturity) : '—';
    $('voHeard').textContent = d.voice.heardVoices ?? d.voice.heard ?? '—';
    $('voGate').textContent = d.voice.mimicryGate != null ? (d.voice.mimicryGate ? 'open' : 'off') : '—';
  }
  if (d.network) { renderRels(d.network); }
  if (d.lineage) { renderLineage(d.lineage); }
  renderCursors(d);
  drawFace(aff);
  if (window.App && window.App.onState) { window.App.onState(d); }
}

// Run a command and return parsed JSON stdout (null on failure).
async function runJson(argv) {
  try {
    const r = await fetch('/api/run?args=' + encodeURIComponent(argv.join('\u001f')));
    const j = await r.json();
    if (!j.ok || !j.stdout) return null;
    try { return JSON.parse(j.stdout); } catch (e) { return null; }
  } catch (e) { return null; }
}

// ---------- two named cursors (user vs brain) ----------
function renderCursors(d) {
  const w = d.writing || {}, dr = d.drawing || {};
  $('wBrainCur').textContent = '▍' + brainName();
  $('wBrainPos').textContent = w.cursorDoc != null ? `doc #${w.cursorDoc} · block #${w.cursorBlock}` : 'no document yet';
  $('wBrainTxt').textContent = w.cursorText ? 'last written: “' + w.cursorText + '…”' : '';
  $('dBrainCur').textContent = '✛ ' + brainName();
  $('dBrainPos').textContent = dr.cursorX != null
    ? `canvas #${dr.cursorCanvas} @ (${Math.round(dr.cursorX)}, ${Math.round(dr.cursorY)}) · ${dr.cursorColor || ''}`
    : 'no strokes yet';
  const cv = $('curCanvas'); if (!cv) return;
  const x = cv.getContext('2d'); const S = cv.width;
  x.clearRect(0, 0, S, S);
  x.strokeStyle = '#1a2530'; x.lineWidth = 1;
  for (let i = 0; i <= S; i += 32) { x.beginPath(); x.moveTo(i, 0); x.lineTo(i, S); x.stroke(); x.beginPath(); x.moveTo(0, i); x.lineTo(S, i); x.stroke(); }
  if (dr.cursorX != null) {
    // Preview is 256x256; canvas dims are read from the engine in D1 — the
    // marker is scaled to the preview for now (documented).
    const sx = Math.min(S - 4, Math.max(4, dr.cursorX)), sy = Math.min(S - 4, Math.max(4, dr.cursorY));
    x.strokeStyle = dr.cursorColor || '#ffb347'; x.lineWidth = 2;
    x.beginPath(); x.arc(sx, sy, 9, 0, Math.PI * 2); x.stroke();
    x.beginPath(); x.moveTo(sx - 14, sy); x.lineTo(sx + 14, sy); x.moveTo(sx, sy - 14); x.lineTo(sx, sy + 14); x.stroke();
    x.fillStyle = dr.cursorColor || '#ffb347';
    x.beginPath(); x.arc(sx, sy, 3, 0, Math.PI * 2); x.fill();
    x.font = '11px Segoe UI, sans-serif'; x.fillText(brainName(), sx + 16, sy - 10);
  } else {
    x.fillStyle = '#33475a'; x.font = '12px Segoe UI, sans-serif';
    x.fillText('no strokes yet', 12, 20);
  }
}

function renderRels(net) {
  const rels = net.relationships || net.rels;
  if (!rels || !rels.length) { $('rels').innerHTML = '<span class="hint">pair with another brain to see bonds form</span>'; return; }
  $('rels').innerHTML = rels.map(r => {
    const fam = (r.familiarity ?? 0) * 100;
    return `<div class="kv"><span>${esc(r.peerId || r.peer_id || '?')}</span><b>fam ${f(fam, 0)}% · trust ${f(r.trust, 2)}</b></div><div class="bar"><i class="ok" style="width:${fam}%"></i></div>`;
  }).join('');
}

function renderLineage(lg) {
  const mom = lg.motherId || lg.mother_id, dad = lg.fatherId || lg.father_id;
  if (!mom && !dad) { $('lineage').innerHTML = '<span class="hint">first-generation file — no parents</span>'; return; }
  $('lineage').innerHTML = `<div class="kv"><span>mother</span><b>${esc(mom || '—')}</b></div><div class="kv"><span>father</span><b>${esc(dad || '—')}</b></div>`;
}

// ---------- the face ----------
function drawFace(aff) {
  const c = $('face'); if (!c) return;
  const x = c.getContext('2d'); const w = c.width;
  x.clearRect(0, 0, w, w);
  const val = aff.valence ?? 0, aro = aff.arousal ?? 0.3;
  const blink = Math.sin(Date.now() / 4000) > 0.92 ? 0.15 : 1;
  const cx = w / 2, cy = w / 2 + 10;
  x.beginPath(); x.ellipse(cx, cy, w * 0.34, w * 0.4, 0, 0, Math.PI * 2);
  x.fillStyle = '#22303e'; x.fill(); x.strokeStyle = '#33475a'; x.lineWidth = 2; x.stroke();
  const ex = w * 0.38, ey = cy - w * 0.09, eo = w * 0.09, gap = w * 0.16;
  [ex, ex + gap].forEach(px => {
    x.beginPath(); x.ellipse(px, ey, eo, eo * 0.75 * blink, 0, 0, Math.PI * 2);
    x.fillStyle = '#0d141c'; x.fill();
    const pr = px + (val * 4), py = ey + (aro * 3 - 2);
    x.beginPath(); x.arc(pr, py, eo * 0.38, 0, Math.PI * 2); x.fillStyle = '#4fc3ff'; x.fill();
    x.beginPath(); x.arc(pr, py, eo * 0.16, 0, Math.PI * 2); x.fillStyle = '#0a0e13'; x.fill();
  });
  const mx = cx, my = cy + w * 0.16, mw = w * 0.16;
  x.beginPath();
  if (val >= 0) { x.arc(mx, my, mw, 0.15 * Math.PI, 0.85 * Math.PI); } else { x.arc(mx, my + mw * 0.8, mw, 1.15 * Math.PI, 1.85 * Math.PI); }
  x.strokeStyle = val > 0.2 ? '#5fe3a0' : val < -0.2 ? '#ff7d7d' : '#8ba5b8';
  x.lineWidth = 2.5; x.stroke();
  const gl = Math.min(0.5, Math.max(0, aro - 0.3)) * 0.5;
  [cx - w * 0.2, cx + w * 0.2].forEach(px => {
    const g = x.createRadialGradient(px, my - w * 0.02, 1, px, my - w * 0.02, w * 0.09);
    g.addColorStop(0, `rgba(255,107,138,${gl})`); g.addColorStop(1, 'rgba(255,107,138,0)');
    x.fillStyle = g; x.beginPath(); x.arc(px, my - w * 0.02, w * 0.09, 0, Math.PI * 2); x.fill();
  });
}

// ---------- shell wiring ----------
document.querySelectorAll('#tabs button').forEach(b => b.onclick = () => {
  document.querySelectorAll('#tabs button').forEach(x => x.classList.remove('active'));
  document.querySelectorAll('.tab').forEach(x => x.classList.remove('active'));
  b.classList.add('active'); $('tab-' + b.dataset.tab).classList.add('active');
});
$('btnState').onclick = refreshState;
// activity log: collapsible floating window (starts collapsed so tabs have room)
const act = $('activity');
function actToggle() { act.classList.toggle('collapsed'); }
$('actHead').onclick = actToggle;
act.classList.add('collapsed');
$('btnNew').onclick = () => {
  const n = p() || 'new.brain';
  const args = ['create', n, '--tier', 'standard', '--chromosomes', 'xx', '--seed', String(Date.now() % 100000)];
  const enc = $('encSel').value;
  if (enc !== 'handcrafted') { args.push('--encoder', enc); }
  run(args);
};
refreshState();
setInterval(refreshState, 5000);

// shared API for tab modules (loaded after app.js)
window.App = { run, runJson, refreshState, getState: () => lastState, $, p, v, esc, pill };
