// hermes-verify-w6-analysis.js — AD-HOC verification of W6 Analysis & Continuity
// logic in tools/desktop/tabs/analysis.js. NOT a canonical suite (pure JS UI
// feature; does not touch the Rust engine). Mirrors the pure functions verbatim
// (style analysis: sentence split, adverb/dialogue density, sentence histogram,
// readability, repeated-word flag) and the doc-ledger output parser, and
// asserts the W6 acceptance: "deliberately repetitive passage -> analysis flags
// it; contradict an entity property -> warning appears." KEPT in tools/verify-js/.
// Rerun: node tools/verify-js/verify-w6-analysis.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM from analysis.js: style functions ----
function sentences(text) {
  const t = (text || '').trim();
  if (!t) return [];
  return t.split(/(?<=[.!?])\s+/).map(s => s.trim()).filter(Boolean);
}
function wordsOf(text) { const m = (text || '').trim().match(/\S+/g); return m ? m.length : 0; }
function adverbDensity(text) { const ad = (text || '').match(/\b\w+ly\b/g) || []; return ad.length / (wordsOf(text) || 1); }
function dialogueRatio(text) { const dq = (text || '').match(/"([^"]*)"/g) || []; const dqW = dq.map(q => wordsOf(q)).reduce((a,b)=>a+b,0); return dqW / (wordsOf(text) || 1); }
function sentenceLenHistogram(text) {
  const ss = sentences(text).map(wordsOf).filter(n => n > 0); const hist = {};
  for (const n of ss) { const k = n <= 3 ? '≤3' : n <= 6 ? '4–6' : n <= 10 ? '7–10' : n <= 15 ? '11–15' : '16+'; hist[k] = (hist[k]||0)+1; }
  return { hist, avg: ss.length ? ss.reduce((a,b)=>a+b,0)/ss.length : 0, count: ss.length };
}
function wordFreq(text) { const f={}; (text||'').toLowerCase().split(/[^a-z']+/).filter(w=>w.length>2).forEach(w=>{f[w]=(f[w]||0)+1;}); return f; }
function repeatedWords(text, min=3) { return Object.entries(wordFreq(text)).filter(([,c])=>c>=min).sort((a,b)=>b[1]-a[1]).slice(0,12); }
function readability(text) {
  const ss = sentences(text); const nSent = Math.max(1, ss.length); const nWord = Math.max(1, wordsOf(text));
  const syll = (text||'').match(/[aeiouy]+/gi) ? (text||'').match(/[aeiouy]+/gi).length : 0;
  const score = 206.835 - 1.015*(nWord/nSent) - 84.6*(syll/nWord);
  return Math.max(0, Math.min(100, Math.round(score)));
}
// ---- VERBATIM from analysis.js: doc ledger parser ----
function parseLedger(out) {
  const header = /continuity ledger: (\d+) entities, (\d+) flags/.exec(out);
  const flags = []; const reFlag = /^\s*FLAG \[(property-conflict|timeline-conflict|.+?)\] (.+?) — (.+)$/gm; let m;
  while ((m = reFlag.exec(out)) !== null) flags.push({ kind: m[1], entity: m[2], detail: m[3] });
  return { header: header ? { entities:+header[1], flags:+header[2] } : null, flags };
}

const repetitive = 'The bridge stands over the river. The bridge is old. The bridge spans the river. The old bridge stands tall. The river flows under the bridge. The bridge is quiet. Success is never final.';
console.log('== 1. Style: repetitive passage flagged ==');
const rp = repeatedWords(repetitive);
ok('repetitive word flagged (top 13x-ish)', rp.some(([w,c]) => w==='bridge' && c>=4), JSON.stringify(rp));
ok('word cloud top contains bridge/river', rp.some(e=>e[0]==='bridge') && rp.some(e=>e[0]==='river'));

console.log('== 2. Style metrics ==');
const hs = sentenceLenHistogram(repetitive);
ok('sentence count = 7', hs.count === 7, 'got '+hs.count);
ok('avg sentence in sane range', hs.avg > 3 && hs.avg < 6, 'avg='+hs.avg);
ok('dialogue ratio 0 (no quotes)', dialogueRatio(repetitive) === 0);
ok('dialogue ratio > 0 when quotes present', dialogueRatio('"Go now," she said. "Leave the bridge behind."') > 0);
ok('adverb density > 0 for -ly', adverbDensity('he walked slowly and spoke softly') > 0);
ok('readability between 0-100', readability(repetitive) >= 0 && readability(repetitive) <= 100);

console.log('== 3. Continuity: ledger parse surfaces property-conflict ==');
const ledger = 'continuity ledger: 6 entities, 1 flags\n  FLAG [property-conflict] bridge — bridge described as both \'old\' (t=710360) and \'new\' (t=710670)\n  entity bridge: 7 mentions, first t=1, last t=2';
const pl = parseLedger(ledger);
ok('header parsed', pl.header && pl.header.entities===6 && pl.header.flags===1, JSON.stringify(pl.header));
ok('property-conflict flag surfaced', pl.flags.length===1 && pl.flags[0].kind==='property-conflict' && pl.flags[0].entity==='bridge', JSON.stringify(pl.flags));

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
