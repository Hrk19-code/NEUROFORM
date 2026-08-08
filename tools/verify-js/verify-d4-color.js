// hermes-verify-d4-color.js — AD-HOC verification of D4 Color logic in
// tools/desktop/tabs/color.js + canvas.js (eyedrop). NOT a canonical suite
// (pure JS UI; Rust untouched). Mirrors the HSV<->hex math and swatch/pickback
// decisions. Rerun: node tools/verify-js/verify-d4-color.js
'use strict';
let pass = 0, fail = 0;
function ok(n,c,d){ if(c){pass++;console.log('  PASS '+n);} else {fail++;console.log('  FAIL '+n+' -> '+(d||''));} }

// ---- VERBATIM from color.js ----
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
function rgbToHex(r,g,b){ return '#' + [r,g,b].map(v => Math.round(v).toString(16).padStart(2,'0')).join(''); }

console.log('== 1. HSV -> hex known colors ==');
ok('red h=0 s=1 v=1', hsvToHex(0,1,1)==='#ff0000');
ok('green h=120', hsvToHex(120,1,1)==='#00ff00');
ok('blue h=240', hsvToHex(240,1,1)==='#0000ff');
ok('white s=0 v=1', hsvToHex(0,0,1)==='#ffffff');
ok('black v=0', hsvToHex(0,1,0)==='#000000');
ok('mid grey value', hsvToHex(0,0,0.5)==='#808080');

console.log('== 2. hex -> HSV round-trip ==');
const h1 = hexToHsv('#4fc3ff');
ok('parses hue in range', h1.h >= 0 && h1.h < 360);
ok('reconstructs close to original', hsvToHex(h1.h, h1.s, h1.v).toLowerCase() === '#4fc3ff');

console.log('== 3. hex validation ==');
ok('valid 6-digit', isHex('#4fc3ff')===true && isHex('4fc3ff')===true);
ok('invalid short', isHex('#fff')===false && isHex('nope')===false);

console.log('== 4. eyedrop -> hex from composite ==');
// a painted pixel rgb(255,0,0) must turn into #ff0000 via the pick conversion
ok('rgb->hex', rgbToHex(255,0,0)==='#ff0000');
ok('composite read maps to hex', rgbToHex(79,195,255).toLowerCase()==='#4fc3ff');

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
