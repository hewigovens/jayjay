// Loaded synchronously in <head> so the saved theme is applied before CSS paints.
(function () {
  const storageKey = 'jayjay-theme';
  const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');

  function savedTheme() {
    return localStorage.getItem(storageKey);
  }

  function preferredDark() {
    const saved = savedTheme();
    return saved ? saved === 'dark' : systemTheme.matches;
  }

  function updateIcon(dark) {
    const icon = document.querySelector('.theme-icon');
    if (icon) icon.textContent = dark ? '☾' : '☀';
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
