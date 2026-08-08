// Drawing tab — D6: Fill, Gradient. Bucket fill (tolerance + contiguous) over
// the composite, and a gradient tool (linear or radial, fg->bg) applied to the
// active layer. Both commit via App.transformCommit (undo-aware) so fill is
// reversible. Bridges through the existing selectionToolHandler (tools 'fill',
// 'gradient').
(function () {
  const { $, esc } = window.App;
  const ci = () => window.App.canvasInternals;
  const W = 512, H = 512;
  const DEFAULT_OB = () => window.App.gradientShape || 'linear';

  function activeLayer() { return window.App.activeDrawable ? window.App.activeDrawable() : null; }
  function commitToLayer(newCanvas) {
    if (window.App.transformCommit) window.App.transformCommit(newCanvas);
  }
  // read composite pixels array
  function compositeData() {
    const scratch = document.createElement('canvas'); scratch.width = W; scratch.height = H;
    const g = scratch.getContext('2d');
    ci().compositeInto(g, ci().getLayers());
    return g.getImageData(0, 0, W, H).data;
  }

  // ---- bucket fill ----
  function bucketFill(x, y, tolerance, contiguous) {
    const ly = activeLayer(); if (!ly || ly.kind === 'group') { $('cvStatus').textContent = 'pick a paint layer first'; return; }
    if (ly.locked) { $('cvStatus').textContent = 'layer locked'; return; }
    const id = compositeData();
    const base = [id[(y * W + x) * 4], id[(y * W + x) * 4 + 1], id[(y * W + x) * 4 + 2]]; // composite color (ignores alpha since paper is white)
    const T = tolerance | 0;
    const fg = (window.App.getColor ? window.App.getColor() : '#ff6633') || '#ff6633';
    // build a mask of fillable pixels (flood vs global) then stamp fg there on the layer
    const fillMask = new Uint8Array(W * H);
    if (contiguous) {
      const seen = new Uint8Array(W * H);
      const stack = [[x | 0, y | 0]];
      while (stack.length) {
        const p = stack.pop(), px = p[0], py = p[1];
        if (px < 0 || py < 0 || px >= W || py >= H || seen[py * W + px]) continue;
        const o = (py * W + px) * 4; seen[py * W + px] = 1;
        const dist = Math.max(0, Math.abs(id[o] - base[0]) + Math.abs(id[o + 1] - base[1]) + Math.abs(id[o + 2] - base[2])) / 3;
        if (dist > T) continue;
        fillMask[py * W + px] = 1;
        stack.push([px + 1, py], [px - 1, py], [px, py + 1], [px, py - 1]);
      }
    } else {
      for (let i = 0; i < W * H; i++) { const o = i * 4; const dist = Math.max(0, Math.abs(id[o] - base[0]) + Math.abs(id[o + 1] - base[1]) + Math.abs(id[o + 2] - base[2])) / 3; fillMask[i] = dist <= T ? 1 : 0; }
    }
    const any = fillMask.some(v => v);
    if (!any) { $('cvStatus').textContent = 'no fillable area at tolerance'; return; }
    // apply fill onto the active layer (source-over where fillMask=1)
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    const g = out.getContext('2d');
    g.drawImage(ly.canvas, 0, 0);
    g.save(); g.globalCompositeOperation = 'source-over';
    // per-pixel stamp: cheap approach = build a mask-image and drawImage with it as clip
    const maskImg = document.createElement('canvas'); maskImg.width = W; maskImg.height = H;
    const mg = maskImg.getContext('2d');
    const mData = mg.createImageData(W, H);
    for (let i = 0; i < W * H; i++) if (fillMask[i]) mData.data[i * 4 + 3] = 255;
    mg.putImageData(mData, 0, 0);
    g.globalCompositeOperation = 'source-in';   // keep only fillable region of layer
    g.drawImage(ly.canvas, 0, 0);
    g.globalCompositeOperation = 'source-over';
    // draw the fill color limited to fillMask by using the mask as a clip via temp
    g.globalAlpha = 1; g.fillStyle = fg; g.fillRect(0, 0, W, H);   // would paint all; re-strict:
    g.globalCompositeOperation = 'destination-in';
    g.drawImage(maskImg, 0, 0);
    g.restore();
    commitToLayer(out);
    $('cvStatus').textContent = 'bucket fill (' + any + ' px)';
  }

  // ---- gradient tool ----
  let gradStart = null, gradEnd = null;
  function applyGradient(shape) {
    const ly = activeLayer(); if (!ly || ly.kind === 'group') { $('cvStatus').textContent = 'pick a paint layer first'; return; }
    if (ly.locked) { $('cvStatus').textContent = 'layer locked'; return; }
    if (!gradStart || !gradEnd) { $('cvStatus').textContent = 'drag to define the gradient'; return; }
    const { x: x1, y: y1 } = gradStart, { x: x2, y: y2 } = gradEnd;
    const len = Math.max(1, Math.hypot(x2 - x1, y2 - y1));
    const fg = (window.App.getColor ? window.App.getColor() : '#ff6633') || '#ff6633';
    const bg = window.App.bgColor || '#ffffff';
    const out = document.createElement('canvas'); out.width = W; out.height = H;
    const g = out.getContext('2d');
    g.drawImage(ly.canvas, 0, 0);
    // Build the gradient into a temp canvas and draw it over the layer.
    const grad = document.createElement('canvas'); grad.width = W; grad.height = H;
    const gg = grad.getContext('2d');
    const gr = shape === 'radial'
      ? gg.createRadialGradient(x1, y1, 0, x1, y1, len)
      : gg.createLinearGradient(x1, y1, x2, y2);
    gr.addColorStop(0, fg); gr.addColorStop(1, bg);
    gg.fillStyle = gr; gg.fillRect(0, 0, W, H);
    g.drawImage(grad, 0, 0);
    commitToLayer(out);
    $('cvStatus').textContent = shape + ' gradient';
  }

  // ---- wire into selection tool routing ----
  window.App.gradientShape = 'linear';
  window.App.getColor = () => { try { return $('#cvColor').value; } catch (_) { return '#ff6633'; } };

  const prev = window.App.selectionToolHandler || {};
  window.App.selectionToolHandler = {
    down(e) {
      if (window.App.tool === 'fill') {
        const pt = ci().viewToDoc(e);
        bucketFill(Math.round(pt.x), Math.round(pt.y), $('#cvTolerance').value, $('#cvContig').checked);
        return;
      }
      if (window.App.tool === 'gradient') {
        gradStart = ci().viewToDoc(e); gradEnd = null;
        return;
      }
      if (prev && prev.down) return prev.down(e);
    },
    move(e) {
      if (window.App.tool === 'gradient' && gradStart) { gradEnd = ci().viewToDoc(e); previewGradient(); return; }
      if (prev && prev.move) return prev.move(e);
    },
    up(e) {
      if (window.App.tool === 'gradient' && gradStart) { gradEnd = ci().viewToDoc(e); applyGradient(window.App.gradientShape); gradStart = gradEnd = null; return; }
      if (prev && prev.up) return prev.up(e);
    },
  };
  function previewGradient() {
    if (!gradStart || !gradEnd) return;
    const cix = ci();
    const g = cix.getCtx(); g.save(); cix.viewTransform(g);
    g.strokeStyle = 'rgba(255,255,255,0.9)';
    g.beginPath(); g.moveTo(gradStart.x, gradStart.y); g.lineTo(gradEnd.x, gradEnd.y); g.stroke();
    g.restore();
  }
})();
