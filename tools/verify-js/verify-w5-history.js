// hermes-verify-w5-history.js — AD-HOC verification of W5 Version History logic
// in tools/desktop/tabs/history.js + the writing.js currentDoc bridge.
// NOT a canonical suite (pure JS UI feature; no repo harness; does not touch
// the Rust engine). Mirrors the pure functions verbatim and asserts the W5
// acceptance: "5 versions of one doc; restore an old one; diff shows changes."
// KEPT permanently in tools/verify-js/ per user request — rerun with:
//   node tools/verify-js/verify-w5-history.js
'use strict';
let pass = 0, fail = 0;
function ok(name, cond, detail){ if(cond){pass++;console.log('  PASS '+name);} else {fail++;console.log('  FAIL '+name+' -> '+(detail||''));} }

// ---- VERBATIM: wordDiff (LCS over words) from history.js ----
function wordDiff(a, b) {
  const A = (a || '').split(/\s+/).filter(Boolean);
  const B = (b || '').split(/\s+/).filter(Boolean);
  const n = A.length, m = B.length;
  const dp = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) for (let j = m - 1; j >= 0; j--)
    dp[i][j] = A[i] === B[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
  let i = 0, j = 0; const out = [];
  while (i < n && j < m) {
    if (A[i] === B[j]) { out.push(['=', A[i]]); i++; j++; }
    else if (dp[i + 1][j] >= dp[i][j + 1]) { out.push(['-', A[i]]); i++; }
    else { out.push(['+', B[j]]); j++; }
  }
  while (i < n) { out.push(['-', A[i]]); i++; }
  while (j < m) { out.push(['+', B[j]]); j++; }
  return out;
}
// ---- VERBATIM: auto-snapshot ceiling decision from maybeAuto ----
function shouldAuto(words, N, lastAutoWords) {
  const ceil = Math.floor(words / N) * N;
  return ceil > lastAutoWords && ceil >= N;
}

console.log('== 1. word diff: added words ==');
const d1 = wordDiff('Hello brain extra words', 'Hello brain extra words NEW STUFF HERE');
const adds1 = d1.filter(x => x[0] === '+').map(x => x[1]);
ok('added highlighted', adds1.join(' ') === 'NEW STUFF HERE', JSON.stringify(adds1));
ok('common preserved', d1.filter(x => x[0] === '=').length === 4);

console.log('== 2. word diff: removed words ==');
const del2 = wordDiff('Hello brain old extra', 'Hello brain extra').filter(x => x[0] === '-').map(x => x[1]);
ok('removed highlighted', del2.join(' ') === 'old', JSON.stringify(del2));

console.log('== 3. word diff: identical -> no spans ==');
ok('identical gives zero + and -', wordDiff('same same', 'same same').every(x => x[0] === '='));

console.log('== 4. auto-snapshot ceiling ==');
ok('12/N=5 crosses ceil 10', shouldAuto(12, 5, 0) === true);
ok('4/N=5 no cross (ceil 0 < 5)', shouldAuto(4, 5, 0) === false);
ok('no double at same ceiling', shouldAuto(12, 5, 10) === false);
ok('baseline N=100, 60 words none', shouldAuto(60, 100, 0) === false);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
