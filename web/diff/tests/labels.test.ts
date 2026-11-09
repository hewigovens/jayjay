import { expect, test } from "vitest";
import { labelsLocale } from "@jayjay/review-web/labels";

test("picks the catalog of the reader's first language with a matching script", () => {
  const available = ["en-XA", "en-US", "ja", "zh-Hans", "zh-Hant"];
  for (const [languages, locale] of [
    [["ja-JP", "en-US"], "ja"],
    [["zh-TW"], "zh-Hant"],
    [["zh-HK"], "zh-Hant"],
    [["zh-CN"], "zh-Hans"],
    [["zh"], "zh-Hans"],
    [["fr-FR", "zh-Hant-TW"], "zh-Hant"],
    [["fr-FR", "de"], "en-US"],
    [["en-GB"], "en-US"],
    [["en-XA"], "en-XA"],
    [["not a tag", "ja"], "ja"],
    [[], "en-US"],
  ] as const)
    expect(labelsLocale(languages, available), languages.join()).toBe(locale);
});
