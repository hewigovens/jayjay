// Popovers render in the top layer, so plain CSS cannot place the language menu against its button.
(function () {
  const button = document.querySelector('.lang-btn');
  const menu = document.querySelector('.lang-menu');
  menu.addEventListener('beforetoggle', event => {
    if (event.newState !== 'open') return;
    const box = button.getBoundingClientRect();
    menu.style.top = box.bottom + 8 + 'px';
    const width = parseFloat(getComputedStyle(menu).width);
    const centered = box.left + box.width / 2 - width / 2;
    const widest = document.documentElement.clientWidth - width - 8;
    menu.style.left = Math.max(8, Math.min(centered, widest)) + 'px';
    menu.style.right = 'auto';
  });
})();
