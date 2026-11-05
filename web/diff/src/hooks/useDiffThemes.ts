import { useLayoutEffect, useState } from "react";
import { resolveDiffTheme } from "@jayjay/review-web/themes";

import {
  diffFontRange,
  interfaceFontRange,
  savedFontSize,
  type AppearanceSettings,
} from "../AppearanceSettings";

const storageKey = "jayjay-diff-themes";

function load(): AppearanceSettings {
  let saved: unknown;
  try {
    saved = JSON.parse(localStorage.getItem(storageKey) ?? "null");
  } catch {
    // Browsers may disable storage even when the page itself can run.
  }
  const value = saved && typeof saved === "object" ? saved : {};
  const appearance = "appearance" in value ? value.appearance : undefined;
  const fontSize = "fontSize" in value ? value.fontSize : undefined;
  return {
    fontSize: savedFontSize(fontSize, diffFontRange, 12),
    interfaceFontSize: savedFontSize(
      "interfaceFontSize" in value ? value.interfaceFontSize : undefined,
      interfaceFontRange,
      14,
    ),
    appearance: appearance === "light" || appearance === "dark" ? appearance : "system",
    light: resolveDiffTheme("light" in value ? value.light : undefined, false).id,
    dark: resolveDiffTheme("dark" in value ? value.dark : undefined, true).id,
  };
}

export function useDiffThemes() {
  const [themes, setThemes] = useState(load);
  useLayoutEffect(() => {
    document.documentElement.dataset.jjAppearance = themes.appearance ?? "system";
    return () => {
      delete document.documentElement.dataset.jjAppearance;
    };
  }, [themes.appearance]);
  useLayoutEffect(() => {
    document.documentElement.style.setProperty("--jj-diff-font-size", `${themes.fontSize}px`);
    return () => {
      document.documentElement.style.removeProperty("--jj-diff-font-size");
    };
  }, [themes.fontSize]);
  useLayoutEffect(() => {
    document.documentElement.style.setProperty(
      "--jj-interface-scale",
      String(themes.interfaceFontSize / 14),
    );
    return () => {
      document.documentElement.style.removeProperty("--jj-interface-scale");
    };
  }, [themes.interfaceFontSize]);
  const select = (next: AppearanceSettings) => {
    setThemes(next);
    try {
      localStorage.setItem(storageKey, JSON.stringify(next));
    } catch {
      // Keep the choice for this visit when storage is unavailable.
    }
  };
  return { themes, select };
}
