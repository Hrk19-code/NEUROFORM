// Drawing tab — D4: Color. HSV hue strip + SV square picker, hex input,
// swatches/palette (save/load/reset, default palette stored in localStorage),
// fg/bg swap (X), eyedropper that reads the composite. Bridges into the canvas
// via window.App.setColor / swapColors / pickColorAt.
(function () {
  const { $, esc } = window.App;

  const hueCv = $('cvHue'), svCv = $('cvSV');
  const hueG = hueCv.getContext('2d'), svG = svCv.getContext('2d');
  const W = svCv.width, H = svCv.height;

  let hue = 22;                 // H in [0,360]
  let sat = 0.62, val = 1;      // S,V in [0,1]
  let selectedSwatch = null;

  // ---- HSV <-> hex ----
  function hsvToHex(h, s, v) {
    const c = v * s, x = c * (1 - Math.abs(((h / 60) % 2) - 1)), m = v - c;
    let r = 0, g = 0, b = 0;
    if (h < 60) { r = c; g = x; } else if (h < 120) { r = x; g = c; } else if (h < 180) { g = c; b = x; }
    else if (h < 240) { g = x; b = c; } else if (h < 300) { r = x; b = c; } else { r = c; b = x; }
    const to = (q) => Math.round((q + m) * 255).toString(16).padStart(2, '0');
    return '#' + to(r) + to(g) + to(b);
  }
  function hexToHsv(hex) {
    const n = hex.replace('#', '');
    const r = parseInt(n.slice(0, 2), 16) / 255, g = parseInt(n.slice(2, 4), 16) / 255, b = parseInt(n.slice(4, 6), 16) / 255;
    const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
    let h = 0;
    if (d) {
      if (mx === r) h = ((g - b) / d) % 6;
      else if (mx === g) h = (b - r) / d + 2;
      else h = (r - g) / d + 4;
      h *= 60; if (h < 0) h += 360;
    }
    return { h, s: mx ? d / mx : 0, v: mx };
  }
  const isHex = (s) => /^#?[0-9a-fA-F]{6}$/.test(s.trim());

  // ---- rendering the pickers ----
  function paintHue() {
    for (let x = 0; x < hueCv.width; x++) {
      hueG.fillStyle = hsvToHex(x / hueCv.width * 360, 1, 1);
      hueG.fillRect(x, 0, 1, hueCv.height);
    }
    // cursor marker
    const cx = hue / 360 * hueCv.width;
    hueG.fillStyle = '#ffffff'; hueG.fillRect(cx - 2, 0, 4, hueCv.height);
  }
  function paintSV() {
    const base = hsvToHex(hue, 1, 1);
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const sv = hueToCss(x, y, base);
        svG.fillStyle = sv; svG.fillRect(x, y, 1, 1);
      }
    }
    const sx = sat * W, sy = (1 - val) * H;
    svG.strokeStyle = 'rgba(255,255,255,0.9)'; svG.lineWidth = 1.5;
    svG.strokeRect(sx - 3, sy - 3, 6, 6);
    svG.strokeStyle = 'rgba(0,0,0,0.6)'; svG.strokeRect(sx - 2, sy - 2, 4, 4);
    function hueToCss(x, y, base) {
      // interpolate white(low val) black(low sat) toward base
      const h = hue;
      const rBase = parseInt(base.slice(1, 3), 16), gBase = parseInt(base.slice(3, 5), 16), bBase = parseInt(base.slice(5, 7), 16);
      const s = x / W, v = 1 - y / H;
      const c = v * s, mm = v - c;
      // recompute full hsv rgb from h,s,v (same as hsvToHex but inline-able)
      const X = c * (1 - Math.abs(((h / 60) % 2) - 1));
      let r = 0, g = 0, b = 0;
      if (h < 60) { r = c; g = X; } else if (h < 120) { r = X; g = c; } else if (h < 180) { g = c; b = X; }
      else if (h < 240) { g = X; b = c; } else if (h < 300) { r = X; b = c; } else { r = c; b = X; }
      return 'rgb(' + ((r + mm) * 255 | 0) + ',' + ((g + mm) * 255 | 0) + ',' + ((b + mm) * 255 | 0) + ')';
    }
  }
  function currentHex() { return hsvToHex(hue, sat, val); }
  function render() {
    paintHue(); paintSV();
    const hex = currentHex();
    $('cvHex').value = hex;
    $('cvHex2').value = hex;
    $('cvFgv').textContent = hex;
    $('cvBgv').textContent = (window.App.bgColor || '#ffffff');
    if (window.App.setColor) window.App.setColor(hex);
  }

  // ---- picking ----
  hueCv.addEventListener('click', e => {
    const r = hueCv.getBoundingClientRect();
    hue = (e.clientX - r.left) / r.width * 360;
    render();
  });
  svCv.addEventListener('click', e => {
    const r = svCv.getBoundingClientRect();
    sat = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width));
    val = Math.max(0, Math.min(1, 1 - (e.clientY - r.top) / r.height));
    render();
  });

  // ---- hex input (two sync fields) ----
  function onHexInput() {
    const v = this.value.trim();
    if (isHex(v)) {
      const c = v.startsWith('#') ? v : '#' + v;
      const h = hexToHsv(c);
      hue = h.h; sat = h.s; val = h.v;
      render();
    }
  }
  $('cvHex').addEventListener('change', onHexInput);
  $('cvHex2').addEventListener('change', onHexInput);
  $('cvColor').addEventListener('input', () => {
    const v = $('cvColor').value;
    const h = hexToHsv(v); hue = h.h; sat = h.s; val = h.v;
    render();
  });

  // ---- swatches / palette ----
  const DEFAULT_PALETTE = ['#ff6633', '#4fc3ff', '#ff6b3d', '#5fe3a0', '#ffd34d', '#b07cff', '#ff7d7d', '#8ba5b8', '#ffffff', '#000000', '#22303e', '#e05a2f'];
  function loadPalette() { try { const p = JSON.parse(localStorage.getItem('nf-palette') || 'null'); return Array.isArray(p) && p.length ? p : DEFAULT_PALETTE.slice(); } catch (_) { return DEFAULT_PALETTE.slice(); } }
  function savePalette(p) { try { localStorage.setItem('nf-palette', JSON.stringify(p)); } catch (_) {} }
  let palette = loadPalette();
  function renderPalette() {
    const box = $('cvPalette');
    box.innerHTML = '';
    palette.forEach((c, i) => {
      const sw = document.createElement('div');
      sw.className = 'sw' + (selectedSwatch === i ? ' sel' : '');
      sw.style.background = c; sw.title = c;
      sw.onclick = () => { selectedSwatch = i; const h = hexToHsv(c); hue = h.h; sat = h.s; val = h.v; render(); renderPalette(); };
      box.appendChild(sw);
    });
  }
  $('cvSaveSwatch').onclick = () => { const c = currentHex(); if (!palette.includes(c)) { palette.push(c); savePalette(palette); renderPalette(); } };
  $('cvDelSwatch').onclick = () => { if (selectedSwatch != null && palette[selectedSwatch] != null) { palette.splice(selectedSwatch, 1); selectedSwatch = null; savePalette(palette); renderPalette(); } };
  $('cvResetPal').onclick = () => { palette = DEFAULT_PALETTE.slice(); selectedSwatch = null; savePalette(palette); renderPalette(); };

  // ---- fg/bg swap (X) ----
  $('cvSwap').onclick = () => { if (window.App.swapColors) window.App.swapColors(); const fg = (window.App.bgColor || '#ffffff'); const h = hexToHsv(fg); hue = h.h; sat = h.s; val = h.v; render(); };
  document.addEventListener('keydown', e => {
    if ((e.key === 'x' || e.key === 'X') && !e.target.closest('input,select,textarea')) $('cvSwap').onclick();
  });

  // ---- eyedropper (needs canvas coord mapping; wired in canvas.js) ----
  $('cvEyedrop').onclick = () => {
    $('cvEyedrop').textContent = '🡇 pick → click canvas';
    $('cvEyedrop').classList.add('pri');
    window.App.eyedropActive = true;
    // canvas.js sets a one-shot pointer listener to call pickColorAt
  };

  render(); renderPalette();
})();
