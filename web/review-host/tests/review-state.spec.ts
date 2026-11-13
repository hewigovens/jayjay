import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import { ready } from "./support";

test("loads syntax on demand, navigates files, and persists group marks and notes", async ({
  page,
}) => {
  const wasm: string[] = [];
  const errors: string[] = [];
  page.on("response", (response) => {
    if (response.url().endsWith(".wasm")) wasm.push(response.url());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await ready(page);
  expect(wasm.length).toBeGreaterThan(0);
  expect(wasm.every((url) => /jj_diff_(base|rust)_bg/.test(url))).toBe(true);
  await page.getByRole("button", { name: "Showing files as a tree", exact: true }).click();
  await expect(page.getByRole("button", { name: "src/greeting.rs", exact: true })).toHaveAttribute(
    "aria-current",
    "true",
  );
  await page.getByRole("button", { name: "Showing files as a flat list", exact: true }).click();
  await page.getByRole("checkbox", { name: /Change group 1/ }).click();
  await ready(page);
  await expect(page.getByRole("checkbox", { name: /Change group 1/ })).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
  ).toHaveJSProperty("indeterminate", true);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Check input handling. 日本語 <img src=x>");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-note.jj-current")).toContainText(
    "Check input handling. 日本語 <img src=x>",
  );
  await expect(page.locator(".jj-review-note img")).toHaveCount(0);
  await page.reload();
  await ready(page);
  await expect(page.getByRole("checkbox", { name: /Change group 1/ })).toBeChecked();
  await expect(page.locator(".jj-review-note.jj-current")).toContainText("Check input handling");
  await page.getByText("Local review data", { exact: true }).click();
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export review state", exact: true }).click();
  const state = JSON.parse(await readFile((await (await download).path())!, "utf8"));
  expect(state.notes[0].created_at_ms).toBeGreaterThan(Date.now() - 60_000);
  expect(state.notes[0].ignore_whitespace).toBe(false);
  expect(Object.keys(state.reviewed)).toContain("control-room-preview|src/greeting.rs");
  await page.getByLabel("Import review state", { exact: true }).setInputFiles({
    name: "review_store.json",
    mimeType: "application/json",
    buffer: Buffer.from('{"reviewed":{},"notes":[]}'),
  });
  await ready(page);
  await expect(page.locator(".jj-review-note")).toHaveCount(0);
  await page.getByRole("button", { name: "Undo import", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-note.jj-current")).toContainText("Check input handling");
  await page.getByRole("button", { name: "Resolve", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-note")).toHaveCount(0);
  await page.getByRole("button", { name: "Resolved: After 5", exact: true }).click();
  await expect(page.locator(".jj-review-note.jj-resolved")).toContainText("Check input handling");
  expect(errors).toEqual([]);
});

test("marking a file in the list keeps the open file", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const cargo = page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true });
  await cargo.click();
  await ready(page);
  await expect(cargo).toBeChecked();
  await expect(page.locator(".jj-file-heading h2")).toHaveText("src/greeting.rs");
});

test("rejects malformed UTF-8 and empty imports without replacing review data", async ({
  page,
}) => {
  await page.goto("/");
  await ready(page);
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  for (const buffer of [Buffer.from([0xff]), Buffer.alloc(0)])
    for (const label of ["Import change", "Import review state"]) {
      if (label === "Import review state")
        await page.getByText("Local review data", { exact: true }).click();
      await page
        .getByLabel(label, { exact: true })
        .setInputFiles({ name: "invalid.json", mimeType: "application/json", buffer });
      await expect(page.getByRole("alert")).toBeVisible();
      await expect(
        page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
      ).toBeChecked();
      await page.reload();
      await ready(page);
      await expect(
        page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
      ).toBeChecked();
    }
});

test("a folder's checkbox marks the files under it and follows their state", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const files = page.locator(".jj-file-column");
  const mark = (path: string) =>
    files.getByRole("checkbox", { name: `Mark reviewed: ${path}`, exact: true });
  await mark("src").click();
  await ready(page);
  await expect(mark("src")).toBeChecked();
  await expect(mark("src/greeting.rs")).toBeChecked();
  await expect(mark("src/web/session.ts")).toBeChecked();
  await expect(mark("config/settings.json")).not.toBeChecked();

  await mark("src/web/session.ts").click();
  await ready(page);
  await expect(mark("src")).toHaveJSProperty("indeterminate", true);
  await page.getByRole("button", { name: "Filter files", exact: true }).click();
  const filter = page.getByRole("searchbox", { name: "Filter files", exact: true });
  await filter.fill("greeting");
  await expect(mark("src")).toBeChecked();
  await filter.fill("");
  await mark("src").click();
  await ready(page);
  await expect(mark("src/web/session.ts")).toBeChecked();

  await mark("src").click();
  await ready(page);
  await expect(mark("src")).not.toBeChecked();
  await expect(mark("src/greeting.rs")).not.toBeChecked();
});
