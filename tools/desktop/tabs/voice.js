// Voice tab — speak command (V phase grows prosody view + face here).
(function () {
  const { $, p, v, run } = window.App;
  $('btnSpeak').onclick = () => run(['voice', p(), 'speak', '--text', v('spText'), '--save']);
})();
