import type { DiffThemeSelection } from "@jayjay/review-web/themes";

export interface AppearanceSettings extends DiffThemeSelection {
  fontSize: number;
  interfaceFontSize: number;
}

export const diffFontRange = { min: 10, max: 24 };
export const interfaceFontRange = { min: 12, max: 20 };

export function savedFontSize(
  value: unknown,
  range: { min: number; max: number },
  fallback: number,
) {
  return typeof value === "number" &&
    Number.isInteger(value) &&
    value >= range.min &&
    value <= range.max
    ? value
    : fallback;
}
