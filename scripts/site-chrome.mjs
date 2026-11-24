// Shared head, nav and footer for every hand-written page under docs/; site.mjs expands them between <!-- site:head|nav|footer --> markers.
const github = 'https://github.com/hewigovens/jayjay';
const githubIcon = '<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"/></svg>';
const languageIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.5 2.5 3.8 5.5 3.8 9s-1.3 6.5-3.8 9c-2.5-2.5-3.8-5.5-3.8-9S9.5 5.5 12 3z"/></svg>';

// Where a page sits: `root` reaches docs/ from it, `home` prefixes landing-page anchors, `section` marks its nav link, and `styles` picks its stylesheets.
export function page(file) {
  const root = '../'.repeat(file.split('/').length - 1);
  const landing = file === 'index.html';
  const section = file === 'guide.html' ? 'guide' : file.startsWith('blog/') ? 'blog' : null;
  const styles = landing || file.startsWith('tutorial/') ? ['landing'] : ['help-common', 'guide', ...(section === 'blog' ? ['blog'] : [])];
  return { root, landing, section, styles, home: landing ? '' : root || './' };
}

export function head({ root, styles }) {
  // landing.css self-hosts its fonts; the guide stylesheets still load them from Google Fonts.
  const fonts = styles.includes('landing') ? [] : [
    '<link rel="preconnect" href="https://fonts.googleapis.com">',
    '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>',
    '<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">',
  ];
  return [
    '<meta name="viewport" content="width=device-width, initial-scale=1.0">',
    '<meta name="author" content="hewigovens">',
    '<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)">',
    '<meta name="theme-color" content="#0b0d14" media="(prefers-color-scheme: dark)">',
    `<link rel="icon" href="${root}favicon.ico" sizes="32x32">`,
    `<link rel="icon" href="${root}favicon.svg" type="image/svg+xml">`,
    `<link rel="apple-touch-icon" href="${root}apple-touch-icon.png">`,
    ...fonts,
    `<script src="${root}js/theme.js"></script>`,
    ...styles.map(name => `<link rel="stylesheet" href="${root}css/${name}.css">`),
  ].join('\n  ');
}

export function nav({ root, landing, section, home }) {
  const link = (href, label, name) => `<a href="${href}"${name === section ? ' class="active"' : ''}>${label}</a>`;
  const language = landing
    ? `
        <button class="lang-btn" popovertarget="lang-menu" aria-label="Language">${languageIcon}<span class="lang-now">EN</span></button>
        <div class="lang-menu" id="lang-menu" popover>
          <a href="./" lang="en" hreflang="en" aria-current="page">English</a>
          <a href="ja/" lang="ja" hreflang="ja">日本語</a>
          <a href="zh-hans/" lang="zh-Hans" hreflang="zh-Hans">简体中文</a>
          <a href="zh-hant/" lang="zh-Hant" hreflang="zh-Hant">繁體中文</a>
        </div>`
    : '';
  return `<nav class="site-nav">
    <div class="wrap nav">
      <a class="brand" href="${home || '#'}"><img src="${root}icon.svg" alt=""> JayJay</a>
      <div class="nav-right">
        <div class="nav-links">
          ${link(`${home}#features`, 'Features')}
          ${link(`${home}#shortcuts`, 'Shortcuts')}
          ${link(`${home}#install`, 'Install')}
          ${link(`${home}#faq`, 'FAQ')}
          ${link(`${root}blog/`, 'Blog', 'blog')}
          ${link(`${root}guide.html`, 'Guide', 'guide')}
          <a class="nav-gh" href="${github}">${githubIcon}GitHub</a>
        </div>${language}
        <button class="theme-btn" aria-label="Toggle theme"><span class="theme-icon"></span></button>
        <a class="nav-dl" href="${github}/releases/latest" data-install="${home}#install">Download</a>
      </div>
    </div>
  </nav>`;
}

export function footer({ root, landing }) {
  const links = [
    ...(landing ? [] : [[root || './', 'Home']]),
    [`${root}blog/`, 'Blog'],
    [`${root}guide.html`, 'Guide'],
    [`${root}tutorial/`, 'Video tutorial'],
    ['https://diff.hewig.dev', 'JayJay Diff'],
    [github, 'GitHub'],
    [`${github}/releases`, 'Releases'],
    [`${github}/issues`, 'Support'],
    ['https://discord.gg/ekknRNkVrT', 'Discord'],
    [`${root}privacy.html`, 'Privacy'],
  ];
  return `<footer>
    <div class="wrap foot">
      <p class="spons">Enjoy JayJay? <a href="${github}" aria-label="Star it on GitHub">★ it on GitHub</a> or <a href="https://github.com/sponsors/hewigovens">♥ Sponsor</a></p>
      <div class="foot-links">
${links.map(([href, label]) => `        <a href="${href}">${label}</a>`).join('\n')}
      </div>
    </div>
  </footer>
  <script src="${root}js/platform.js" defer></script>`;
}
