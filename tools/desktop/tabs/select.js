// Drawing tab — D5: Selections & Transform.
// Selection tools over the canvas: rect / ellipse / lasso / magic wand
// (tolerance + contiguous), select-all / deselect / invert, feather, move
// selection content, and transform (move / scale / rotate / flip H/V) with
// Enter (commit) / Escape (cancel). Bridges into the canvas via App.tool,
// selectionToolHandler, canvasInternals, and transformCommit.
(function () {
  const { $, esc } = window.App;
  const ci = () => window.App.canvasInternals;
  const W = 512, H = 512;

  // ---- selection mask: Uint8Array W*H, 1 = selected ----
  let mask = null;              // Uint8Array
  let bounds = null;            // {x0,y0,x1,y1} of selection content
  let marchingT = 0;

  // ---- tool mode wiring ----
  function setTool(t) {
    window.App.tool = t;
    document.querySelectorAll('#cvTools [data-tool]').forEach(b => b.classList.toggle('sel', b.dataset.tool === t));
    cancelTransform();
  }
  document.querySelectorAll('#cvTools [data-tool]').forEach(b => b.onclick = () => setTool(b.dataset.tool));

  // expose read-only selection info for verification/debug
  window.App.selectionInfo = () => ({ selCount: mask ? mask.reduce((a, v) => a + v, 0) : 0, bounds: bounds ? { ...bounds } : null });
  window.App.setTool = setTool;

  // ---- marching ants rendering (drawn over the composite) ----
  function clearMask() { mask = null; bounds = null; redraw(); }
  function redraw() {
    // repaint composite so any previous overlay is gone
    const cix = ci(); if (!cix) return;
    const cvs = $('#cvCanvas');
    const ctx = cix.getCtx();
    cix._drawPreview();   // recompose + draw stroke preview (clears overlays)
    if (!mask) return;
    // draw marching ants along selection boundary (crude: draw an outline on the visible ctx)
    const g = ctx;
    g.save();
    cix.viewTransform(g);
    drawBoundary(g);
    g.restore();
  }
  function drawBoundary(g) {
    if (!mask || !bounds) return;
    // simple rectangle boundary for transform clarity + fill tint for shape selections
    const { x0, y0, x1, y1 } = bounds;
    g.strokeStyle = 'rgba(255,255,255,0.85)';
    g.setLineDash([6, 4]);
    g.lineDashOffset = -marchingT;
    g.lineWidth = 1.2;
    g.strokeRect(x0, y0, x1 - x0, y1 - y0);
    // faint fill for the selected area, ignoring hard alpha (representational)
    g.globalAlpha = 0.12;
    g.fillStyle = '#4fc3ff';
    g.fillRect(x0 + 1, y0 + 1, x1 - x0 - 2, y1 - y0 - 2);
    g.globalAlpha = 1;
    g.setLineDash([]);
  }
  setInterval(() => { marchingT = (marchingT + 1) % 16; if (mask) redraw(); }, 120);
  redraw();

  // ---- make a mask from a path (rect/ellipse/lasso) ----
  function maskFromPoly(points, rectX1, rectY1) {
    const m = new Uint8Array(W * H);
    bounds = { x0: W, y0: H, x1: 0, y1: 0 };
    // rasterize: point-in-polygon for lasso; simple bounds fill for rect/ellipse
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const inside = points ? pointInPoly(x, y, points)
          : (rectX1 != null
            ? (x >= points.x && x <= rectX1 && y >= points.y && y <= rectY1)
            : inEllipse(x, y, points, { r: Math.max(1, Math.abs(rectX1 - points.x)), c: Math.max(1, Math.abs(rectY1 - points.y)) }));
        if (inside) { m[y * W + x] = 1; bounds.x0 = Math.min(bounds.x0, x); bounds.y0 = Math.min(bounds.y0, y); bounds.x1 = Math.max(bounds.x1, x); bounds.y1 = Math.max(bounds.y1, y); }
      }
    }
    if (bounds.x1 < bounds.x0) { mask = null; bounds = null; return null; }
    mask = m;
    return m;
  }
  function pointInPoly(x, y, pts) {
    let inside = false;
    for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
      const xi = pts[i].x, yi = pts[i].y, xj = pts[j].x, yj = pts[j].y;
      if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) inside = !inside;
    }
    return inside;
  }
  function inEllipse(x, y, c, r) { const dx = (x - c.x) / r.r, dy = (y - c.y) / r.c; return dx * dx + dy * dy <= 1; }

  // ---- magic wand ----
  function magicWand(cx, cy, tolerance, contiguous) {
    // read a scratch composite's pixel at (cx,cy)
    const scratch = document.createElement('canvas'); scratch.width = W; scratch.height = H;
    const g = scratch.getContext('2d');
    ci().compositeInto(g, ci().getLayers());
    const id = g.getImageData(0, 0, W, H).data;
    const base = seedAt(id, cx, cy);
    const m = new Uint8Array(W * H);
    const T = tolerance | 0;
    if (contiguous) {
      const seen = new Uint8Array(W * H);
      const stack = [[cx | 0, cy | 0]];
      while (stack.length) {
        const p = stack.pop(); const x = p[0], y = p[1];
        if (x < 0 || y < 0 || x >= W || y >= H || seen[idx(x, y)]) continue;
        const off = idx(x, y);
        seen[off] = 1;
        if (colorDistAt(id, off, base) > T) continue;    // stop at tolerance boundary
        m[off] = 1;
        stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
      }
    } else {
      for (let i = 0; i < W * H; i++) m[i] = (colorDistAt(id, i, base) <= T) ? 1 : 0;
    }
    mask = m;
    boundsFromMask();
    if (mask && !mask.some(v => v)) { mask = null; bounds = null; }
  }
  function seedAt(id, x, y) { const o = idx(x, y) * 4; return [id[o], id[o + 1], id[o + 2], id[o + 3]]; }
  function colorDistAt(id, i, base) { const o = i * 4; const dr = id[o] - base[0], dg = id[o + 1] - base[1], db = id[o + 2] - base[2]; return Math.max(0, Math.abs(dr) + Math.abs(dg) + Math.abs(db)) / 3; }
  function idx(x, y) { return (y | 0) * W + (x | 0); }
  function boundsFromMask() {
    bounds = { x0: W, y0: H, x1: 0, y1: 0 };
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) if (mask && mask[idx(x, y)]) {
      bounds.x0 = Math.min(bounds.x0, x); bounds.y0 = Math.min(bounds.y0, y);
      bounds.x1 = Math.max(bounds.x1, x); bounds.y1 = Math.max(bounds.y1, y);
    }
  }

  // ---- selection action buttons ----
  $('cvSelectAll').onclick = () => { mask = new Uint8Array(W * H).fill(1); bounds = { x0: 0, y0: 0, x1: W - 1, y1: H - 1 }; redraw(); };
  $('cvDeselect').onclick = clearMask;
  $('cvInvert').onclick = () => { if (!mask) return; for (let i = 0; i < mask.length; i++) mask[i] = 1 - mask[i]; boundsFromMask(); redraw(); };
  $('cvFeatherSel').onclick = () => { if (!mask) { $('cvStatus').textContent = 'make a selection first'; return; } const r = 2; feather(mask, r); boundsFromMask(); redraw(); };
  function feather(m, r) {
    const src = m.slice();
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const v = src[idx(x, y)];
      if (v) continue;
      for (let dy = -r; dy <= r; dy++) for (let dx = -r; dx <= r; dx++) {
        const nx = x + dx, ny = y + dy;
        if (nx >= 0 && ny >= 0 && nx < W && ny < H && src[idx(nx, ny)]) { m[idx(x, y)] = 1; dx = r + 1; dy = r + 1; x = W; y = H; break; }
      }
    }
  }

  // ---- move selection content (cut + translate) ----
  $('cvMoveSel').onclick = () => { if (!mask || !bounds) { $('cvStatus').textContent = 'make a selection first'; return; } window.App.dragMode = 'move'; $('cvStatus').textContent = 'drag with brush tool to move selection content'; };

  // ---- transform (scale/rotate/flip) ----
  let xform = null;   // { kind:'scale'|'rotate'|'flipH'|'flipV', angle, ...pending }
  $('cvScaleSel').onclick = () => { if (!mask) return; xform = { kind: 'scale', fx: 1, fy: 1 }; $('cvStatus').textContent = 'scale: +/- keys or drag handles · Enter to apply'; };
  $('cvRotateSel').onclick = () => { if (!mask) return; xform = { kind: 'rotate', angle: 0 }; $('cvStatus').textContent = 'rotate: arrows or R · Enter to apply'; };
  $('cvFlipH').onclick = () => applyFlip(true, false);
  $('cvFlipV').onclick = () => applyFlip(false, true);
  $('cvApplyX').onclick = () => { if (xform) commitTransform(); else $('cvStatus').textContent = 'no transform pending'; };
  $('cvCancelX').onclick = cancelTransform;
  function cancelTransform() { xform = null; $('cvStatus').textContent = 'transform cancelled'; redraw(); }
  // keyboard: Enter commit, Esc cancel, R rotate, arrows nudge
  document.addEventListener('keydown', e => {
    if (e.target.closest('input,select,textarea')) return;
    if (e.key === 'Enter' && xform) { commitTransform(); e.preventDefault(); }
    else if (e.key === 'Escape' && xform) { cancelTransform(); e.preventDefault(); }
    else if (e.key === 'r' && xform && xform.kind === 'rotate') { xform.angle = (xform.angle + 15) % 360; redraw(); }
  });

  function applyFlip(h, v) {
    if (!mask || !bounds) { $('cvStatus').textContent = 'make a selection first'; return; }
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    // copy only selected pixels from the active layer to the output, transformed
    const src = activeLayerCanvas();
    const g = out.getContext('2d');
    const { x0, y0, x1, y1 } = bounds;
    const cw = x1 - x0 + 1, ch = y1 - y0 + 1;
    g.drawImage(src, x0, y0, cw, ch, h ? W - x1 - 1 : x0, v ? H - y1 - 1 : y0, cw, ch);
    commitReplace(out);
    $('cvStatus').textContent = 'flipped' + (h ? ' H' : ' V');
  }
  function activeLayerCanvas() {
    const ly = window.App.activeDrawable ? window.App.activeDrawable() : null;
    if (ly && ly.canvas) return ly.canvas;
    // fallback: last plain layer
    const arr = ci().getLayers().filter(x => !x.kind);
    return arr[arr.length - 1].canvas;
  }
  function commitTransform() {
    if (!xform || !mask || !bounds) { $('cvStatus').textContent = 'nothing to apply'; return; }
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    const g = out.getContext('2d');
    const src = activeLayerCanvas();
    const { x0, y0, x1, y1 } = bounds;
    g.save();
    if (xform.kind === 'scale') { g.scale(xform.fx || 1, xform.fy || 1); g.drawImage(src, x0 / (xform.fx || 1), y0 / (xform.fy || 1), (x1 - x0 + 1) / (xform.fx || 1), (y1 - y0 + 1) / (xform.fy || 1)); }
    else if (xform.kind === 'rotate') {
      const cx = (x0 + x1) / 2, cy = (y0 + y1) / 2;
      g.translate(cx, cy); g.rotate(xform.angle * Math.PI / 180); g.translate(-cx, -cy);
      g.drawImage(src, 0, 0);
    }
    g.restore();
    commitReplace(out);
    xform = null;
    $('cvStatus').textContent = 'transform applied';
  }
  function commitReplace(out) {
    // blend transformed result ONLY where selected: isolated copy of selection from src
    const src = activeLayerCanvas();
    const final = document.createElement('canvas'); final.width = W; final.height = H;
    const fg = final.getContext('2d');
    fg.drawImage(src, 0, 0);             // start = original layer
    // clear the selection region in final then paste transformed in
    fg.save(); fg.globalCompositeOperation = 'destination-out';
    clearSel(fg); fg.restore();
    fg.drawImage(out, 0, 0);
    // call undo-aware commit
    if (window.App.transformCommit) window.App.transformCommit(final);
    redraw();
  }
  function clearSel(g) {
    if (!mask || !bounds) return;
    // draw the mask as a shape to erase selected region
    // (fill rects per horizontal run for correctness)
    const img = document.createElement('canvas'); img.width = W; img.height = H;
    const ig = img.getContext('2d');
    // build a white image only where selected, then destination-out that
    const id = ig.createImageData(W, H);
    const buf = id.data;
    for (let i = 0; i < mask.length; i++) if (mask[i]) buf[i * 4 + 3] = 255;
    ig.putImageData(id, 0, 0);
    g.drawImage(img, 0, 0);
  }

  // ---- drag: move selection content with brush tool (simplified) ----
  let dragValid = false;
  window.App.selectionToolHandler = {
    down(e) {
      const pt = ci().viewToDoc(e);
      if (window.App.tool === 'rect') { rectStart = pt; selPoly = null; dragValid = true; }
      else if (window.App.tool === 'ellipse') { rectStart = pt; elSel = true; dragValid = true; }
      else if (window.App.tool === 'lasso') { lassoPts = [pt]; dragValid = true; }
      else if (window.App.tool === 'wand') { magicWand(pt.x, pt.y, $('cvTolerance').value, $('cvContig').checked); redraw(); $('cvStatus').textContent = 'wand select'; }
      else if (window.App.dragMode === 'move' && mask) { dragFrom = pt; dragValid = true; }
    },
    move(e) {
      const pt = ci().viewToDoc(e);
      if (!dragValid) return;
      if (window.App.tool === 'rect') { selPoly = null; elSel = false; liveRect = { a: rectStart, b: pt }; redraw(); }
      else if (window.App.tool === 'ellipse') { elSel = true; liveEll = { a: rectStart, b: pt }; redraw(); }
      else if (window.App.tool === 'lasso') { lassoPts = [...lassoPts, pt]; drawLasso(); }
      else if (window.App.dragMode === 'move' && dragFrom && mask) { dragDelta = { dx: pt.x - dragFrom.x, dy: pt.y - dragFrom.y }; redraw(); }
    },
    up(e) {
      if (!dragValid) return; dragValid = false;
      if (window.App.tool === 'rect' && liveRect) { maskFromPoly(liveRect.a, liveRect.b.x, liveRect.b.y); liveRect = null; }
      else if (window.App.tool === 'ellipse' && liveEll) { maskFromPoly(liveEll.a, liveEll.b.x, liveEll.b.y); liveEll = null; }
      else if (window.App.tool === 'lasso' && lassoPts && lassoPts.length > 2) { maskFromPoly(lassoPts, null, null); lassoPts = null; }
      else if (window.App.dragMode === 'move' && dragDelta) { applyMoveDrag(); }
      window.App.dragMode = null; dragFrom = null; dragDelta = null;
      redraw();
    },
  };
  let rectStart = null, liveRect = null, liveEll = null, elSel = false, selPoly = null, lassoPts = null, dragFrom = null, dragDelta = null;
  function drawLasso() {
    // rasterize the lasso immediately as a poly selection preview
    redraw();
    const g = ci().getCtx(); g.save(); ci().viewTransform(g);
    g.strokeStyle = 'rgba(255,255,255,0.9)'; g.lineWidth = 1.3;
    g.beginPath(); g.moveTo(lassoPts[0].x, lassoPts[0].y);
    for (let i = 1; i < lassoPts.length; i++) g.lineTo(lassoPts[i].x, lassoPts[i].y);
    g.closePath(); g.stroke(); g.restore();
  }
  function applyMoveDrag() {
    if (!mask || !bounds || !dragDelta) return;
    const src = activeLayerCanvas();
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    const g = out.getContext('2d');
    const dx = Math.round(dragDelta.dx), dy = Math.round(dragDelta.dy);
    const { x0, y0, x1, y1 } = bounds;
    // copy selected pixels to scratch, clear source region, paste at offset
    const sel = document.createElement('canvas'); sel.width = W; sel.height = H;
    const sg = sel.getContext('2d');
    sg.drawImage(src, 0, 0);
    const id = sg.getImageData(0, 0, W, H);
    const buf = id.data;
    for (let i = 0; i < mask.length; i++) if (mask[i]) buf[i * 4 + 3] = 0;  // clear selected
    sg.putImageData(id, 0, 0);
    // draw cleared src, then paste the grabbed selection translated
    g.drawImage(sel, 0, 0);
    const grab = document.createElement('canvas'); grab.width = W; grab.height = H;
    const gg = grab.getContext('2d');
    gg.drawImage(src, x0, y0, x1 - x0 + 1, y1 - y0 + 1, x0, y0, x1 - x0 + 1, y1 - y0 + 1);
    const gid = gg.getImageData(0, 0, W, H); const gbuf = gid.data;
    for (let i = 0; i < mask.length; i++) if (!mask[i]) gbuf[i * 4 + 3] = 0;  // keep only selected
    gg.putImageData(gid, 0, 0);
    g.drawImage(grab, dx, dy);
    if (window.App.transformCommit) window.App.transformCommit(out);
    $('cvStatus').textContent = 'moved selection';
  }

  // init
})();
