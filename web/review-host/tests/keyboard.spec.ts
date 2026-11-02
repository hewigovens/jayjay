import { expect, test } from "@playwright/test";
import { ready } from "./support";

test("Space marks the focused file and moves to the next unreviewed one", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "src/greeting.rs", exact: true }).click();
  await ready(page);
  await page.keyboard.press("Space");
  await ready(page);
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
  ).toBeChecked();
  const next = page.locator('.jj-file-select[aria-current="true"]');
  await expect(next).not.toHaveAccessibleName("src/greeting.rs");
  await expect(next).toBeFocused();
  await page.getByRole("button", { name: "src/greeting.rs", exact: true }).click();
  await ready(page);
  await page.keyboard.press("Space");
  await ready(page);
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
  ).not.toBeChecked();
  await expect(page.getByRole("button", { name: "src/greeting.rs", exact: true })).toHaveAttribute(
    "aria-current",
    "true",
  );
});

test("J and K move through the file list, and N skips reviewed files", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const rows = page.locator(".jj-file-rows .jj-file-select");
  const current = page.locator('.jj-file-select[aria-current="true"]');
  await rows.nth(0).click();
  await ready(page);
  await page.keyboard.press("j");
  await ready(page);
  await expect(current).toHaveAccessibleName((await rows.nth(1).getAttribute("aria-label")) ?? "");
  await expect(current).toBeFocused();
  await page.keyboard.press("Space");
  await ready(page);
  await page.keyboard.press("k");
  await page.keyboard.press("k");
  await ready(page);
  await expect(current).toHaveAccessibleName((await rows.nth(0).getAttribute("aria-label")) ?? "");
  await page.keyboard.press("n");
  await ready(page);
  await expect(current).toHaveAccessibleName((await rows.nth(2).getAttribute("aria-label")) ?? "");
  await page.keyboard.press("?");
  await expect(page.getByRole("dialog", { name: "Keyboard shortcuts" })).toBeVisible();
});

test("marking the open file jumps forward, wraps, and keeps a note draft", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const open = page.locator('.jj-file-select[aria-current="true"]');
  const names = await page
    .locator(".jj-file-select")
    .evaluateAll((rows) => rows.map((row) => row.getAttribute("aria-label")!));
  await page.getByRole("button", { name: names[1], exact: true }).click();
  await ready(page);
  await page.keyboard.press("Space");
  await ready(page);
  await expect(open).toHaveAccessibleName(names[2]);
  await page.getByRole("button", { name: names[names.length - 1], exact: true }).click();
  await ready(page);
  await page.keyboard.press("Space");
  await ready(page);
  await expect(open).toHaveAccessibleName(names[0]);
  await page
    .getByRole("button", { name: /^Add note: After/ })
    .first()
    .click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Keep this draft");
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  await expect(open).toHaveAccessibleName(names[0]);
  await expect(page.getByRole("textbox", { name: "Write a review note", exact: true })).toHaveValue(
    "Keep this draft",
  );
});

test("Space keeps focus in the list while reviewed files are hidden", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true }).click();
  await ready(page);
  await page.getByRole("button", { name: "Hide reviewed", exact: false }).click();
  const unselected = page.locator(".jj-file-select:not([aria-current])").first();
  await unselected.focus();
  await page.keyboard.press("Space");
  await ready(page);
  await expect(page.locator(".jj-file-select:focus")).toHaveCount(1);
});
