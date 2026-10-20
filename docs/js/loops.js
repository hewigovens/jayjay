// Plays a showcase loop only while it is on screen and in the current theme, drawing the pointer from the take's click cues and the step captions on its clock; under reduced motion the poster stands in.
(function () {
  if (matchMedia('(prefers-reduced-motion: reduce)').matches || !('IntersectionObserver' in window)) return;
  const move = 0.5, lead = 0.08;
  const ease = p => (p < 0.5 ? 2 * p * p : 1 - Math.pow(-2 * p + 2, 2) / 2);

  function place(el, x, y) {
    el.style.left = x * 100 + '%';
    el.style.top = y * 100 + '%';
  }

  function caption(box) {
    const el = document.createElement('p');
    el.className = 'loop-cap';
    box.append(el);
    const steps = box.dataset.captions.split('|').map(step => {
      const at = step.indexOf(' ');
      return { t: Number(step.slice(0, at)), text: step.slice(at + 1) };
    });
    let shown = -1;
    return t => {
      let i = 0;
      while (i + 1 < steps.length && t >= steps[i + 1].t) i++;
      if (i === shown) return;
      shown = i;
      const count = document.createElement('span');
      count.className = 'n';
      count.textContent = i + 1 + '/' + steps.length;
      el.replaceChildren(count, steps[i].text);
    };
  }

  function attach(video, cursor, ripple, showCaption) {
    const steps = video.dataset.cues.split(',').map(cue => {
      const [t, x, y] = cue.trim().split(' ').map(Number);
      return { t, x, y };
    });
    let clicked = -1, last = 0, frame = 0;

    function render() {
      const t = video.currentTime;
      showCaption(t);
      if (t < last) clicked = -1;
      last = t;
      let from = steps[steps.length - 1];
      for (let i = 0; i < steps.length; i++) {
        const step = steps[i], start = step.t - lead - move;
        if (t < start) break;
        if (t < step.t - lead) {
          const p = ease((t - start) / move);
          place(cursor, from.x + (step.x - from.x) * p, from.y + (step.y - from.y) * p);
          return;
        }
        if (i > clicked && t >= step.t) {
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
      frame = requestAnimationFrame(loop);
    }

    video.addEventListener('playing', () => {
      cursor.style.visibility = 'visible';
      cancelAnimationFrame(frame);
      loop();
    });
    video.addEventListener('pause', () => cancelAnimationFrame(frame));
  }

  const visible = new IntersectionObserver(entries => {
    for (const { target, isIntersecting } of entries) {
      if (isIntersecting) target.play().catch(() => {});
      else target.pause();
    }
  }, { threshold: 0.4 });

  document.querySelectorAll('.show-img[data-captions]').forEach(box => {
    const stage = box.querySelector('.loop-stage');
    stage.insertAdjacentHTML('beforeend', '<span class="tour-ripple"></span><svg class="tour-cursor" viewBox="0 0 14 22" aria-hidden="true" style="visibility: hidden"><path d="M1 1v17.5l4.4-4.2 2.9 6.6 2.7-1.2-2.9-6.5H14z" fill="#000" stroke="#fff" stroke-width="1.3" stroke-linejoin="round"/></svg>');
    const ripple = stage.querySelector('.tour-ripple'), cursor = stage.querySelector('.tour-cursor');
    const showCaption = caption(box);
    showCaption(0);
    stage.querySelectorAll('video.loop').forEach(video => {
      attach(video, cursor, ripple, showCaption);
      visible.observe(video);
    });
  });
})();
