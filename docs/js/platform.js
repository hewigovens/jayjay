// Points the nav and hero buttons at the visitor's OS and, on the landing page, turns the install panels into an ARIA tab set; without scripts the pages offer macOS first and show every panel.
(function () {
  const platform = (navigator.userAgentData?.platform || navigator.platform || navigator.userAgent).toLowerCase();
  const os = /^win|windows/.test(platform) ? 'windows' : /linux|x11/.test(platform) && !/android|cros/i.test(navigator.userAgent) ? 'linux' : 'macos';
  for (const button of document.querySelectorAll('.cta [data-os]')) button.hidden = button.dataset.os !== os;
  const download = document.querySelector('.nav-dl');
  if (os !== 'macos') download.href = download.dataset.install;

  const list = document.querySelector('.os-tabs');
  if (!list) return;
  const tabs = [...list.querySelectorAll('[role="tab"]')];
  const select = (tab, focus) => {
    for (const other of tabs) {
      const selected = other === tab;
      other.setAttribute('aria-selected', selected);
      other.tabIndex = selected ? 0 : -1;
      document.getElementById(other.getAttribute('aria-controls')).hidden = !selected;
    }
    if (focus) tab.focus();
  };
  tabs.forEach((tab, index) => {
    tab.addEventListener('click', () => select(tab));
    tab.addEventListener('keydown', event => {
      const last = tabs.length - 1;
      const next = { ArrowLeft: index ? index - 1 : last, ArrowRight: index === last ? 0 : index + 1, Home: 0, End: last }[event.key];
      if (next === undefined) return;
      event.preventDefault();
      select(tabs[next], true);
    });
  });
  for (const button of document.querySelectorAll('.cta [data-os]')) {
    const tab = tabs.find(tab => tab.dataset.os === button.dataset.os);
    if (tab) button.addEventListener('click', () => select(tab));
  }
  select(tabs.find(tab => tab.dataset.os === os));
  list.hidden = false;
})();
