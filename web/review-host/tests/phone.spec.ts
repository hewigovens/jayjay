import { expect, test, type Page } from "@playwright/test";
import { bundle, ready } from "./support";

test.describe("phone portrait", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test("a reviewed file folds until it is unmarked or a note opens it", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    await bundle(page);
    const mark = page.getByRole("checkbox", { name: "Mark reviewed: a.txt", exact: true });
    const lines = page.locator(".jj-diff-cell");
    await page.getByRole("button", { name: "Add note: After 1", exact: true }).click();
    await page
      .getByRole("textbox", { name: "Write a review note", exact: true })
      .fill("Inspect the duplicate line");
    await mark.tap();
    await ready(page);
    await expect(
      page.getByRole("textbox", { name: "Write a review note", exact: true }),
    ).toHaveValue("Inspect the duplicate line");
    await page.getByRole("button", { name: "Save note", exact: true }).click();
    await ready(page);
    await expect(lines).toHaveCount(0);
    await page.locator(".jj-notes-banner button").first().click();
    await ready(page);
    await expect(lines.first()).toBeVisible();
    await mark.tap();
    await ready(page);
    await expect(lines.first()).toBeVisible();
    await page.getByRole("checkbox", { name: /Change group 1/ }).tap();
    await ready(page);
    await expect(lines).toHaveCount(0);
  });

  test("lists every file's diff in one scroll with its own mark", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    await expect(page.locator(".jj-file-column")).toBeHidden();
    await expect(page.locator(".jj-file-section")).toHaveCount(5);
    const toggle = page.getByRole("button", { name: "Expand description", exact: true });
    await toggle.click();
    await expect(
      page.getByRole("button", { name: "Collapse description", exact: true }),
    ).toHaveAttribute("aria-expanded", "true");
    const cargo = page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true });
    await cargo.scrollIntoViewIfNeeded();
    await expect(
      page.locator('.jj-diff-scroll[data-path="Cargo.toml"] .jj-diff-cell').first(),
    ).toBeVisible();
    await cargo.tap();
    await ready(page);
    await expect(cargo).toBeChecked();
    await expect(
      page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }),
    ).not.toBeChecked();
    await page.reload();
    await ready(page);
    await expect(
      page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true }),
    ).toBeChecked();
  });

  test("importing review state refreshes every loaded file's marks", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    const cargo = page.locator(".jj-file-section", {
      has: page.locator('.jj-diff-scroll[data-path="Cargo.toml"]'),
    });
    await cargo.scrollIntoViewIfNeeded();
    await expect(page.locator(".jj-file-section .jj-diff-lines")).toHaveCount(5);
    const group = cargo.getByRole("checkbox", { name: /Change group 1/ });
    await group.tap();
    await ready(page);
    await expect(
      page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true }),
    ).toBeChecked();
    await expect(group).toHaveCount(0);
    await page.getByText("Local review data", { exact: true }).click();
    await page.getByLabel("Import review state", { exact: true }).setInputFiles({
      name: "review.json",
      mimeType: "application/json",
      buffer: Buffer.from('{"reviewed":{},"notes":[]}'),
    });
    await ready(page);
    await expect(group).not.toBeChecked();
  });

  test("the file drawer filters the list and jumps to the picked file", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    await page.locator(".jj-files-button").click();
    const drawer = page.getByRole("dialog", { name: "Files" });
    await expect(drawer).toBeVisible();
    await drawer.getByRole("button", { name: "Filter files", exact: true }).click();
    await drawer.getByRole("searchbox").fill("cargo");
    await expect(drawer.locator(".jj-file-select")).toHaveCount(1);
    await drawer.getByRole("button", { name: "Cargo.toml", exact: true }).click();
    await expect(drawer).toBeHidden();
    await expect(page.locator('.jj-diff-scroll[data-path="Cargo.toml"]')).toBeInViewport();
    await page.locator(".jj-files-button").click();
    await expect(drawer.getByRole("searchbox")).toHaveValue("cargo");
  });

  test("a picked file stays on top while the files above it render", async ({ page }) => {
    await page.addInitScript(() => {
      const renders = new Set<number>();
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        postMessage(message: { kind?: string; id?: number }) {
          if (message.kind === "render") renders.add(message.id!);
          super.postMessage(message);
        }
        set onmessage(handler: (event: MessageEvent) => void) {
          super.onmessage = (event) =>
            renders.has(event.data.id) ? setTimeout(() => handler(event), 500) : handler(event);
        }
      };
    });
    await page.goto("/");
    await ready(page);
    await page.locator(".jj-files-button").click();
    await page
      .getByRole("dialog", { name: "Files" })
      .getByRole("button", { name: "Cargo.toml", exact: true })
      .click();
    await expect(page.locator(".jj-file-section .jj-diff-lines")).toHaveCount(5);
    await expect(
      page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true }),
    ).toBeInViewport();
  });

  test("J and K move between files and / opens the file filter", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    const sections = page.locator(".jj-file-section");
    await page.keyboard.press("j");
    await expect(sections.nth(1)).toBeFocused();
    await page.keyboard.press("j");
    await expect(sections.nth(2)).toBeFocused();
    await expect(sections.nth(2).locator(".jj-file-heading")).toBeInViewport();
    await page.keyboard.press("k");
    await expect(sections.nth(1)).toBeFocused();
    await page.keyboard.press("/");
    await expect(page.getByRole("dialog", { name: "Files" }).getByRole("searchbox")).toBeFocused();
  });

  test("J starts from the file on screen after a scroll", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    const file = (index: number) => ({
      path: `f${index}.txt`,
      reviewIdentity: `${index}`,
      before: "",
      after: Array.from({ length: 20 }, (_, line) => `line ${line}\n`).join(""),
    });
    await page.getByLabel("Import change", { exact: true }).setInputFiles({
      name: "change.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          schemaVersion: 1,
          repositoryId: "test/repository",
          changeId: "stable-change",
          title: "Many files",
          files: Array.from({ length: 6 }, (_, index) => file(index)),
        }),
      ),
    });
    await ready(page);
    const sections = page.locator(".jj-file-section");
    // Every section renders before the scroll, so no late render above moves the file off the top.
    await expect(page.locator(".jj-file-section .jj-diff-lines")).toHaveCount(6);
    await page.keyboard.press("j");
    await expect(sections.nth(1)).toBeFocused();
    await sections.nth(3).evaluate((section) => section.scrollIntoView({ block: "start" }));
    await page.keyboard.press("j");
    await expect(sections.nth(4)).toBeFocused();
  });

  test("Escape clears the drawer's filter before closing the drawer", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    await page.locator(".jj-files-button").click();
    const drawer = page.getByRole("dialog", { name: "Files" });
    await drawer.getByRole("button", { name: "Filter files", exact: true }).click();
    const search = drawer.getByRole("searchbox");
    await search.fill("cargo");
    await search.press("Escape");
    await expect(drawer).toBeVisible();
    await expect(search).toHaveValue("");
    await expect(drawer.locator(".jj-file-select")).toHaveCount(5);
    await search.press("Escape");
    await expect(drawer).toBeHidden();
  });

  test("the drawer stays closed after the window widens and narrows again", async ({ page }) => {
    await page.goto("/");
    await ready(page);
    await page.locator(".jj-files-button").click();
    await expect(page.getByRole("dialog", { name: "Files" })).toBeVisible();
    await page.setViewportSize({ width: 1000, height: 844 });
    await expect(page.locator(".jj-files-button")).toBeHidden();
    await page.setViewportSize({ width: 390, height: 844 });
    await expect(page.locator(".jj-files-button")).toBeVisible();
    await expect(page.getByRole("dialog", { name: "Files" })).toBeHidden();
  });

  async function longBundle(page: Page) {
    await page.goto("/");
    await ready(page);
    const after = Array.from({ length: 120 }, (_, line) => `line ${line}`).join("\n");
    await page.getByLabel("Import change", { exact: true }).setInputFiles({
      name: "change.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          schemaVersion: 1,
          repositoryId: "test/repository",
          changeId: "stable-change",
          title: "Long",
          files: [
            { path: "long.txt", reviewIdentity: "1", before: "", after },
            { path: "next.txt", reviewIdentity: "2", before: "", after: "x\n" },
          ],
        }),
      ),
    });
    await ready(page);
    await page.locator(".jj-diff-cell").nth(100).scrollIntoViewIfNeeded();
  }

  test("folding a long file from a group keeps its header in view and focused", async ({
    page,
  }) => {
    await longBundle(page);
    const toggle = page.getByRole("checkbox", { name: "Mark reviewed: long.txt", exact: true });
    await page
      .locator(".jj-file-section")
      .first()
      .getByRole("checkbox", { name: /Change group 1/ })
      .focus();
    await page.keyboard.press("Space");
    await ready(page);
    await expect(page.locator(".jj-file-section").first().locator(".jj-diff-cell")).toHaveCount(0);
    await expect(toggle).toBeInViewport();
    await expect(toggle).toBeFocused();
  });

  test("a note draft holds the fold and the reader's place", async ({ page }) => {
    await longBundle(page);
    await page.getByRole("button", { name: "Add note: After 100", exact: true }).click();
    const draft = page.getByRole("textbox", { name: "Write a review note", exact: true });
    await draft.fill("Keep this draft");
    await page.getByRole("checkbox", { name: "Mark reviewed: long.txt", exact: true }).tap();
    await ready(page);
    await expect(draft).toBeInViewport();
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.locator(".jj-file-section").first().locator(".jj-diff-cell")).toHaveCount(0);
    await expect(
      page.getByRole("checkbox", { name: "Mark reviewed: long.txt", exact: true }),
    ).toBeInViewport();
  });

  test("a note on a reviewed file stays in the banner after a reload", async ({ page }) => {
    await longBundle(page);
    await page.getByRole("button", { name: "Add note: After 1", exact: true }).first().click();
    await page
      .getByRole("textbox", { name: "Write a review note", exact: true })
      .fill("Still here");
    await page.getByRole("button", { name: "Save note", exact: true }).click();
    await ready(page);
    await page.getByRole("checkbox", { name: "Mark reviewed: long.txt", exact: true }).tap();
    await ready(page);
    await page.reload();
    await ready(page);
    await expect(page.locator(".jj-notes-banner")).toContainText("Still here");
  });

  test("a refresh keeps every section in place until its new render arrives", async ({ page }) => {
    await longBundle(page);
    const change = {
      schemaVersion: 1,
      repositoryId: "test/repository",
      changeId: "stable-change",
      title: "Long",
      files: [
        {
          path: "long.txt",
          reviewIdentity: "1",
          before: "",
          after: Array.from({ length: 120 }, (_, line) => `line ${line}`).join("\n"),
        },
        { path: "next.txt", reviewIdentity: "2", before: "", after: "x\n" },
      ],
    };
    await page.getByRole("button", { name: "Add note: After 100", exact: true }).click();
    const draft = page.getByRole("textbox", { name: "Write a review note", exact: true });
    await draft.fill("Keep this draft");
    await page.evaluate(() => {
      const state = window as unknown as { shortest: number };
      state.shortest = Infinity;
      new ResizeObserver(() => {
        state.shortest = Math.min(state.shortest, document.documentElement.scrollHeight);
      }).observe(document.body);
    });
    const before = await page.evaluate(() => document.documentElement.scrollHeight);
    await page.getByLabel("Import change", { exact: true }).setInputFiles({
      name: "change.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(change)),
    });
    await ready(page);
    expect(
      await page.evaluate(() => (window as unknown as { shortest: number }).shortest),
    ).toBeGreaterThanOrEqual(before - 1);
    await expect(draft).toHaveValue("Keep this draft");
    await page.getByRole("button", { name: "Save note", exact: true }).click();
    await ready(page);
    await expect(page.locator(".jj-review-note.jj-current")).toContainText("Keep this draft");
  });

  test("a draft outside the open file stays saveable after a refresh, and survives a failed one", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      const state = window as unknown as { failRender?: number };
      const failed = new Set<number>();
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        postMessage(message: { kind?: string; id?: number; file?: number }) {
          if (message.kind === "render" && message.file === state.failRender) {
            failed.add(message.id!);
            state.failRender = undefined;
          }
          super.postMessage(message);
        }
        set onmessage(handler: (event: MessageEvent) => void) {
          super.onmessage = (event) =>
            handler(
              failed.has(event.data.id)
                ? new MessageEvent("message", { data: { id: event.data.id, error: "Render lost" } })
                : event,
            );
        }
      };
    });
    await page.goto("/");
    await ready(page);
    const change = Buffer.from(
      JSON.stringify({
        schemaVersion: 1,
        repositoryId: "test/repository",
        changeId: "stable-change",
        title: "Two",
        files: [
          { path: "a.txt", reviewIdentity: "1", before: "", after: "a\n" },
          { path: "b.txt", reviewIdentity: "2", before: "", after: "b\n" },
        ],
      }),
    );
    const reopen = () =>
      page
        .getByLabel("Import change", { exact: true })
        .setInputFiles({ name: "change.json", mimeType: "application/json", buffer: change });
    await reopen();
    await ready(page);
    const section = page.locator(".jj-file-section", {
      has: page.locator('.jj-diff-scroll[data-path="b.txt"]'),
    });
    await section.getByRole("button", { name: "Add note: After 1", exact: true }).click();
    const draft = page.getByRole("textbox", { name: "Write a review note", exact: true });
    await draft.fill("Keep this draft");
    await page.evaluate(() => {
      (window as unknown as { failRender?: number }).failRender = 1;
    });
    await reopen();
    await ready(page);
    await section.getByRole("button", { name: "Retry", exact: true }).click();
    await expect(section.getByRole("button", { name: "Retry", exact: true })).toHaveCount(0);
    await expect(draft).toHaveValue("Keep this draft");
    await reopen();
    await ready(page);
    await page.getByRole("button", { name: "Save note", exact: true }).click();
    await ready(page);
    await expect(section.locator(".jj-review-note.jj-current")).toContainText("Keep this draft");
  });

  test("retries a file whose diff failed to load", async ({ page }) => {
    await page.addInitScript(() => {
      const failed = new Set<number>();
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        postMessage(message: { kind?: string; id?: number; file?: number }) {
          if (message.kind === "render" && message.file === 4 && !failed.size)
            failed.add(message.id!);
          super.postMessage(message);
        }
        set onmessage(handler: (event: MessageEvent) => void) {
          super.onmessage = (event) =>
            handler(
              failed.has(event.data.id)
                ? new MessageEvent("message", { data: { id: event.data.id, error: "Render lost" } })
                : event,
            );
        }
      };
    });
    await page.goto("/");
    await ready(page);
    const cargo = page.locator(".jj-file-section", {
      has: page.locator('.jj-diff-scroll[data-path="Cargo.toml"]'),
    });
    await cargo.scrollIntoViewIfNeeded();
    await expect(page.locator(".jj-file-section .jj-diff-lines")).toHaveCount(4);
    await cargo.getByRole("button", { name: "Retry", exact: true }).click();
    await expect(cargo.locator(".jj-diff-cell").first()).toBeVisible();
  });
});
