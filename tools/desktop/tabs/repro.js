// Reproduction tab — union, relay, birth, growth (B phase grows the wizard).
(function () {
  const { $, p, v, run } = window.App;
  $('btnRpPropose').onclick = () => run(['net', p(), 'union-propose', '--session', v('rpSid'), '--save']);
  $('btnRpRelayProp').onclick = () => run(['net', p(), 'inject', '--session', v('rpSid'), '--type', 'union-proposal', '--from-file', v('rpFrom'), '--save']);
  $('btnRpRelayAcc').onclick = () => run(['net', p(), 'inject', '--session', v('rpSid'), '--type', 'union-accept', '--from-file', v('rpFrom'), '--save']);
  $('btnRpBirth').onclick = () => run(['net', p(), 'birth', '--session', v('rpSid'), '--out', v('rpOut'), '--force', '--save']);
  $('btnRpGrow').onclick = () => run(['grow', v('rpOut'), '--save']);
})();
