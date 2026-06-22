// Click any screenshot to open it full-size; click anywhere or press Esc to close.
// Shared by index.html and guide.html.
(function () {
  var css =
    '.show-img img, .shot img { cursor: zoom-in; }' +
    '.lightbox { position: fixed; inset: 0; z-index: 1000; display: none; align-items: center; justify-content: center; padding: 4vh 4vw; background: rgba(0,0,0,.82); cursor: zoom-out; }' +
    '.lightbox.open { display: flex; }' +
    '.lightbox img { max-width: 96vw; max-height: 92vh; border-radius: 12px; box-shadow: 0 24px 70px rgba(0,0,0,.55); }';
  var style = document.createElement('style');
  style.textContent = css;
  document.head.appendChild(style);

  var lb = document.createElement('div');
  lb.className = 'lightbox';
  lb.setAttribute('aria-hidden', 'true');
  var lbImg = document.createElement('img');
  lbImg.alt = 'Enlarged screenshot';
  lb.appendChild(lbImg);
  document.body.appendChild(lb);

  function close() { lb.classList.remove('open'); }
  document.querySelectorAll('.show-img img, .shot img').forEach(function (img) {
    img.addEventListener('click', function () {
      lbImg.src = img.currentSrc || img.src;
      lb.classList.add('open');
    });
  });
  lb.addEventListener('click', close);
  document.addEventListener('keydown', function (e) { if (e.key === 'Escape') close(); });
})();
