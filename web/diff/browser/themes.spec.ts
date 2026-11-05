import { expect, test, type Page } from "@playwright/test";
import { openPull } from "./pullFixture";

async function pick(page: Page, label: string, choice: string) {
  await page.getByRole("button", { name: new RegExp(`^${label} `) }).click();
  await page
    .getByRole("menu", { name: label, exact: true })
    .getByRole("menuitemradio", { name: choice, exact: true })
    .click();
}

for (const width of [390, 1280]) {
  test(`diff themes follow appearance and survive reload at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 850 });
    await page.emulateMedia({ colorScheme: "light" });
    await openPull(page);
    const diff = page.locator(".jj-diff-lines").first();
    const keyword = diff.locator(".jj-token-Keyword").first();
    await expect(keyword).toBeVisible();
    const original = await diff.evaluate((element) => getComputedStyle(element).backgroundColor);
    const syntax = await keyword.evaluate((element) => getComputedStyle(element).color);
    const header = page.locator(".jj-change-header");
    const chrome = await header.evaluate((element) => getComputedStyle(element).backgroundColor);
    const mounted = await diff.elementHandle();
    const toggle = page
      .locator(".pull-app-actions")
      .getByRole("button", { name: "Appearance", exact: true });
    await toggle.click();
    await pick(page, "Light theme", "Solarized Light");
    await pick(page, "Dark theme", "Nord");
    const sizeStepper = page.getByRole("group", { name: "Diff", exact: true });
    const interfaceStepper = page.getByRole("group", { name: "Interface", exact: true });
    const increase = sizeStepper.getByRole("button", { name: "Increase font size" });
    for (let i = 0; i < 6; i++) await increase.click();
    await expect(diff).toHaveCSS("font-size", "18px");
    await expect(page.locator(".jj-line-number").first()).toHaveCSS("font-size", "16.5px");
    const originalInterface = await header.evaluate((element) =>
      parseFloat(getComputedStyle(element).fontSize),
    );
    await interfaceStepper.getByRole("button", { name: "Increase font size" }).click();
    await interfaceStepper.getByRole("button", { name: "Increase font size" }).click();
    await expect(interfaceStepper).toContainText("16 px");
    expect(
      await header.evaluate((element) => parseFloat(getComputedStyle(element).fontSize)),
    ).toBeGreaterThan(originalInterface);
    await expect(diff).toHaveCSS("font-size", "18px");
    const themePicker = page.getByRole("button", { name: "Dark theme Nord", exact: true });
    await themePicker.click();
    await page.keyboard.press("Escape");
    await expect(themePicker).toBeFocused();
    await expect(page.locator(".diff-theme-picker")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(toggle).toBeFocused();
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth - innerWidth),
    ).toBeLessThanOrEqual(0);
    const light = await diff.evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(light).not.toBe(original);
    expect(await keyword.evaluate((element) => getComputedStyle(element).color)).not.toBe(syntax);
    expect(await header.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe(
      chrome,
    );
    expect(await mounted!.evaluate((element) => element.isConnected)).toBe(true);

    await page.emulateMedia({ colorScheme: "dark" });
    const dark = await diff.evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(dark).not.toBe(light);
    await page.reload();
    await expect(diff).toBeVisible();
    await expect(diff).toHaveCSS("background-color", dark);
    await toggle.click();
    await expect(
      page.getByRole("button", { name: "Light theme Solarized Light", exact: true }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Dark theme Nord", exact: true })).toBeVisible();
    await expect(diff).toHaveCSS("font-size", "18px");
    await expect(interfaceStepper).toContainText("16 px");
    const decrease = sizeStepper.getByRole("button", { name: "Decrease font size" });
    await decrease.focus();
    await page.keyboard.press("Enter");
    await expect(diff).toHaveCSS("font-size", "17px");
    for (let i = 0; i < 7; i++) await decrease.click();
    await expect(diff).toHaveCSS("font-size", "10px");
    await expect(decrease).toBeDisabled();
    for (let i = 0; i < 14; i++) await increase.click();
    await expect(increase).toBeDisabled();
    await expect(diff).toHaveCSS("font-size", "24px");
    await page.getByRole("button", { name: "Close", exact: true }).click();
    await expect(toggle).toBeFocused();
    await page.emulateMedia({ colorScheme: "light" });
    await expect(diff).toHaveCSS("background-color", light);

    await toggle.click();
    await page.getByRole("radio", { name: "Dark", exact: true }).check();
    await expect(diff).toHaveCSS("background-color", dark);
    await expect(page.locator(".diff-theme-picker")).toHaveCSS("color-scheme", "dark");
    await page.reload();
    await expect(diff).toHaveCSS("background-color", dark);
    await toggle.click();
    const darkMode = page.getByRole("radio", { name: "Dark", exact: true });
    await expect(darkMode).toBeChecked();
    await darkMode.focus();
    await page.keyboard.press("ArrowLeft");
    await expect(page.getByRole("radio", { name: "Light", exact: true })).toBeChecked();
    await page.emulateMedia({ colorScheme: "dark" });
    await expect(diff).toHaveCSS("background-color", light);
    await expect(header).toHaveCSS("background-color", chrome);
    await page.keyboard.press("ArrowLeft");
    await expect(page.getByRole("radio", { name: "System", exact: true })).toBeChecked();
    await expect(diff).toHaveCSS("background-color", dark);
    await page.getByRole("button", { name: "Close", exact: true }).click();
    await expect(toggle).toBeFocused();
  });
}

test("invalid saved themes fall back and disabled storage still allows selection", async ({
  page,
}) => {
  await page.addInitScript(() => {
    Storage.prototype.getItem = () =>
      '{"light":"nord","dark":"missing-theme","fontSize":999,"interfaceFontSize":-1}';
    Storage.prototype.setItem = () => {
      throw new DOMException("Storage disabled", "SecurityError");
    };
  });
  await openPull(page);
  await page
    .locator(".pull-app-actions")
    .getByRole("button", { name: "Appearance", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Light theme Default Light", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Dark theme Default Dark", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("group", { name: "Diff", exact: true })).toContainText("12 px");
  await expect(page.getByRole("group", { name: "Interface", exact: true })).toContainText("14 px");
  await pick(page, "Dark theme", "Dracula");
  const dark = page.getByRole("button", { name: "Dark theme Dracula", exact: true });
  await expect(dark).toBeVisible();
  await page.getByRole("button", { name: "Close", exact: true }).click();
  await page
    .locator(".pull-app-actions")
    .getByRole("button", { name: "Appearance", exact: true })
    .click();
  await expect(dark).toBeVisible();
});
