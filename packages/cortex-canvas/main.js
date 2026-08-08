// Cortex Canvas M0 scaffold — static snapshot renderer.
// Reads snapshot.json next to index.html (fetch) or via drag & drop.
// Contract (DESIGN.md §3.5): every rendered metric must exist in the snapshot
// schema emitted by Brain::snapshot_json(). No invented fields.

const REGIONS = [
  // left hemisphere (detail/sequence emphasis)
  { side: 'L', id: 'prefrontal',  label: 'prefrontal executive',   color: '#3b82f6', metric: s => s.vigilance.energy * 0.6 + s.vigilance.attentionFocus * 0.4 },
  { side: 'L', id: 'language',    label: 'language interface',     color: '#38bdf8', metric: s => s.development.posture },
  { side: 'L', id: 'somatosensory', label: 'somatosensory strip',  color: '#22c55e', metric: s => s.embodied.bodyComfort },
  { side: 'L', id: 'motor-habit', label: 'habit strip',            color: '#84cc16', metric: s => s.vigilance.alertness },
  // right hemisphere (context/space emphasis)
  { side: 'R', id: 'prefrontal',  label: 'prefrontal executive',   color: '#3b82f6', metric: s => s.social.openness * 0.5 + s.vigilance.attentionFocus * 0.5 },
  { side: 'R', id: 'vestibular',  label: 'vestibular cluster',     color: '#22c55e', metric: s => s.embodied.motionComfort },
  { side: 'R', id: 'visual',      label: 'visual cortex',          color: '#22c55e', metric: s => s.vigilance.alertness * 0.7 + s.development.curiosity * 0.3 },
  { side: 'R', id: 'auditory',    label: 'auditory cortex',        color: '#22c55e', metric: s => s.social.peerPresence * 0.5 + s.vigilance.alertness * 0.5 },
  // shared mid regions
  { side: 'M', id: 'limbic',      label: 'limbic / salience',     color: '#f59e0b', metric: s => (Math.abs(s.affect.valence) + s.affect.arousal) / 2 },
  { side: 'M', id: 'insula',      label: 'insula / interoception', color: '#a855f7', metric: s => s.embodied.interoceptiveLoad },
  { side: 'M', id: 'hippocampus', label: 'episodic binder',        color: '#ec4899', metric: s => s.development.plasticityWindow },
  { side: 'M', id: 'development', label: 'development',            color: '#64748b', metric: s => s.development.posture },
];

const MOD_IDS = ['da', '5ht', 'ne', 'ach', 'ecb', 'cort', 'oxt', 'avp'];
const MOD_COLORS = ['#fbbf24', '#34d399', '#f87171', '#60a5fa', '#c084fc', '#fb923c', '#f472b6', '#94a3b8'];

const cv = document.getElementById('cv');
const ctx = cv.getContext('2d');
const tip = document.getElementById('tip');
const meta = document.getElementById('meta');
const statusEl = document.getElementById('status');

let SNAPSHOT = null;
let hovered = null;

function clamp01(x) { return Math.max(0, Math.min(1, x)); }

function valColor(hex, v) {
  // mix hex color with black by (1 - v); add glow when v high
  const r = parseInt(hex.slice(1, 3), 16), g = parseInt(hex.slice(3, 5), 16), b = parseInt(hex.slice(5, 7), 16);
  const k = clamp01(v);
  const rr = Math.round(r * k), gg = Math.round(g * k), bb = Math.round(b * k);
  return `rgb(${rr},${gg},${bb})`;
}

function draw() {
  ctx.clearRect(0, 0, cv.width, cv.height);
  if (!SNAPSHOT) {
    ctx.fillStyle = '#475569';
    ctx.font = '14px system-ui';
    ctx.fillText('drop a snapshot.json here (from: neuroform inspect <file> --json --out snapshot.json)', 240, 270);
    return;
  }
  const s = SNAPSHOT;
  const W = cv.width, H = cv.height;
  const cy = H / 2;

  // hemisphere lobes
  for (const [side, cx, rx] of [['L', W * 0.30, W * 0.26], ['R', W * 0.70, W * 0.26]]) {
    const g = ctx.createRadialGradient(cx, cy, 10, cx, cy, rx);
    const act = side === 'L' ? s.vigilance.attentionFocus : s.development.curiosity;
    g.addColorStop(0, `rgba(56,96,150,${0.10 + 0.20 * act})`);
    g.addColorStop(1, 'rgba(20,30,50,0.35)');
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.ellipse(cx, cy, rx, rx * 0.78, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = '#1e3a5f';
    ctx.lineWidth = 1;
    ctx.stroke();
  }
  // callosum bridge (M1: measured bandwidth; M0: static estimate)
  const bridge = 0.25 + 0.3 * clamp01(s.social.openness);
  ctx.strokeStyle = `rgba(148,163,184,${bridge})`;
  ctx.lineWidth = 14;
  ctx.beginPath();
  ctx.moveTo(W * 0.42, cy);
  ctx.quadraticCurveTo(W * 0.5, cy - 30, W * 0.58, cy);
  ctx.stroke();

  // region nodes
  hovered = null;
  const cx = cv.getBoundingClientRect().left + (mouseX - cv.getBoundingClientRect().left) || 0;
  const nodes = [];
  for (const r of REGIONS) {
    let x, y;
    if (r.side === 'L') { x = W * 0.13 + (r.id === 'prefrontal' ? 0 : 0.08) * W; y = cy - 70 + (REGIONS.indexOf(r) % 3) * 60; }
    else if (r.side === 'R') { x = W * 0.87 - (r.id === 'prefrontal' ? 0 : 0.08) * W; y = cy - 70 + (REGIONS.indexOf(r) % 3) * 60; }
    else { x = W * (0.44 + 0.12 * (REGIONS.indexOf(r) % 2)); y = cy + 40 + (REGIONS.indexOf(r) % 4) * 46; }
    const v = clamp01(r.metric(s));
    const rad = 7 + 9 * v;
    nodes.push({ ...r, x, y, v, rad });
    ctx.fillStyle = valColor(r.color, 0.25 + 0.75 * v);
    ctx.beginPath();
    ctx.arc(x, y, rad, 0, Math.PI * 2);
    ctx.fill();
    if (v > 0.55) {
      ctx.shadowColor = r.color; ctx.shadowBlur = 12;
      ctx.beginPath(); ctx.arc(x, y, rad, 0, Math.PI * 2); ctx.fill();
      ctx.shadowBlur = 0;
    }
  }

  // modulator ring (bottom)
  const ringY = H - 34;
  for (let i = 0; i < MOD_IDS.length; i++) {
    const v = clamp01(s.modulators[MOD_IDS[i]] || 0);
    const x = W * 0.32 + i * (W * 0.36 / 8);
    ctx.fillStyle = valColor(MOD_COLORS[i], 0.3 + 0.7 * v);
    ctx.fillRect(x, ringY - 16 * v, 8, 16 * v);
    ctx.fillStyle = '#64748b';
    ctx.font = '10px system-ui';
    ctx.fillText(MOD_IDS[i], x - 1, ringY + 14);
    nodes.push({ id: `mod-${MOD_IDS[i]}`, label: `modulator ${MOD_IDS[i]}`, side: 'M', x: x + 4, y: ringY - 8 * v, v, rad: 4 });
  }
  ctx.fillStyle = '#475569';
  ctx.font = '10px system-ui';
  ctx.fillText(`fullness ${(s.capacity.fullness * 100).toFixed(1)}%`, W - 150, H - 20);

  // mouse hover
  if (mouseInside) {
    let best = null, bestD = 1e9;
    for (const n of nodes) {
      const d = Math.hypot(n.x - mouseX, n.y - mouseY);
      if (d < Math.max(14, n.rad + 6) && d < bestD) { bestD = d; best = n; }
    }
    hovered = best;
    if (best) {
      tip.style.display = 'block';
      tip.style.left = (best.x + 14) + 'px';
      tip.style.top = (best.y - 8) + 'px';
      tip.textContent = `${best.label}: ${best.v.toFixed(3)}`;
    } else {
      tip.style.display = 'none';
    }
  }
  requestAnimationFrame(draw);
}

let mouseX = 0, mouseY = 0, mouseInside = false;
cv.addEventListener('mousemove', e => { const r = cv.getBoundingClientRect(); mouseX = e.clientX - r.left; mouseY = e.clientY - r.top; mouseInside = true; });
cv.addEventListener('mouseleave', () => { mouseInside = false; tip.style.display = 'none'; });

function apply(snap) {
  SNAPSHOT = snap;
  const s = snap;
  meta.textContent = `brain ${s.brainId ? s.brainId.slice(0, 8) : '?'} · tier ${s.tier} · sim ${s.simTime} ticks (${(s.simTime / 36000).toFixed(1)} h) · seed ${s.seed}`;
  statusEl.textContent = `valence ${s.affect.valence.toFixed(3)} · arousal ${s.affect.arousal.toFixed(3)} · energy ${s.vigilance.energy.toFixed(3)} · fatigue ${s.vigilance.fatigue.toFixed(3)} · stress ${s.stress.load.toFixed(3)} · curiosity ${s.development.curiosity.toFixed(3)}`;
}

// load snapshot.json (works on http servers; on file:// use make_preview.py)
fetch('snapshot.json').then(r => r.json()).then(apply).catch(() => {
  statusEl.textContent = 'no snapshot.json — drag & drop one, or run tools/make_preview.py';
});
// drag & drop fallback
document.addEventListener('dragover', e => e.preventDefault());
document.addEventListener('drop', e => {
  e.preventDefault();
  const f = e.dataTransfer.files[0];
  if (!f) return;
  const reader = new FileReader();
  reader.onload = () => { try { apply(JSON.parse(reader.result)); } catch { statusEl.textContent = 'bad JSON'; } };
  reader.readAsText(f);
});

requestAnimationFrame(draw);
