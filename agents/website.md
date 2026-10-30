# Website Guide

Load this file when changing the public site at [jayjay.hewig.dev](https://jayjay.hewig.dev), whose GitHub Pages source is `docs/`. Feature PRs do not update shipped site content; refresh it once per [release](release.md) unless the task is specifically website work.

## Scope

- `docs/index.html` is the landing page and FAQ (`/#faq`). `docs/ja/`, `docs/zh-hans/`, and `docs/zh-hant/` are its generated translations; they share `docs/css/landing.css` and the scripts. The guide, blog, and privacy page stay English, and translated pages label links to them as English.
- `docs/guide.html` is the public workflow guide and user-doc source of truth; `UserGuide.md` only points to it.
- `docs/blog/` contains the blog index and posts.
- `docs/llms.txt` is the machine-readable project summary; `docs/sitemap.xml` and `docs/robots.txt` cover discovery.
- `docs/css/`, `docs/js/`, `docs/imgs/`, and the site icons are shared website assets.
- `docs/appcast.xml` is the Sparkle update feed, not website content. Load [Release](release.md) before changing it.

## Translated Landing Pages

`docs/index.html` is the template; never edit the generated pages. `site/<locale>.ftl` holds the translations, kept apart from the app catalog because `crates/jayjay-i18n` compiles every message into the app. The comment above each message is its English source, so `just site` fails on any string left missing or unused after an English edit. Update those files, then run `just site`; `just site --check` is the CI gate in Site CI. Write copy the way a native speaker would rather than mirroring the English sentence by sentence; keep the facts, commands, and UI labels exact. Take terms from `crates/jayjay-i18n/locales/` so the page matches the app. A new page section with a new markup shape needs a matching pass in `scripts/site.mjs`. Keep each page's `hreflang` link and sitemap `<loc>`.

## Relationship To The Help Book

The website and the embedded macOS Help Book are separate surfaces. The public guide and screenshots are shared release inputs, but the Help Book has its own topic HTML, bundle metadata, build, and system-Help verification. Load [Help Book](help-book.md) only for that embedded bundle.

## Style And Verification

Load [Design](design.md) before changing site copy or presentation. Keep the static pages accessible, responsive, consistent across landing, guide, and blog chrome, and usable in light and dark modes. Check changed pages and links locally; publication is handled by the Pages workflow for `docs/**` changes.
