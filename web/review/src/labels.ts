import type { Labels } from "./model/diff";

const catalogs = import.meta.glob<Labels>("./wasm/labels/*.json", { import: "default" });
const directory = "./wasm/labels/";
const pseudo = "en-XA";

// Mirrors `jayjay_i18n::set_preferred_languages`: same language and script, else en-US.
export function labelsLocale(languages: readonly string[], available: readonly string[]): string {
  if (languages[0] === pseudo && available.includes(pseudo)) return pseudo;
  for (const language of languages) {
    let requested: Intl.Locale;
    try {
      requested = new Intl.Locale(language).maximize();
    } catch {
      continue;
    }
    const match = available.find((tag) => {
      if (tag === pseudo) return false;
      const locale = new Intl.Locale(tag).maximize();
      return locale.language === requested.language && locale.script === requested.script;
    });
    if (match) return match;
  }
  return "en-US";
}

export function loadLabels(languages: readonly string[] = navigator.languages): Promise<Labels> {
  const available = Object.keys(catalogs).map((path) =>
    path.slice(directory.length, -".json".length),
  );
  const load = catalogs[`${directory}${labelsLocale(languages, available)}.json`];
  if (!load)
    return Promise.reject(new Error("UI labels are missing; run scripts/build-diff-web.sh"));
  return load();
}
