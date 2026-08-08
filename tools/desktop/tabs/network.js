// Network tab — inter-brain sessions and relationships.
(function () {
  const { $, p, v, run } = window.App;
  $('btnNetKey').onclick = () => run(['net', p(), 'key']);
  $('btnNetPair').onclick = () => run(['net', p(), 'pair', '--peer', v('ntPeer'), '--peer-key', v('ntKey'), '--save']);
  $('btnNetStatus').onclick = () => run(['net', p(), 'status']);
  $('btnNetDisc').onclick = () => run(['net', p(), 'discover', '--on', '--save']);
  $('btnNetEst').onclick = () => run(['net', p(), 'establish', '--session', v('ntSid'), '--save']);
  $('btnNetSend').onclick = () => run(['net', p(), 'send', '--session', v('ntSid'), '--text', v('ntMsg'), '--save']);
  $('btnNetInject').onclick = () => run(['net', p(), 'inject', '--session', v('ntSid'), '--text', v('ntMsg'), '--save']);
})();
