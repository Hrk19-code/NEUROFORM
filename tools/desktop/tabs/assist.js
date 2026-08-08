// Drawing tab — D7: Assist Tools. Vertical/horizontal symmetry (live mirror in
// the stroke pipeline, wired via App.symmetry), straight-line + shape tools
// (line/rect/ellipse, outline or fill) committed via transformCommit, a
// non-destructive perspective guide overlay (1/2-point vanishing rulers), and a
// reference board (load an image, opacity slider — sits in a side panel).
(function () {
  const { $, esc } = window.App;
  const ci = () => window.App.canvasInternals;
  const W = 512, H = 512;

  // ---- symmetry toggle ----
  $('cvSymV').onclick = function () {
    const s = window.App.symmetry;
    s.on = !s.on;
    s.axis = s.on ? 'vertical' : 'vertical';
    this.classList.toggle('pri', s.on);
    $('cvStatus').textContent = s.on ? 'vertical symmetry ON (mirrors live)' : 'symmetry off';
  };

  // ---- perspective guide overlay (non-destructive) ----
  let guide = { on: false, horizon: 256, leftVP: -400, rightVP: 900 };
  function drawGuide(g) {
    if (!guide.on) return;
    g.save();
    g.strokeStyle = 'rgba(120,200,255,0.35)'; g.lineWidth = 1; g.setLineDash([6, 4]);
    g.globalAlpha = 0.6;
    // horizon
    g.beginPath(); g.moveTo(0, guide.horizon); g.lineTo(W, guide.horizon); g.stroke();
    // vanishing point rays (1-point: use two symmetric VPs for 2-point effect)
    for (let i = 0; i <= 8; i++) {
      const x = 40 + i * 60;
      // rays to left VP and right VP
      g.beginPath(); g.moveTo(x, H); g.lineTo(guide.leftVP, guide.horizon); g.stroke();
      g.beginPath(); g.moveTo(x, H); g.lineTo(guide.rightVP, guide.horizon); g.stroke();
    }
    g.restore();
  }
  $('cvGuide').onclick = function () {
    guide.on = !guide.on;
    this.classList.toggle('pri', guide.on);
    if (guide.on) window.App._overlays.push(drawGuide);
    else window.App._overlays = window.App._overlays.filter(f => f !== drawGuide);
    redrawCanvas();
    $('cvStatus').textContent = guide.on ? 'perspective guide on (1/2-point, non-destructive)' : 'guide off';
  };
  function redrawCanvas() { try { ci()._drawPreview(); } catch (_) {} }

  // ---- shape tools (line / rect / ellipse, outline or fill) ----
  const prev = window.App.selectionToolHandler;
  let shapeStart = null, shapeEndCur = null;
  window.App.shapeFill = 'outline';   // outline | fill
  $('cvShapeFill').onchange = e => { window.App.shapeFill = e.target.value; };

  function commitShape(kind, a, b) {
    const ly = window.App.activeDrawable ? window.App.activeDrawable() : null;
    if (!ly || ly.kind === 'group') { $('cvStatus').textContent = 'pick a paint layer first'; return; }
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    const g = out.getContext('2d');
    g.drawImage(ly.canvas, 0, 0);
    const fg = window.App.getColor ? window.App.getColor() : '#ff6633';
    g.fillStyle = fg; g.strokeStyle = fg; g.lineWidth = Math.max(1, (window.App.getBrushSize ? window.App.getBrushSize() : 4));
    g.beginPath();
    const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y), w = Math.abs(b.x - a.x), h = Math.abs(b.y - a.y);
    if (kind === 'line') { g.moveTo(a.x, a.y); g.lineTo(b.x, b.y); }
    else if (kind === 'rect') g.rect(x, y, Math.max(1, w), Math.max(1, h));
    else if (kind === 'ellipse') g.ellipse((a.x + b.x) / 2, (a.y + b.y) / 2, Math.max(1, w / 2), Math.max(1, h / 2), 0, 0, Math.PI * 2);
    if (window.App.shapeFill === 'fill') g.fill(); else g.stroke();
    if (window.App.transformCommit) window.App.transformCommit(out);
    $('cvStatus').textContent = kind + (window.App.shapeFill === 'fill' ? ' (fill)' : ' (outline)');
  }
  function previewShape(g) {
    if (!shapeStart || !shapeEndCur) return;
    g.save(); ci().viewTransform(g);
    g.globalAlpha = 0.6; g.strokeStyle = '#ffffff'; g.fillStyle = '#ffffff'; g.lineWidth = 1;
    g.beginPath();
    const a = shapeStart, b = shapeEndCur;
    const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y), w = Math.abs(b.x - a.x), hh = Math.abs(b.y - a.y);
    g.rect(x, y, w, hh);
    g.stroke(); g.restore();
  }

  // ---- reference board ----
  let refImg = null;
  function renderRef() {
    const board = $('cvRefBoard');
    if (refImg) { board.innerHTML = ''; const img = new Image(); img.src = refImg; img.style.opacity = $('cvRefOp').value; board.appendChild(img); }
    else board.innerHTML = '<span class="placeholder">load a reference image here</span>';
  }
  $('cvRefFile').addEventListener('change', e => {
    const f = e.target.files[0]; if (!f) return;
    const rd = new FileReader();
    rd.onload = () => { refImg = rd.result; renderRef(); };
    rd.readAsDataURL(f);
  });
  $('cvRefClear').onclick = () => { refImg = null; $('cvRefFile').value = ''; renderRef(); };
  $('cvRefOp').oninput = () => renderRef();

  // ---- wire shape tools through the tool routing ----
  if (prev) {
    const _down = prev.down, _move = prev.move, _up = prev.up;
    prev.down = function (e) {
      if (window.App.tool === 'line' || window.App.tool === 'rect-shape' || window.App.tool === 'ellipse-shape') {
        shapeStart = ci().viewToDoc(e); shapeEndCur = null; return;
      }
      if (_down) return _down(e);
    };
    prev.move = function (e) {
      if (shapeStart && (window.App.tool === 'line' || window.App.tool === 'rect-shape' || window.App.tool === 'ellipse-shape')) {
        shapeEndCur = ci().viewToDoc(e);
        const cix = ci(); const g = cix.getCtx(); g.save(); cix.viewTransform(g);
        g.strokeStyle = 'rgba(120,200,255,0.9)'; g.lineWidth = 1.2; g.setLineDash([4, 3]);
        g.beginPath();
        const a = shapeStart, b = shapeEndCur;
        if (window.App.tool === 'line') { g.moveTo(a.x, a.y); g.lineTo(b.x, b.y); }
        else { const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y), w = Math.abs(b.x - a.x), hh = Math.abs(b.y - a.y); g.rect(x, y, w, hh); }
        g.stroke(); g.setLineDash([]); g.restore();
        return;
      }
      if (_move) return _move(e);
    };
    prev.up = function (e) {
      if (shapeStart && (window.App.tool === 'line' || window.App.tool === 'rect-shape' || window.App.tool === 'ellipse-shape')) {
        shapeEndCur = ci().viewToDoc(e);
        const kind = window.App.tool === 'line' ? 'line' : window.App.tool === 'rect-shape' ? 'rect' : 'ellipse';
        commitShape(kind, shapeStart, shapeEndCur);
        shapeStart = shapeEndCur = null;
        redrawCanvas();
        return;
      }
      if (_up) return _up(e);
    };
  }

  renderRef();
})();
