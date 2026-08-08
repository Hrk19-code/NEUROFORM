// Body tab — sensory embodiment commands.
(function () {
  const { $, p, v, run } = window.App;
  $('btnTouch').onclick = () => run(['body', p(), 'touch', '--pressure', v('bdP'), '--velocity', v('bdV'), '--duration', '1200', '--save']);
  $('btnMotion').onclick = () => run(['body', p(), 'motion', '--linear', v('bdM'), '--rotational', '0.2,0,0', '--save']);
  $('btnIntero').onclick = () => run(['body', p(), 'interocept', '--energy-load', v('bdI'), '--save']);
  $('btnBodyStatus').onclick = () => run(['body', p(), 'status']);
})();
