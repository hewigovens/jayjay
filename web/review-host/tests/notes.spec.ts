import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { bundle, ready } from "./support";

test("keeps the notes filter reachable after the last note is resolved", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Check this");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await ready(page);
  const filter = page.getByRole("button", { name: "With notes", exact: true });
  await filter.click();
  await page.getByRole("button", { name: "Resolve", exact: true }).click();
  await ready(page);
  await filter.click();
  await expect(page.getByRole("button", { name: "Cargo.toml", exact: true })).toBeVisible();
});

test("a refresh keeps a note draft unless its line is gone", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await bundle(page);
  await page.getByRole("button", { name: "Add note: After 1", exact: true }).click();
  const draft = page.getByRole("textbox", { name: "Write a review note", exact: true });
  await draft.fill("Keep this draft");
  await bundle(page);
  await expect(draft).toHaveValue("Keep this draft");
  await bundle(page, "changed\nfoo\nbar\n", "identity-2");
  await expect(draft).toHaveCount(0);
});

test("offers the whitespace toggle only after a note turns the mode on", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await bundle(page);
  const toggle = page.getByLabel("Ignore whitespace", { exact: true });
  await expect(toggle).toHaveCount(0);
  await page.getByRole("button", { name: "Add note: After 1", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Inspect the duplicate line");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await ready(page);
  await page.getByText("Local review data", { exact: true }).click();
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export review state", exact: true }).click();
  const state = JSON.parse(await readFile((await (await download).path())!, "utf8"));
  state.notes[0].ignore_whitespace = true;
  await page.getByLabel("Import review state", { exact: true }).setInputFiles({
    name: "review_store.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(state)),
  });
  await ready(page);
  await page.locator(".jj-notes-banner button").first().click();
  await ready(page);
  await toggle.click();
  await ready(page);
  await expect(toggle).toHaveCount(0);
});
