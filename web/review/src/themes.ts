import { diffThemes } from "./generated/diffThemes";

export { diffThemes };

export interface DiffThemeSelection {
  appearance?: "system" | "light" | "dark";
  light: string;
  dark: string;
}

export function resolveDiffTheme(id: unknown, dark: boolean) {
  return (
    diffThemes.find((theme) => theme.id === id && theme.dark === dark) ??
    diffThemes.find((theme) => theme.id === (dark ? "jayjay-dark" : "jayjay-light"))!
  );
}

export function diffThemeAttributes(themes?: DiffThemeSelection) {
  return themes
    ? {
        "data-jj-appearance": themes.appearance,
        "data-jj-theme-light": resolveDiffTheme(themes.light, false).id,
        "data-jj-theme-dark": resolveDiffTheme(themes.dark, true).id,
      }
    : {};
}
