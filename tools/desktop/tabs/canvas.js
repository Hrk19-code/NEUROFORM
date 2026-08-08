// Drawing tab — D2: Layers (extends the D1 canvas engine).
// A real painter: viewport zoom/pan, pressure strokes, stabilizer, undo/redo,
// and now a full layer stack: per-layer offscreen canvases, visibility, lock,
// opacity, blend modes (normal, multiply, screen, overlay, luminosity, add),
// drag-reorder, merge down, flatten visible, layer groups (folder + group opacity).
// Save strokes into the brain via `draw stroke --save` (binds memory + ticks).
(function () {
  const { $, p, run, runJson, esc } = window.App;

  // ---- canvas state ----
  const cvs = $('cvCanvas');
  const ctx = cvs.getContext('2d');
  const W = 512, H = 512;         // logical canvas document
  cvs.width = W; cvs.height = H;

  // ---- layer stack ----
  // each layer is { id, name, canvas, visible, locked, opacity, blend, group }
  // group is null for top-level layers, or a group id for members.
  let layers = [];
  // the paper layer (bottom) is white; others start transparent.
  function mkLayer(name, white) {
    const c = document.createElement('canvas'); c.width = W; c.height = H;
    const g = c.getContext('2d');
    if (white) { g.fillStyle = '#ffffff'; g.fillRect(0, 0, W, H); }
    return { id: nextId(), name, canvas: c, visible: true, locked: false, opacity: 1, blend: 'normal', group: null };
  }
  let _id = 0; function nextId() { return ++_id; }
  // start with the paper layer + one paint layer on top
  layers.push(mkLayer('paper', true));
  layers.push(mkLayer('layer 1', false));
  let activeLayer = 1;            // index into layers[]

  let view = { zoom: 1, panX: 0, panY: 0 };
  let currentCanvas = 1, currentLayer = 1;
  let undoStack = [], redoStack = [];
  const MAX_UNDO = 40;

  // ---- brush model (D3) ----
  // params: size, minSize (pressure floor), opacity, flow (buildup), hardness,
  // spacing, mode ('paint'|'erase-hard'|'erase-soft'), color
  let brush = { color: '#ff6633', size: 24, minSize: 2, opacity: 1, flow: 1, hardness: 1, spacing: 0.35, mode: 'paint', stabilizer: 4 };
  // eraser acts as a paint brush that writes transparent (alpha only)
  function brushIsEraser() { return brush.mode !== 'paint'; }
  function brushAlpha() { return brushIsEraser() ? 1 : brush.opacity; }

  // built-in presets (>= 10): name -> partial brush overrides
  const BRUSH_PRESETS = [
    { name: 'pen', size: 4, minSize: 1, hardness: 1, opacity: 1, flow: 1, spacing: 0.45, mode: 'paint' },
    { name: 'hard round', size: 18, minSize: 3, hardness: 0.95, opacity: 1, flow: 1, spacing: 0.25, mode: 'paint' },
    { name: 'soft airbrush', size: 40, minSize: 6, hardness: 0.05, opacity: 0.35, flow: 0.4, spacing: 0.12, mode: 'paint' },
    { name: 'marker', size: 14, minSize: 4, hardness: 0.7, opacity: 0.7, flow: 0.9, spacing: 0.2, mode: 'paint' },
    { name: 'watercolor-ish', size: 30, minSize: 5, hardness: 0.15, opacity: 0.5, flow: 0.6, spacing: 0.2, mode: 'paint' },
    { name: 'charcoal', size: 22, minSize: 2, hardness: 0.4, opacity: 0.8, flow: 0.7, spacing: 0.3, mode: 'paint' },
    { name: 'ink dash', size: 8, minSize: 1, hardness: 0.85, opacity: 1, flow: 1, spacing: 0.7, mode: 'paint' },
    { name: 'eraser hard', size: 24, minSize: 4, hardness: 1, opacity: 1, flow: 1, spacing: 0.25, mode: 'erase-hard' },
    { name: 'eraser soft', size: 40, minSize: 6, hardness: 0.1, opacity: 1, flow: 1, spacing: 0.15, mode: 'erase-soft' },
    { name: 'spray', size: 60, minSize: 10, hardness: 0.02, opacity: 0.3, flow: 0.3, spacing: 0.5, mode: 'paint' },
  ];
  let customPresets = loadCustomPresets();
  function loadCustomPresets() { try { return JSON.parse(localStorage.getItem('nf-custom-brushes') || '[]'); } catch (_) { return []; } }
  function saveCustomPresets() { try { localStorage.setItem('nf-custom-brushes', JSON.stringify(customPresets)); } catch (_) {} }
  function applyBrush(p) { Object.assign(brush, p); syncBrushUI(); }

  // brush stamp registry: keyed by color|radius|hardness|mode so erasers/soft edges cache too
  const brushStamps = (() => {
    const map = {};
    const mk = (color, r, hardness) => {
      const c = document.createElement('canvas'); c.width = c.height = r * 2;
      const g = c.getContext('2d');
      // hard: sharp inner alpha; soft: radial falloff to transparent. hard=1 -> opaque core.
      const stops = hardness >= 0.98
        ? [[0, 'ff'], [0.75, 'ff'], [1, '00']]
        : [[0, 'ff'], [1 - hardness * 0.9, 'ff'], [1, '00']];
      const gr = g.createRadialGradient(r, r, 0, r, r, r);
      gr.addColorStop(0, color + 'ff');
      gr.addColorStop(stops[1][0], color + stops[1][1]);
      gr.addColorStop(1, color + '00');
      g.fillStyle = gr; g.beginPath(); g.arc(r, r, r, 0, Math.PI * 2); g.fill();
      return c;
    };
    const stampColor = () => brushIsEraser() ? '#000000' : brush.color;  // eraser writes transparent pixels
    return {
      get: (color, size, hardness, modeKey) => {
        const r = Math.max(1, Math.round(size));
        const h = Math.round((hardness != null ? hardness : brush.hardness) * 100);
        const key = (modeKey || brush.mode) + '|' + color + '|' + r + '|' + h;
        if (!map[key]) map[key] = mk(color, r, h / 100);
        return map[key];
      },
      color: stampColor,
    };
  })();

  // ---- view transform: doc (512x512 buffer) <-> screen (CSS-scaled) ----
  function screenToBuf(e) {
    const r = cvs.getBoundingClientRect();
    return { x: (e.clientX - r.left) * (W / r.width), y: (e.clientY - r.top) * (H / r.height) };
  }
  function bufToDoc(s) { return { x: (s.x - view.panX) / view.zoom, y: (s.y - view.panY) / view.zoom }; }
  function viewToDoc(e) { return bufToDoc(screenToBuf(e)); }
  function docToView(x, y) { return { x: x * view.zoom + view.panX, y: y * view.zoom + view.panY }; }

  // ---- compositing: all visible layers bottom->top with blend + opacity ----
  // groups: a group row holds { kind:'group', name, open, opacity, members:[] };
  // members composite inside the group with the group's opacity multiplied in.
  // D6: each layer may have a `mask` canvas (white=show, black=hide) applied
  // non-destructively, and a `clipping` flag (layer shows only where the layer
  // below has alpha — SAI behavior).
  const BLEND = ['normal', 'multiply', 'screen', 'overlay', 'luminosity', 'add'];
  function applyLayerMask(g, it) {
    if (!it.mask) return;
    g.save();
    g.globalCompositeOperation = 'destination-in';   // keep only where mask is opaque
    g.drawImage(it.mask, 0, 0);
    g.restore();
  }
  function drawLayer(g, it, groupOp) {
    g.save();
    g.globalAlpha = (it.opacity ?? 1) * (groupOp || 1);
    g.globalCompositeOperation = it.blend || 'normal';
    g.drawImage(it.canvas, 0, 0);
    g.restore();
    applyLayerMask(g, it);
  }
  function compositeInto(g, list, groupOp) {
    // track the most recent PLAIN layer's canvas for clipping (SAI: clip to the
    // layer directly BELOW, excluding the white paper background).
    let belowCanvas = null;
    for (const it of list) {
      if (!it.visible) continue;
      if (it.kind === 'group') {
        if (it.members && it.members.length) {
          g.save();
          // inside a group, the clip base is the group's own cumulative content
          compositeInto(g, it.members, (groupOp || 1) * (it.opacity ?? 1));
          g.restore();
        }
        continue;
      }
      if (it.clipping) {
        applyClipBelow(g, it, groupOp, belowCanvas);
        if (it.canvas) belowCanvas = it.canvas;   // still count it as content below subsequent ones? SAI: clip only the one directly below; update so chains work
        continue;
      }
      drawLayer(g, it, groupOp);
      belowCanvas = it.canvas;
    }
  }
  function applyClipBelow(g, it, groupOp, belowCanvas) {
    // Draw this layer clipped ONLY where the layer directly below has alpha.
    const layer = document.createElement('canvas'); layer.width = W; layer.height = H;
    const lg = layer.getContext('2d');
    lg.globalAlpha = (it.opacity ?? 1) * (groupOp || 1);
    lg.globalCompositeOperation = 'source-over';
    lg.drawImage(it.canvas, 0, 0);
    applyLayerMask(lg, it);
    if (belowCanvas) {
      lg.globalCompositeOperation = 'destination-in';
      lg.drawImage(belowCanvas, 0, 0);   // keep only where below layer has pixels
      applyLayerMask(lg, it);
    }
    g.save();
    g.globalCompositeOperation = it.blend || 'normal';
    g.drawImage(layer, 0, 0);
    g.restore();
  }
  function composite(g, layersArr) {
    g.clearRect(0, 0, W, H);
    const tmp = document.createElement('canvas'); tmp.width = W; tmp.height = H;
    const tg = tmp.getContext('2d');
    compositeInto(tg, layersArr, 1);
    g.drawImage(tmp, 0, 0);
  }
  function setTransform() {
    ctx.setTransform(view.zoom, 0, 0, view.zoom, view.panX, view.panY);
    composite(ctx, layers);
    ctx.imageSmoothingEnabled = false;
  }

  // ---- stroke pipeline (D3): pressure min-size, flow buildup, spacing, eraser ----
  const raw = [], smooth = [];
  let preview = null;   // overlay canvas holding the live stroke until commit

  // width from pressure: size at max pressure -> minSize at min pressure
  function widthForPressure(p) {
    return Math.max(brush.minSize, brush.size * (0.15 + 0.85 * p));
  }
  function pushRaw(pt) {
    raw.push(pt);
    if (raw.length > brush.stabilizer + 2) raw.shift();
    if (raw.length > brush.stabilizer) {
      const s = brush.stabilizer;
      const a = raw[raw.length - 1], b = raw[raw.length - 1 - s];
      const x = (a.x + b.x) / 2, y = (a.y + b.y) / 2;
      const dx = a.x - b.x, dy = a.y - b.y;
      const w = widthForPressure(a.p);
      smooth.push({ x, y, w: w, angle: Math.atan2(dy, dx) });
      if (smooth.length > 8) smooth.shift();
    }
  }
  // stamp one dab. D3: spacing (steps by distance/spacing) + flow additive buildup.
  // Repeated dabs on the same spot use 'lighter' so flow darkens (adds) up to the
  // stroke's effective opacity once committed.
  // ---- D7: symmetry ----
  window.App.symmetry = { on: false, axis: 'vertical' };   // vertical | horizontal | both
  window.App._overlays = [];                               // extra renderOverlay draw callbacks
  // mirror a homogenous point across the enabled symmetry axis
  function mirroredPts(x, y) {
    const s = window.App.symmetry;
    if (!s.on) return [{ x, y }];
    const pts = [{ x, y }];
    if (s.axis === 'vertical' || s.axis === 'both') pts.push({ x: W - x, y });
    if (s.axis === 'horizontal' || s.axis === 'both') pts.push({ x, y: H - y });
    return pts;
  }
  function stampSegmentTo(g, prev, cur, alpha, additive) {
    const sp = Math.max(0.08, brush.spacing || 0.35);
    const dist = Math.hypot(cur.x - prev.x, cur.y - prev.y);
    const steps = Math.max(1, Math.ceil(dist / (cur.w * sp)));
    const a = alpha != null ? alpha : (brushIsEraser() ? 1 : brush.flow);
    g.save();
    g.globalAlpha = a;
    const isErase = brushIsEraser();
    g.globalCompositeOperation = isErase ? 'destination-out' : (additive ? 'lighter' : 'source-over');
    const stCol = isErase ? '#000000' : brush.color;
    // stamp the segment AND any mirrored copies (symmetry)
    const segs = [{ a: prev, b: cur }];
    if (window.App.symmetry.on) {
      for (const p1 of mirroredPts(prev.x, prev.y)) {
        for (const p2 of mirroredPts(cur.x, cur.y)) {
          if (!(p1.x === prev.x && p1.y === prev.y) || !(p2.x === cur.x && p2.y === cur.y)) segs.push({ a: p1, b: p2 });
        }
      }
    }
    for (const sseg of segs) {
      for (let i = 1; i <= steps; i++) {
        const t = i / steps;
        const x = sseg.a.x + (sseg.b.x - sseg.a.x) * t;
        const y = sseg.a.y + (sseg.b.y - sseg.a.y) * t;
        const r = cur.w;
        g.drawImage(brushStamps.get(stCol, r, brush.hardness, brush.mode), x - r, y - r, r * 2, r * 2);
      }
    }
    g.restore();
  }
  function ensurePreview() {
    if (!preview) { preview = document.createElement('canvas'); preview.width = W; preview.height = H; }
    return preview.getContext('2d');
  }
  function clearPreview() { if (preview) { const g = preview.getContext('2d'); g.clearRect(0, 0, W, H); } }

  function renderOverlay() {
    // redraw plain composite (layers only)
    setTransform();
    // D7 overlays (perspective guide, etc.) drawn after composite
    if (window.App._overlays) { for (const fn of window.App._overlays) { try { fn(ctx); } catch (_) {} } }
    const pctx = ensurePreview();   // create preview if this is the first move
    pctx.save(); pctx.clearRect(0, 0, W, H);
    const isMask = window.App.maskPaintEnabled;
    if (isMask) {
      for (let i = 1; i < smooth.length; i++) stampSegmentToWithColor(pctx, smooth[i - 1], smooth[i], '#ffffff');
    } else if (brushIsEraser()) {
      for (let i = 1; i < smooth.length; i++) stampSegmentTo(pctx, smooth[i - 1], smooth[i], 1);
    } else {
      for (let i = 1; i < smooth.length; i++) stampSegmentTo(pctx, smooth[i - 1], smooth[i], brush.flow, true);
    }
    // LIVE RAW TRAIL: show the cursor path immediately (stabilizer may not have
    // produced a smooth point yet, so the trail must come from `raw` directly).
    // This is what makes mouse-pad drags feel like a real brush — the dot follows
    // your cursor instantly even before the stabilizer resolves.
    if (raw.length >= 2) {
      pctx.save();
      pctx.globalAlpha = isMask ? 1 : brush.opacity;
      pctx.globalCompositeOperation = isMask ? 'source-over' : (brushIsEraser() ? 'destination-out' : 'source-over');
      pctx.fillStyle = isMask ? '#ffffff' : (brushIsEraser() ? '#000000' : brush.color);
      const rw = Math.max(1, widthForPressure(raw[raw.length - 1].p));
      for (let i = 1; i < raw.length; i++) {
        const d = Math.hypot(raw[i].x - raw[i - 1].x, raw[i].y - raw[i - 1].y);
        const steps = Math.max(1, Math.ceil(d / (rw * 1.2)));
        for (let k = 1; k <= steps; k++) {
          const t = k / steps, x = raw[i - 1].x + (raw[i].x - raw[i - 1].x) * t, y = raw[i - 1].y + (raw[i].y - raw[i - 1].y) * t;
          pctx.beginPath(); pctx.arc(x, y, rw / 2, 0, Math.PI * 2); pctx.fill();
        }
      }
      pctx.restore();
    }
    pctx.restore();
    // draw preview through the view transform
    ctx.save(); ctx.setTransform(view.zoom, 0, 0, view.zoom, view.panX, view.panY);
    ctx.globalAlpha = 1;
    ctx.drawImage(preview, 0, 0);
    ctx.restore();
  }

  // ---- pointer interaction ----
  let drawing = false;
  function selHandler() { return window.App.selectionToolHandler; }
  function onPointerDown(e) {
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) { startPan(e); return; }
    // route selection tools to the selection module
    const sel = selHandler();
    if (window.App.tool !== 'brush' && sel) { sel.down(e); return; }
    const ly = drawTarget();
    if (!ly || ly.locked || !ly.visible) return;
    drawing = true; raw.length = 0; smooth.length = 0; strokeBaked = false;
    try { cvs.setPointerCapture(e.pointerId); } catch (_) {}
    const pt = viewToDoc(e);
    const pv = e.pressure && e.pressure > 0 ? e.pressure : 0.5;
    pushRaw({ x: pt.x, y: pt.y, p: pv });
    beginUndo();   // snapshot once so undo removes the whole stroke
  }
  function onPointerMove(e) {
    if (panning) { doPan(e); return; }
    const sel = selHandler();
    if (window.App.tool !== 'brush' && sel) { sel.move(e); return; }
    if (!drawing) return;
    const pt = viewToDoc(e);
    const pv = e.pressure && e.pressure > 0 ? e.pressure : 0.5;
    pushRaw({ x: pt.x, y: pt.y, p: pv });
    bakeLiveStrokeTransport();   // ink is permanent as you drag (real-paper)
    bakeRawTrail();              // and the cursor trail lands immediately too
    if (!panning) setTransform();
  }
  function onPointerUp(e) {
    if (window.App.tool !== 'brush') { const sel = selHandler(); if (sel) { sel.up(e); } return; }
    if (drawing) {
      drawing = false;
      finishStroke();   // already baked live during the drag; just clear state
    }
    if (panning) panning = false;
  }
  // snapshot the ACTIVE drawable BEFORE the stroke (layer is pristine because the
  // live stroke lives on `preview`), then bake the stroke onto the layer — or onto
  // the layer's MASK when mask-paint is enabled (black hides, white shows).
  function strokeTarget() {
    const ly = drawTarget();
    if (!ly) return null;
    return window.App.maskPaintEnabled && ly.mask ? { canvas: ly.mask, layer: ly } : { canvas: ly.canvas, layer: ly };
  }
  // ---- REAL-PAPER semantics: ink commits continuously as you drag ----
  // (undo snapshot taken once at stroke start so undo removes the whole stroke)
  let strokeBaked = false;
  function beginUndo() {
    const tgt = strokeTarget(); if (!tgt) return;
    const snap = document.createElement('canvas'); snap.width = W; snap.height = H;
    snap.getContext('2d').drawImage(tgt.canvas, 0, 0);
    undoStack.push(snap); if (undoStack.length > MAX_UNDO) undoStack.shift();
    redoStack.length = 0; strokeBaked = false;
  }
  // bake the newest portion of the stroke DIRECTLY onto the layer as you drag.
  // The smoothing trail follows the cursor; the ink stays where it lands.
  function bakeLiveStrokeTransport() {
    const tgt = strokeTarget();
    if (!tgt || tgt.layer.locked) return;
    const g = tgt.canvas.getContext('2d');
    const isMask = window.App.maskPaintEnabled && tgt.layer.mask;
    g.save();
    if (isMask) { g.globalCompositeOperation = 'destination-out'; g.globalAlpha = 1; }
    else {
      g.globalAlpha = brushIsEraser() ? 1 : brush.opacity;
      g.globalCompositeOperation = brushIsEraser() ? 'destination-out' : 'source-over';
    }
    let drew = false;
    if (isMask) {
      for (let i = 1; i < smooth.length; i++) { stampSegmentToWithColor(g, smooth[i - 1], smooth[i], '#ffffff'); drew = true; }
    } else {
      for (let i = 1; i < smooth.length; i++) { stampSegmentTo(g, smooth[i - 1], smooth[i], brush.flow); drew = true; }
    }
    g.restore();
    if (drew) { strokeBaked = true; lastStrokePoints = smooth.map(pt => ({ x: Math.round(pt.x), y: Math.round(pt.y), p: 0.5 })); }
  }
  // bake the RAW trail too (so the line follows the cursor even before the
  // stabilizer emits smooth points — mouse-pad sparse events)
  function bakeRawTrail() {
    const tgt = strokeTarget();
    if (!tgt || tgt.layer.locked || raw.length < 1) return;
    const g = tgt.canvas.getContext('2d'); const isMask = window.App.maskPaintEnabled && tgt.layer.mask;
    g.save();
    if (isMask) { g.globalCompositeOperation = 'destination-out'; g.globalAlpha = 1; }
    else { g.globalAlpha = brushIsEraser() ? 1 : brush.opacity; g.globalCompositeOperation = brushIsEraser() ? 'destination-out' : 'source-over'; }
    // convert raw points {x,y,p} -> {x,y,w} for the stamper
    const pts = raw.map(r => ({ x: r.x, y: r.y, w: widthForPressure(r.p) }));
    for (let i = 1; i < pts.length; i++) stampSegmentTo(g, pts[i - 1], pts[i], brush.flow);
    g.restore();
    if (pts.length >= 2) strokeBaked = true;
  }
  function finishStroke() {
    // undo snapshot already taken at beginUndo (pre-stroke), so undo removes
    // the whole stroke. State cleanup only — no re-bake (already on the layer).
    if (!strokeBaked) {
      // tap/dot with no movement: bake the single point so a tap leaves a dot
      const tgt = strokeTarget();
      if (tgt && !tgt.layer.locked && smooth.length >= 0 && raw.length >= 1) {
        const g = tgt.canvas.getContext('2d');
        g.save();
        if (window.App.maskPaintEnabled && tgt.layer.mask) { g.globalCompositeOperation = 'destination-out'; g.globalAlpha = 1; }
        else { g.globalAlpha = brushIsEraser() ? 1 : brush.opacity; g.globalCompositeOperation = brushIsEraser() ? 'destination-out' : 'source-over'; }
        if (window.App.maskPaintEnabled && tgt.layer.mask) stampSegmentToWithColor(g, raw[0], raw[0], '#ffffff');
        else stampSegmentTo(g, raw[0], raw[0], brush.flow);
        g.restore();
      }
    }
    smooth.length = 0; raw.length = 0;
    clearPreview();
    setTransform();
  }
  // compat: full-bake on demand (used by mask toggle / external callers)
  function commitStroke() { bakeLiveStrokeTransport(); finishStroke(); }
  function stampSegmentToWithColor(g, prev, cur, color) {
    const sp = Math.max(0.08, brush.spacing || 0.35);
    const dist = Math.hypot(cur.x - prev.x, cur.y - prev.y);
    const steps = Math.max(1, Math.ceil(dist / (cur.w * sp)));
    g.save(); g.globalAlpha = 1; g.globalCompositeOperation = 'source-over';
    for (let i = 1; i <= steps; i++) {
      const t = i / steps;
      const x = prev.x + (cur.x - prev.x) * t, y = prev.y + (cur.y - prev.y) * t, r = cur.w;
      g.drawImage(brushStamps.get(color, r, brush.hardness, brush.mode), x - r, y - r, r * 2, r * 2);
    }
    g.restore();
  }

  // ---- pan/zoom ----
  let panning = false, lastPan = { x: 0, y: 0 };
  function startPan(e) {
    panning = true; lastPan = { x: e.clientX, y: e.clientY };
    try { cvs.setPointerCapture(e.pointerId); } catch (_) {}
  }
  function doPan(e) {
    const r = cvs.getBoundingClientRect();
    view.panX += (e.clientX - lastPan.x) * (W / r.width);
    view.panY += (e.clientY - lastPan.y) * (H / r.height);
    lastPan = { x: e.clientX, y: e.clientY };
    setTransform();
  }
  function zoomAt(screenX, factor) {
    const r = cvs.getBoundingClientRect();
    const sx = (screenX - r.left) * (W / r.width);
    const docX = (sx - view.panX) / view.zoom;
    const newZ = clamp(view.zoom * factor, 0.01, 32);
    view.panX = sx - docX * newZ;
    view.zoom = newZ;
    setTransform();
  }
  function onWheel(e) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    zoomAt(e.clientX, factor);
  }
  function clamp(x, a, b) { return Math.max(a, Math.min(b, x)); }

  // ---- undo / redo (whole-stack snapshot for layer ops; layer snapshot for strokes) ----
  function stackSnap() {
    const c = document.createElement('canvas'); c.width = W; c.height = H;
    const g = c.getContext('2d');
    composite(g, layers.map(ly => ({ ...ly, canvas: ly.canvas })));
    return c;
  }
  function undo() {
    if (!undoStack.length) return;
    const tgt = strokeTarget(); if (!tgt) return;
    const snap = document.createElement('canvas'); snap.width = W; snap.height = H;
    snap.getContext('2d').drawImage(tgt.canvas, 0, 0);
    redoStack.push(snap);
    const prev = undoStack.pop();
    const g = tgt.canvas.getContext('2d');
    g.clearRect(0, 0, W, H); g.drawImage(prev, 0, 0);
    setTransform();
  }
  function redo() {
    if (!redoStack.length) return;
    const tgt = strokeTarget(); if (!tgt) return;
    const snap = document.createElement('canvas'); snap.width = W; snap.height = H;
    snap.getContext('2d').drawImage(tgt.canvas, 0, 0);
    undoStack.push(snap);
    const next = redoStack.pop();
    const g = tgt.canvas.getContext('2d');
    g.clearRect(0, 0, W, H); g.drawImage(next, 0, 0);
    setTransform();
  }

  // drawable target: normally layers[activeLayer]; if the active entry is a
  // group, the draw target is the LAST member (addLayer pushes into the group).
  let activeMember = null;   // {g, m} when drawing inside a group
  function activeIdx() { return Math.min(Math.max(activeLayer, 0), layers.length - 1); }
  function drawTarget() {
    const cur = layers[activeLayer];
    if (cur && cur.kind === 'group') {
      const m = cur.members[cur.members.length - 1];
      if (m) return m;
    }
    if (activeMember) {
      const g = layers[activeMember.g];
      if (g && g.kind === 'group' && g.members[activeMember.m]) return g.members[activeMember.m];
    }
    return layers[activeLayer];
  }
  function setActiveLayer(i) {
    activeLayer = Math.max(0, Math.min(i, layers.length - 1));
    activeMember = null;
    renderLayersUI();
  }
  function selectMember(i, mi) {
    const g = layers[i];
    if (g && g.kind === 'group' && g.members[mi]) {
      activeLayer = i;
      activeMember = { g: i, m: mi };
      renderLayersUI();
    }
  }
  function addLayer() {
    // if the active entry is a group, insert into its members; else above active
    const ai = activeIdx(); const cur = layers[ai];
    const ly = mkLayer('layer ' + (layers.length + 1), false);
    if (cur && cur.kind === 'group') {
      cur.members.push(ly);
      activeMember = { g: ai, m: cur.members.length - 1 };
    } else { layers.splice(ai + 1, 0, ly); activeLayer = ai + 1; activeMember = null; }
    renderLayersUI(); setTransform();
  }
  function addGroup() {
    const grp = { kind: 'group', name: 'group ' + (1 + layers.filter(x => x.kind === 'group').length), visible: true, open: true, opacity: 1, members: [] };
    layers.splice(activeIdx() + 1, 0, grp);
    activeLayer = activeIdx() + 1;
    renderLayersUI(); setTransform();
  }
  function deleteLayer() {
    const i = activeIdx();
    const it = layers[i];
    if (!it) return;
    if (it.kind === 'group') {
      if (layers.length - it.members.length <= 1) return;   // keep paper
      // remove members + group, keep paper first
      layers.splice(i, 1);
      activeLayer = Math.max(0, Math.min(i, layers.length - 1));
    } else {
      if (layers.length <= 1 || i === 0) return;
      layers.splice(i, 1);
      activeLayer = Math.max(0, Math.min(i, layers.length - 1));
    }
    renderLayersUI(); setTransform();
  }
  function duplicateLayer() {
    const i = activeIdx(); const src = layers[i];
    if (!src || src.kind === 'group') { addGroup(); return; }
    const c = document.createElement('canvas'); c.width = W; c.height = H;
    c.getContext('2d').drawImage(src.canvas, 0, 0);
    const ly = { ...src, id: nextId(), name: src.name + ' copy', canvas: c };
    layers.splice(i + 1, 0, ly);
    activeLayer = i + 1;
    renderLayersUI(); setTransform();
  }
  function moveLayer(dir) {
    const i = activeIdx(); const j = i + dir;
    if (j < 0 || j >= layers.length) return;
    if (layers[i].kind !== 'group' && layers[j].kind === 'group') return;  // don't cross into a group with a plain layer
    [layers[i], layers[j]] = [layers[j], layers[i]];
    activeLayer = j;
    renderLayersUI(); setTransform();
  }
  function mergeDown() {
    const i = activeIdx();
    if (i < 1 || layers[i].kind === 'group') return;
    const dst = layers[i - 1], src = layers[i];
    if (dst.kind === 'group') return;
    const g = dst.canvas.getContext('2d');
    g.save(); g.globalAlpha = src.opacity; g.globalCompositeOperation = src.blend; g.drawImage(src.canvas, 0, 0); g.restore();
    layers.splice(i, 1);
    activeLayer = i - 1;
    renderLayersUI(); setTransform();
  }
  function flattenVisible() {
    const merged = document.createElement('canvas'); merged.width = W; merged.height = H;
    const g = merged.getContext('2d');
    composite(g, layers);
    layers = [mkLayer('paper', false)];
    layers[0].canvas = merged;
    layers[0].name = 'flattened';
    const paper = layers[0];
    const out = merged;
    const final = document.createElement('canvas'); final.width = W; final.height = H;
    const fg = final.getContext('2d');
    fg.fillStyle = '#ffffff'; fg.fillRect(0, 0, W, H);
    fg.drawImage(out, 0, 0);
    paper.canvas = final;
    activeLayer = 0;
    renderLayersUI(); setTransform();
  }
  function toggleVisible(i) {
    const it = layers[i];
    it.visible = !it.visible;
    if (it.kind === 'group') { it.members.forEach(m => { m.visible = it.visible; }); }  // folder toggle cascades
    renderLayersUI(); setTransform();
  }
  function toggleLock(i) { layers[i].locked = !layers[i].locked; renderLayersUI(); }
  function setOpacity(i, v) { layers[i].opacity = Math.max(0, Math.min(1, Number(v) || 1)); renderLayersUI(); setTransform(); }
  function setBlend(i, b) { if (BLEND.includes(b)) { layers[i].blend = b; renderLayersUI(); setTransform(); } }
  function selectLayer(i) { setActiveLayer(i); }
  function toggleGroupOpen(i) { layers[i].open = !layers[i].open; renderLayersUI(); }

  // ---- layers UI ----
  function renderLayersUI() {
    const box = $('cvLayers');
    if (!box) return;
    box.innerHTML = '';
    layers.forEach((ly, i) => {
      if (ly.kind === 'group') {
        const row = document.createElement('div');
        row.className = 'cv-ly grp' + (i === activeIdx() ? ' on' : '') + (ly.visible ? '' : ' off');
        row.innerHTML =
          '<span class="cvly-arrow">' + (ly.open ? '▾' : '▸') + '</span>' +
          '<button class="cvly-eye" title="group visibility">' + (ly.visible ? '👁' : '—') + '</button>' +
          '<span class="cvly-name">' + esc(ly.name) + '<span class="hint"> (' + ly.members.length + ')</span></span>' +
          '<input type="range" class="cvly-op" min="0" max="1" step="0.05" value="' + ly.opacity + '" title="group opacity">';
        row.addEventListener('click', ev => {
          if (ev.target.closest('.cvly-arrow')) { toggleGroupOpen(i); return; }
          if (ev.target.closest('.cvly-eye')) { toggleVisible(i); return; }
          selectLayer(i);
        });
        row.querySelector('.cvly-op').addEventListener('input', ev => { ev.stopPropagation(); setOpacity(i, ev.target.value); });
        box.appendChild(row);
        if (ly.open) {
          ly.members.forEach((m, mi) => {
            const mrow = document.createElement('div');
            mrow.className = 'cv-ly member' + (mi === 0 && ly.members[mi] === layers[activeIdx()] ? '' : '');
            mrow.innerHTML =
              '<button class="cvly-eye" title="visibility">' + (m.visible ? '👁' : '—') + '</button>' +
              '<button class="cvly-lock" title="locked">' + (m.locked ? '🔒' : '🔓') + '</button>' +
              '<span class="cvly-name">↳ ' + esc(m.name) + '</span>' +
              '<input type="range" class="cvly-op" min="0" max="1" step="0.05" value="' + m.opacity + '" title="opacity">' +
              '<select class="cvly-blend">' + BLEND.map(b => '<option ' + (b === m.blend ? 'selected ' : '') + '>' + b + '</option>').join('') + '</select>';
            mrow.addEventListener('click', ev => {
              if (ev.target.closest('.cvly-eye')) { ev.stopPropagation(); toggleMemberVisible(i, mi); return; }
              if (ev.target.closest('.cvly-lock')) { ev.stopPropagation(); toggleMemberLock(i, mi); return; }
              selectMember(i, mi);
            });
            mrow.querySelector('.cvly-op').addEventListener('input', ev => { ev.stopPropagation(); setMemberOpacity(i, mi, ev.target.value); });
            mrow.querySelector('.cvly-blend').addEventListener('change', ev => { ev.stopPropagation(); setMemberBlend(i, mi, ev.target.value); });
            box.appendChild(mrow);
          });
        }
        return;
      }
      const row = document.createElement('div');
      row.className = 'cv-ly' + (i === activeIdx() ? ' on' : '') + (ly.locked ? ' lock' : '') + (ly.visible ? '' : ' off');
      row.innerHTML =
        '<button class="cvly-eye" title="visibility">' + (ly.visible ? '👁' : '—') + '</button>' +
        '<button class="cvly-lock" title="locked">' + (ly.locked ? '🔒' : '🔓') + '</button>' +
        '<span class="cvly-mk" title="' + (ly.mask ? 'mask' : '') + (ly.clipping ? ' / clip' : '') + '">' + (ly.mask ? (ly.clipping ? '▣' : '▤') : (ly.clipping ? '◉' : '')) + '</span>' +
        '<span class="cvly-name">' + esc(ly.name) + '</span>' +
        '<input type="range" class="cvly-op" min="0" max="1" step="0.05" value="' + ly.opacity + '" title="opacity">' +
        '<select class="cvly-blend">' + BLEND.map(b => '<option ' + (b === ly.blend ? 'selected ' : '') + '>' + b + '</option>').join('') + '</select>';
      row.addEventListener('click', ev => {
        if (ev.target.closest('.cvly-eye')) { toggleVisible(i); return; }
        if (ev.target.closest('.cvly-lock')) { toggleLock(i); return; }
        selectLayer(i);
      });
      row.querySelector('.cvly-op').addEventListener('input', ev => { ev.stopPropagation(); setOpacity(i, ev.target.value); });
      row.querySelector('.cvly-blend').addEventListener('change', ev => { ev.stopPropagation(); setBlend(i, ev.target.value); });
      box.appendChild(row);
    });
    $('cvActiveName').textContent = layers[activeIdx()] ? (layers[activeIdx()].kind === 'group' ? layers[activeIdx()].name : layers[activeIdx()].name) : '';
  }
  // group member accessors (they live in layers[i].members)
  function memberIdx(i, mi) { return layers[i] && layers[i].kind === 'group' ? layers[i].members[mi] : null; }
  function toggleMemberVisible(i, mi) { const m = memberIdx(i, mi); if (m) { m.visible = !m.visible; renderLayersUI(); setTransform(); } }
  function toggleMemberLock(i, mi) { const m = memberIdx(i, mi); if (m) { m.locked = !m.locked; renderLayersUI(); } }
  function setMemberOpacity(i, mi, v) { const m = memberIdx(i, mi); if (m) { m.opacity = Math.max(0, Math.min(1, Number(v) || 1)); renderLayersUI(); setTransform(); } }
  function setMemberBlend(i, mi, b) { const m = memberIdx(i, mi); if (m && BLEND.includes(b)) { m.blend = b; renderLayersUI(); setTransform(); } }

  // ---- brush presets UI ----
  function presetsList() { return BRUSH_PRESETS.concat(customPresets); }
  function renderPresets() {
    const sel = $('cvPresetSel'); if (!sel) return;
    const cur = sel.value;
    sel.innerHTML = presetsList().map(p => '<option ' + (p.name === cur ? 'selected ' : '') + '>' + esc(p.name) + '</option>').join('');
  }
  function syncBrushUI() {
    ['cvSize','cvMinSize','cvOp','cvFlow','cvHard','cvSpace'].forEach(id => { const el = $(id); if (el) el.value = brush[id === 'cvSize' ? 'size' : id === 'cvMinSize' ? 'minSize' : id === 'cvOp' ? 'opacity' : id === 'cvFlow' ? 'flow' : id === 'cvHard' ? 'hardness' : 'spacing']; });
    const m = $('cvModeSel'); if (m) m.value = brush.mode;
    if (brush.mode !== 'paint' && $('cvColor')) $('cvColor').style.opacity = 0.35;
    const sv = $('cvStab'); if (sv) sv.value = brush.stabilizer;
  }

  // ---- save to the engine (draw stroke --save, binds into brain) ----
  let lastStrokePoints = [];
  function saveStroke(points) {
    if (!points || !points.length) { $('cvStatus').textContent = 'draw something first'; return; }
    const pts = points.map(pt => Math.round(pt.x) + ',' + Math.round(pt.y) + ',' + '0.5').join(';');
    run(['draw', 'stroke', p(), '--canvas', String(currentCanvas), '--layer', String(currentLayer), '--brush', '1', '--color', brush.color.replace('#', ''), '--width', String(Math.max(1, Math.round(brush.size))), '--points', pts, '--save']);
    $('cvStatus').textContent = 'stroke → brain (draw stroke --save)';
  }

  // ---- two named cursors: brain cursor from snapshot ----
  function onState(d) {
    if (!d || !d.drawing) return;
    const b = d.drawing;
    const bx = b.cursorX, by = b.cursorY;
    const bc = document.getElementById('dBrainCur');
    if (bc && bx != null && by != null) {
      bc.textContent = '✛ ' + brainName() + ' @' + Math.round(bx) + ',' + Math.round(by);
      drawBrainCursor(bx, by);
    } else if (bc) bc.textContent = '✛ —';
  }
  function brainName() { return (p().split(/[\\\\/]/).pop() || 'brain').replace(/\.brain$/, ''); }
  function drawBrainCursor(x, y) {
    ctx.save(); ctx.setTransform(1, 0, 0, 1, 0, 0);
    const vp = docToView(x, y);
    ctx.strokeStyle = '#ff6b3d'; ctx.lineWidth = 1.5;
    ctx.beginPath(); ctx.moveTo(vp.x - 7, vp.y); ctx.lineTo(vp.x + 7, vp.y);
    ctx.moveTo(vp.x, vp.y - 7); ctx.lineTo(vp.x, vp.y + 7);
    ctx.stroke(); ctx.restore();
  }

  // ---- fit / 100% ----
  function fit() {
    const r = cvs.getBoundingClientRect();
    if (!r.width || !r.height) return;
    view.zoom = Math.min(r.width / W, r.height / H);
    view.panX = (W - W * view.zoom) / 2;
    view.panY = (H - H * view.zoom) / 2;
    setTransform();
  }
  function zoom100() { view.zoom = 1; view.panX = 0; view.panY = 0; setTransform(); }

  // ---- wiring ----
  cvs.addEventListener('pointerdown', onPointerDown);
  cvs.addEventListener('pointermove', onPointerMove);
  cvs.addEventListener('pointerup', onPointerUp);
  cvs.addEventListener('pointercancel', onPointerUp);
  cvs.addEventListener('wheel', onWheel, { passive: false });
  $('cvUndo').onclick = undo;
  $('cvRedo').onclick = redo;
  $('cvFit').onclick = fit;
  $('cv100').onclick = zoom100;
  $('cvColor').oninput = () => { brush.color = $('cvColor').value; };
  $('cvSize').oninput = () => { brush.size = Math.max(1, Number($('cvSize').value) || 3); $('cvSizeV').textContent = brush.size; };
  $('cvMinSize').oninput = () => { brush.minSize = Number($('cvMinSize').value) || 0; };
  $('cvOp').oninput = () => { brush.opacity = Number($('cvOp').value) || 1; };
  $('cvFlow').oninput = () => { brush.flow = Number($('cvFlow').value) || 1; };
  $('cvHard').oninput = () => { brush.hardness = Number($('cvHard').value) || 1; };
  $('cvSpace').oninput = () => { brush.spacing = Number($('cvSpace').value) || 0.35; };
  $('cvStab').oninput = () => { brush.stabilizer = Math.max(0, Math.min(7, Math.floor(Number($('cvStab').value) || 0))); $('cvStabV').textContent = 'S' + brush.stabilizer; };
  $('cvModeSel').onchange = () => { brush.mode = $('cvModeSel').value; if (brush.mode !== 'paint') $('cvColor').style.opacity = 0.35; else $('cvColor').style.opacity = 1; };
  $('cvPresetSel').onchange = () => { const all = presetsList(); const pr = all.find(x => x.name === $('cvPresetSel').value); if (pr) applyBrush(pr); };
  $('cvSaveCustom').onclick = () => {
    const name = (prompt('Preset name:') || '').trim();
    if (!name) return;
    const pr = { name, size: brush.size, minSize: brush.minSize, opacity: brush.opacity, flow: brush.flow, hardness: brush.hardness, spacing: brush.spacing, mode: brush.mode };
    customPresets = customPresets.filter(x => x.name !== name).concat([pr]);
    saveCustomPresets();
    renderPresets();
    syncBrushUI();
    $('cvStatus').textContent = 'custom preset saved: ' + name;
  };
  $('cvDeleteCustom').onclick = () => {
    const sel = $('cvPresetSel').value;
    const isCustom = customPresets.some(x => x.name === sel);
    if (isCustom && confirm('Delete custom preset "' + sel + '"?')) {
      customPresets = customPresets.filter(x => x.name !== sel);
      saveCustomPresets();
      renderPresets();
    }
  };
  $('cvSave').onclick = () => { const last = lastStrokePoints; if (last.length) saveStroke(last); };
  $('cvClear').onclick = () => { if (confirm('Clear the active layer?')) { const ly = drawTarget(); if (ly) { const g = ly.canvas.getContext('2d'); g.clearRect(0, 0, W, H); } redoStack.length = 0; undoStack.length = 0; setTransform(); } };
  $('cvAddLayer').onclick = addLayer;
  $('cvAddGroup').onclick = addGroup;
  $('cvDelLayer').onclick = deleteLayer;
  $('cvDupLayer').onclick = duplicateLayer;
  $('cvUpLayer').onclick = () => moveLayer(-1);
  $('cvDownLayer').onclick = () => moveLayer(1);
  $('cvMergeDown').onclick = mergeDown;
  $('cvFlatten').onclick = flattenVisible;
  $('cvAddMask').onclick = () => { window.App.addLayerMask(); window.App.maskPaintEnabled = true; $('cvStatus').textContent = 'mask added — brush now paints the mask (black hides)'; };
  $('cvClipToggle').onclick = () => { window.App.toggleLayerClip(); };

  window.App.onState = onState;

  // ---- D5: tool mode + selection integration ----
  window.App.tool = 'brush';      // brush | rect | ellipse | lasso | wand | fill | gradient | line | rect-shape | ellipse-shape
  window.App.getBrushSize = () => brush.size;
  // expose internals select.js needs
  window.App.activeDrawable = () => drawTarget();   // expose the draw target layer
  window.App.canvasInternals = {
    W, H, get view() { return view; },
    compositeInto,
    getLayers: () => layers,
    viewToDoc: (e) => viewToDoc(e),
    _drawPreview: renderOverlay,
    getCtx: () => ctx,
    viewTransform: (g) => g.setTransform(view.zoom, 0, 0, view.zoom, view.panX, view.panY),
  };
  // commit a transformed active layer into the canvas + undo stack
  window.App.transformCommit = (newLayerCanvas) => {
    const ly = drawTarget(); if (!ly) return false;
    const snap = document.createElement('canvas'); snap.width = W; snap.height = H;
    snap.getContext('2d').drawImage(ly.canvas, 0, 0);
    undoStack.push(snap); if (undoStack.length > MAX_UNDO) undoStack.shift();
    redoStack.length = 0;
    ly.canvas = newLayerCanvas;
    setTransform();
    return true;
  };

  // ---- D6: layer mask + clipping hooks ----
  window.App.addLayerMask = () => {
    const ly = drawTarget(); if (!ly || ly.kind === 'group') return;
    if (!ly.mask) {
      const m = document.createElement('canvas'); m.width = W; m.height = H;
      const g = m.getContext('2d'); g.fillStyle = '#ffffff'; g.fillRect(0, 0, W, H);   // white mask = fully visible
      ly.mask = m;
    }
    setTransform(); renderLayersUI();
  };
  window.App.toggleLayerClip = () => {
    const ly = drawTarget(); if (!ly || ly.kind === 'group') return;
    ly.clipping = !ly.clipping;
    setTransform(); renderLayersUI();
  };
  // paint onto the active layer's MASK (black hides, white shows). Used by the
  // mask-paint tool when it's enabled.
  window.App.maskPaintEnabled = false;
  window.App.clearLayerMask = () => {
    const ly = drawTarget(); if (!ly || !ly.mask) return;
    const g = ly.mask.getContext('2d'); g.clearRect(0, 0, W, H);
    setTransform();
  };
  // ---- test/verification hooks (deterministic - no pointer mapping) ----
  window.App.layerByName = (n) => layers.find(l => l.name === n);
  window.App.fillLayerRect = (ly, color, x0, y0, x1, y1) => {
    const g = ly.canvas.getContext('2d'); g.fillStyle = color; g.fillRect(x0, y0, x1 - x0, y1 - y0);
    setTransform();
  };
  window.App.countColorInComposite = (test) => {
    const scratch = document.createElement('canvas'); scratch.width = W; scratch.height = H;
    const g = scratch.getContext('2d'); composite(g, layers);
    const d = g.getImageData(0, 0, W, H).data;
    let n = 0; for (let i = 0; i < d.length; i += 4) { if (test(d[i], d[i + 1], d[i + 2], d[i + 3])) n++; }
    return n;
  };
  window.App.compositeAt = (x, y) => {
    const scratch = document.createElement('canvas'); scratch.width = W; scratch.height = H;
    const g = scratch.getContext('2d'); composite(g, layers);
    const d = g.getImageData(x, y, 1, 1).data; return [d[0], d[1], d[2], d[3]];
  };

  // ---- D4 color integration hooks ----
  window.App.setColor = (hex) => { brush.color = hex; const e = $('cvColor'); if (e) e.value = hex; };
  window.App.eyedropActive = false;
  // one-shot eyedropper: next canvas click reads the composite pixel
  cvs.addEventListener('pointerdown', function eyedropHandler(e) {
    if (!window.App.eyedropActive) return;
    const pt = viewToDoc(e);
    const hex = window.App.pickColorAt(pt.x, pt.y);
    window.App.eyedropActive = false;
    const ed = $('cvEyedrop'); if (ed) { ed.textContent = '🡇 pick'; ed.classList.remove('pri'); }
    $('cvStatus').textContent = 'picked ' + hex;
  });
  window.App.swapColors = () => {
    const fg = brush.color;
    brush.color = window.App.bgColor || '#ffffff';
    window.App.bgColor = fg;
    const e = $('cvColor'); if (e) e.value = brush.color;
  };
  // eyedrop: read a pixel from the COMPOSITE at doc coordinates
  window.App.pickColorAt = (docX, docY) => {
    const px = Math.round(docX), py = Math.round(docY);
    // composite to a scratch, read the pixel
    const scratch = document.createElement('canvas'); scratch.width = W; scratch.height = H;
    const g = scratch.getContext('2d');
    composite(g, layers);
    const d = g.getImageData(Math.max(0, Math.min(W - 1, px)), Math.max(0, Math.min(H - 1, py)), 1, 1).data;
    const hex = '#' + [d[0], d[1], d[2]].map(v => v.toString(16).padStart(2, '0')).join('');
    brush.color = hex;
    const e = $('cvColor'); if (e) e.value = hex;
    return hex;
  };

  // refit when the Drawing tab becomes visible
  document.addEventListener('click', (e) => {
    if (e.target && e.target.closest && e.target.closest('nav button[data-tab="drawing"]')) { setTimeout(fit, 0); }
  });
  fit();
  renderLayersUI();
  renderPresets();
  syncBrushUI();
  setInterval(() => { const s = window.App.getState; if (s) { const d = s(); if (d && d.drawing && d.drawing.cursorX != null) { drawBrainCursor(d.drawing.cursorX, d.drawing.cursorY); const bc = $('dBrainCur'); if (bc) bc.textContent = '✛ ' + brainName() + ' @' + Math.round(d.drawing.cursorX) + ',' + Math.round(d.drawing.cursorY); } } }, 3000);
})();
