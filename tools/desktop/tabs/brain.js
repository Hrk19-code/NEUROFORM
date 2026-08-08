// Brain tab — feed-the-brain controls. The live dashboard lives in app.js.
(function () {
  const { $, p, v, run, esc } = window.App;
  $('btnEv').onclick = () => run(['event', p(), '--text', v('evText'), '--valence', v('evVal'), '--arousal', v('evAro'), '--save']);
  $('btnExpose').onclick = () => run(['expose', p(), '--text', v('exText'), '--repeat', v('exRep'), '--save']);
  $('btnTick').onclick = () => run(['tick', p(), '--ticks', v('tkN'), '--save']);
  $('btnSleep').onclick = () => run(['sleep', p(), '--cycles', '1', '--save']);
  $('btnGrow').onclick = () => run(['grow', p(), '--save']);
  $('btnPhysics').onclick = () => run(['physics', p(), 'demo', '--save']);

  // ---------- Phase L1/L2: LLM endpoint manager ----------
  async function llmRefresh() {
    try {
      const r = await fetch('/api/llm/status');
      const j = await r.json();
      const sel = $('llmSel');
      sel.innerHTML = (j.profiles || []).map(pf => `<option value="${esc(pf.name)}">${esc(pf.name)} (${esc(pf.model)})</option>`).join('')
        || '<option value="">(none)</option>';
      sel.value = j.active || '';
      $('llmPill').textContent = j.active ? 'attached: ' + j.active : 'detached';
      $('llmPill').style.color = j.active ? 'var(--ok)' : '';
    } catch (e) { /* server restarting */ }
  }
  $('btnLlmSave').onclick = () => {
    const a = ['llm', 'save', '--name', v('llmName'), '--endpoint', v('llmEndpoint'), '--model', v('llmModel'), '--temperature', v('llmTemp'), '--active'];
    const k = $('llmKey').value.trim();
    if (k) a.push('--key', k);
    run(a);
    setTimeout(llmRefresh, 900);
  };
  $('btnLlmActive').onclick = () => { run(['llm', 'active', '--name', v('llmSel')]); setTimeout(llmRefresh, 900); };
  $('btnLlmTest').onclick = () => { run(['llm', 'test']); setTimeout(llmRefresh, 900); };
  $('btnLlmMock').onclick = () => { run(['llm', 'test', '--mock']); setTimeout(llmRefresh, 900); };
  llmRefresh();
  setInterval(llmRefresh, 15000);
  // Refresh the pill after every shell command too.
  window.App.onRunDone = llmRefresh;
})();
