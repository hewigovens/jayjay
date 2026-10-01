// Plays the hero tour in a dialog, drawing the pointer from the take's click cues (video/examples/hero.json).
(function () {
  const steps = [
    [0.79, 0.141, 0.430, 'Select a change in the graph'],
    [2.07, 0.399, 0.185, 'Open its diff'],
    [4.34, 0.141, 0.213, 'Jump to the working copy'],
    [5.63, 0.399, 0.326, 'Open a file'],
    [7.41, 0.298, 0.326, 'Mark it reviewed'],
    [9.18, 0.141, 0.322, 'Back where we started'],
  ].map(([t, x, y, caption]) => ({ t, x, y, caption }));
  const clips = {
    light: { src: 'https://media.hewig.dev/tour-7416ccff.mp4', poster: 'imgs/tour.webp' },
    dark: { src: 'https://media.hewig.dev/tour-dark-283eba50.mp4', poster: 'imgs/tour-dark.webp' },
  };
  const rest = { x: 0.55, y: 0.6 };
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const move = reduced ? 0 : 0.5, lead = 0.08, scale = 1.35;

  const dialog = document.querySelector('.tour');
  const stage = dialog.querySelector('.tour-stage');
  const video = stage.querySelector('video');
  const cursor = stage.querySelector('.tour-cursor');
  const ripple = stage.querySelector('.tour-ripple');
  const caption = dialog.querySelector('.tour-cap');
  const zoomButton = dialog.querySelector('[data-zoom]');
  let zoom = !reduced;
  let shown = -1, clicked = -1, frame = 0;
  zoomButton.setAttribute('aria-pressed', zoom);

  const ease = p => (p < 0.5 ? 2 * p * p : 1 - Math.pow(-2 * p + 2, 2) / 2);
  const percent = v => v * 100 + '%';

  function place(el, x, y) {
    el.style.left = percent(x);
    el.style.top = percent(y);
  }

  function focus(step) {
    stage.style.transformOrigin = step ? percent(step.x) + ' ' + percent(step.y) : '50% 50%';
    stage.style.transform = zoom && step ? 'scale(' + scale + ')' : 'none';
  }

  function show(i) {
    shown = i;
    const count = document.createElement('span');
    count.className = 'n';
    count.textContent = i + 1 + '/' + steps.length;
    caption.replaceChildren(count, steps[i].caption);
    focus(steps[i]);
  }

  function render() {
    const t = video.currentTime;
    let from = rest;
    for (let i = 0; i < steps.length; i++) {
      const step = steps[i], start = step.t - lead - move;
      if (t < start) break;
      if (shown < i) show(i);
      if (t < step.t - lead) {
        const p = ease((t - start) / move);
        place(cursor, from.x + (step.x - from.x) * p, from.y + (step.y - from.y) * p);
        return;
      }
      if (i > clicked && t >= step.t && !reduced) {
        clicked = i;
        place(ripple, step.x, step.y);
        ripple.classList.remove('on');
        void ripple.offsetWidth;
        ripple.classList.add('on');
      }
      from = step;
    }
    place(cursor, from.x, from.y);
  }

  function loop() {
    render();
    if (!video.paused) frame = requestAnimationFrame(loop);
  }

  function play() {
    shown = clicked = -1;
    caption.textContent = '';
    focus(null);
    place(cursor, rest.x, rest.y);
    video.currentTime = 0;
    video.play().catch(() => {});
  }

  function open() {
    const clip = document.documentElement.dataset.theme === 'dark' ? clips.dark : clips.light;
    if (video.getAttribute('src') !== clip.src) {
      video.poster = clip.poster;
      video.src = clip.src;
    }
    dialog.showModal();
    play();
  }

  document.querySelectorAll('.hero-shot img, .play-btn').forEach(el => el.addEventListener('click', open));
  video.addEventListener('play', () => { cancelAnimationFrame(frame); loop(); });
  video.addEventListener('timeupdate', render);
  video.addEventListener('ended', () => focus(null));
  dialog.querySelector('[data-replay]').addEventListener('click', play);
  dialog.querySelector('[data-close]').addEventListener('click', () => dialog.close());
  dialog.addEventListener('click', e => { if (e.target === dialog) dialog.close(); });
  dialog.addEventListener('close', () => { video.pause(); cancelAnimationFrame(frame); });
  zoomButton.addEventListener('click', () => {
    zoom = !zoom;
    zoomButton.setAttribute('aria-pressed', zoom);
    focus(video.ended ? null : steps[shown]);
  });
})();
