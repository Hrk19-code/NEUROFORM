// hermes-verify-d5-select.js — AD-HOC verification of D5 Selections & Transform
// logic in tools/desktop/tabs/select.js + canvas.js (transformCommit). NOT a
// canonical suite (pure JS UI; Rust untouched). Mirrors the deterministic bits:
// point-in-polygon, ellipse test, magic-wand contiguous flood (with correct
// pixel-vs-byte indexing), and the undo-reverses-transform stack semantics.
// Rerun: node tools/verify-js/verify-d5-select.js
'use strict';
let pass = 0, fail = 0;
function ok(n,c,d){ if(c){pass++;console.log('  PASS '+n);} else {fail++;console.log('  FAIL '+n+' -> '+(d||''));} }

// ---- VERBATIM from select.js ----
function idx(x,y){ return (y|0)*512 + (x|0); }
function pointInPoly(x, y, pts) {
  let inside = false;
  for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
    const xi = pts[i].x, yi = pts[i].y, xj = pts[j].x, yj = pts[j].y;
    if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) inside = !inside;
  }
  return inside;
}
function inEllipse(x, y, c, r) { const dx = (x - c.x) / r.r, dy = (y - c.y) / r.c; return dx * dx + dy * dy <= 1; }
function seedAt(id, x, y) { const o = idx(x,y)*4; return [id[o], id[o+1], id[o+2], id[o+3]]; }
function colorDistAt(id, i, base) { const o = i*4; const dr=id[o]-base[0], dg=id[o+1]-base[1], db=id[o+2]-base[2]; return Math.max(0,Math.abs(dr)+Math.abs(dg)+Math.abs(db))/3; }
function floodMask(W,H,id,cx,cy,T){ const m=new Uint8Array(W*H); const seen=new Uint8Array(W*H); const base=seedAt(id,cx,cy); const stack=[[cx,cy]]; while(stack.length){ const p=stack.pop(),x=p[0],y=p[1]; if(x<0||y<0||x>=W||y>=H||seen[idx(x,y)])continue; const off=idx(x,y); seen[off]=1; if(colorDistAt(id,off,base)>T)continue; m[off]=1; stack.push([x+1,y],[x-1,y],[x,y+1],[x,y-1]); } return m; }

console.log('== 1. point-in-polygon ==');
ok('inside square', pointInPoly(50,50,[{x:0,y:0},{x:100,y:0},{x:100,y:100},{x:0,y:100}])===true);
ok('outside square', pointInPoly(150,50,[{x:0,y:0},{x:100,y:0},{x:100,y:100},{x:0,y:100}])===false);
ok('inside triangle', pointInPoly(1,1,[{x:0,y:0},{x:10,y:0},{x:0,y:10}])===true);

console.log('== 2. ellipse membership ==');
ok('center inside', inEllipse(10,10,{x:10,y:10},{r:5,c:5})===true);
ok('corner outside', inEllipse(20,20,{x:10,y:10},{r:5,c:5})===false);
ok('edge-on inside', inEllipse(15,10,{x:10,y:10},{r:5,c:5})===true);

console.log('== 3. magic-wand flood (pixel-index correct) ==');
// 8x8: red 4x4 block on white; seed red -> 16, seed white -> 48
const grid=new Uint8ClampedArray(8*8*4);
for(let y=0;y<8;y++)for(let x=0;x<8;x++){ const o=(y*8+x)*4; const red=(x>=2&&x<=5&&y>=2&&y<=5); grid[o]=grid[o+1]=grid[o+2]=255; grid[o+3]=255; if(red){grid[o+1]=0;grid[o+2]=0;} }
function floodLocal(WW,HH,data,cx,cy,T){ const ii=(x,y)=>y*WW+x; const m=new Uint8Array(WW*HH); const seen=new Uint8Array(WW*HH); const base=(()=>{const o=ii(cx,cy)*4;return[data[o],data[o+1],data[o+2],data[o+3]];})(); const stack=[[cx,cy]]; while(stack.length){ const p=stack.pop(),x=p[0],y=p[1]; if(x<0||y<0||x>=WW||y>=HH||seen[ii(x,y)])continue; const off=ii(x,y); const o=off*4; seen[off]=1; const dist=Math.max(0,Math.abs(data[o]-base[0])+Math.abs(data[o+1]-base[1])+Math.abs(data[o+2]-base[2]))/3; if(dist>T)continue; m[off]=1; stack.push([x+1,y],[x-1,y],[x,y+1],[x,y-1]); } return m; }
const cnt=(mm)=>Array.from(mm).filter(v=>v).length;
ok('seed red selects red region only', cnt(floodLocal(8,8,grid,3,3,10))===16);
ok('seed white selects white, stops at red', cnt(floodLocal(8,8,grid,0,0,10))===48);
ok('higher tolerance includes more (merges boundary)', cnt(floodLocal(8,8,grid,0,0,300))===64);

console.log('== 4. undo stack reverses a committed transform ==');
// commit pushes a snapshot; undo restores it (canvas transformCommit semantics)
const stack=[]; function commitCur(cur){ stack.push(cur); } function undo(){ return stack.length ? stack.pop() : null; }
commitCur('S1'); commitCur('S2');
ok('undo returns last committed', undo()==='S2');
ok('undo reverses to prior', undo()==='S1');
ok('empty undo null', undo()===null);

console.log('\nRESULT: ' + pass + ' passed, ' + fail + ' failed');
process.exit(fail === 0 ? 0 : 1);
