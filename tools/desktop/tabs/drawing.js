// Drawing tab — legacy command wiring superseded by D1 canvas (tabs/canvas.js).
// Kept only for the motif/canvases inspector buttons; every lookup is guarded so
// a missing element never breaks the script chain.
(function () {
  const { $, p, run } = window.App;
  const btnDwNew = $('btnDwNew'), btnDwStroke = $('btnDwStroke'), btnDwMotifs = $('btnDwMotifs'), btnDwCanvases = $('btnDwCanvases');
  if (btnDwNew) btnDwNew.onclick = () => run(['draw', 'new', p(), '--name', ($('dwName') || { value: 'Sketch' }).value, '--w', '512', '--h', '512', '--save']);
  if (btnDwStroke) btnDwStroke.onclick = () => run(['draw', 'stroke', p(), '--canvas', '1', '--layer', '1', '--brush', '1', '--color', 'ff6633', '--width', '3', '--points', ($('dwPts') || { value: '10,10,0.5;30,20,0.8;60,15,0.4' }).value, '--save']);
  if (btnDwMotifs) btnDwMotifs.onclick = () => run(['draw', 'motifs', p()]);
  if (btnDwCanvases) btnDwCanvases.onclick = () => run(['draw', 'canvases', p()]);
})();
