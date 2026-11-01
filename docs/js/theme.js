// Loaded synchronously in <head> so the saved theme is applied before CSS paints.
(function () {
  const storageKey = 'jayjay-theme';
  const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');
  const stroke = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"';
  const icons = {
    moon: `<svg ${stroke}><path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5z"/></svg>`,
    sun: `<svg ${stroke}><circle cx="12" cy="12" r="4"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4L7 17M17 7l1.4-1.4"/></svg>`,
  };

  function savedTheme() {
    return localStorage.getItem(storageKey);
  }

  function preferredDark() {
    const saved = savedTheme();
    return saved ? saved === 'dark' : systemTheme.matches;
  }

  function updateIcon(dark) {
    const icon = document.querySelector('.theme-icon');
    if (icon) icon.innerHTML = dark ? icons.moon : icons.sun;
  }

  function applyTheme(dark) {
    document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
    updateIcon(dark);
  }

  function toggleTheme() {
    const dark = document.documentElement.getAttribute('data-theme') !== 'dark';
    localStorage.setItem(storageKey, dark ? 'dark' : 'light');
    applyTheme(dark);
  }

  function connectThemeButton() {
    const button = document.querySelector('.theme-btn');
    if (button) button.addEventListener('click', toggleTheme);
    updateIcon(document.documentElement.getAttribute('data-theme') === 'dark');
  }

  applyTheme(preferredDark());
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', connectThemeButton, { once: true });
  } else {
    connectThemeButton();
  }

  systemTheme.addEventListener('change', function (event) {
    if (savedTheme()) return;
    applyTheme(event.matches);
  });
})();
