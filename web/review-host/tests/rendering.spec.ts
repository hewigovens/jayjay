import { expect, test } from "@playwright/test";
import { bundle, ready } from "./support";

test("reviews a file without syntax when its language pack cannot load", async ({ page }) => {
  await page.route("**/jj_diff_data_bg*.wasm", (route) => route.abort("failed"));
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "config/settings.json", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-detail .jj-notice")).toContainText(
    "Syntax highlighting is unavailable",
  );
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: config/settings.json", exact: true }),
  ).toBeChecked();
});

test("wraps long lines instead of widening the diff", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await bundle(page, `foo\n${"wrapped ".repeat(80)}\nbar\n`);
  for (const layout of ["unified", "split"]) {
    if (layout === "split") {
      await page.locator(".jj-header-actions > button.jj-header-action").click();
      await ready(page);
    }
    const scroll = page.locator(".jj-diff-scroll");
    expect(
      await scroll.evaluate((element) => element.scrollWidth - element.clientWidth),
      layout,
    ).toBeLessThanOrEqual(1);
    const cell = page.locator(".jj-diff-cell", { hasText: "wrapped wrapped" }).last();
    expect((await cell.boundingBox())!.height, layout).toBeGreaterThan(19 * 2);
  }
});

test("a reviewed file keeps its diff on a wide screen", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await bundle(page);
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
  await expect(page.locator(".jj-diff-cell").first()).toBeVisible();
});

test("a shown image preview replaces its source", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const svg = (size: number) =>
    `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}"></svg>\n`;
  const change = Buffer.from(
    JSON.stringify({
      schemaVersion: 1,
      repositoryId: "test/repository",
      changeId: "stable-change",
      title: "Images",
      files: [
        { path: "fine.svg", reviewIdentity: "1", before: svg(8), after: svg(16) },
        { path: "broken.svg", reviewIdentity: "3", before: svg(8), after: "<svg\n" },
      ],
    }),
  );
  await page
    .getByLabel("Import change", { exact: true })
    .setInputFiles({ name: "change.json", mimeType: "application/json", buffer: change });
  await ready(page);
  await expect(page.locator(".jj-image-side img")).toHaveCount(2);
  await expect(page.locator(".jj-diff-cell")).toHaveCount(0);
  await page.getByRole("button", { name: "Show source", exact: true }).click();
  await page
    .getByRole("button", { name: /^Add note: After/ })
    .first()
    .click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Check the size");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await ready(page);
  await page.getByRole("button", { name: "broken.svg", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-diff-cell").first()).toBeVisible();
  await page.getByRole("button", { name: "fine.svg", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-note.jj-current")).toContainText("Check the size");
  await page.getByRole("button", { name: "Resolve", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-diff-cell").first()).toBeVisible();
  await page
    .getByRole("button", { name: /^Add note: After/ })
    .first()
    .click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Keep this draft");
  await page
    .getByLabel("Import change", { exact: true })
    .setInputFiles({ name: "change.json", mimeType: "application/json", buffer: change });
  await ready(page);
  await expect(page.getByRole("textbox", { name: "Write a review note", exact: true })).toHaveValue(
    "Keep this draft",
  );
  await expect(page.locator(".jj-image-side img")).toHaveCount(2);
});

test("gives each file kind its own colour and shows the open file icon", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.getByLabel("Import change", { exact: true }).setInputFiles({
    name: "change.json",
    mimeType: "application/json",
    buffer: Buffer.from(
      JSON.stringify({
        schemaVersion: 1,
        repositoryId: "test/repository",
        changeId: "stable-change",
        title: "Kinds",
        files: [
          { path: "added.rs", reviewIdentity: "1", before: "", after: "a\n" },
          { path: "changed.rs", reviewIdentity: "2", before: "a\n", after: "b\n" },
          { path: "removed.rs", reviewIdentity: "3", before: "a\n", after: "" },
          { path: "moved.rs", oldPath: "old.rs", reviewIdentity: "4", before: "a\n", after: "a\n" },
        ],
      }),
    ),
  });
  await ready(page);
  const colours = await page
    .locator(".jj-kind-dot")
    .evaluateAll((dots) => dots.map((dot) => getComputedStyle(dot).backgroundColor));
  expect(new Set(colours).size).toBe(4);
  await expect(page.locator(".jj-file-title .jj-seti")).toHaveCount(1);
  expect(
    await page.evaluate(async () => {
      await document.fonts.ready;
      return document.fonts.check('16px "JayJay Seti"', "\ue001");
    }),
  ).toBe(true);
});

test("ignores a slow syntax result after selecting another file", async ({ page }) => {
  await page.addInitScript(() => {
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        this.addEventListener("message", (event) => {
          const result = event.data.result;
          const file = result?.overview?.files[result.file]?.path;
          if (file) document.documentElement.dataset.workerFile = file;
        });
      }
    };
  });
  let release: () => void = () => {};
  const blocked = new Promise<void>((resolve) => {
    release = resolve;
  });
  let requested: () => void = () => {};
  const requestStarted = new Promise<void>((resolve) => {
    requested = resolve;
  });
  await page.route("**/jj_diff_web_bg*.wasm", async (route) => {
    requested();
    await blocked;
    await route.continue();
  });
  await page.goto("/");
  await ready(page);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click();
  await requestStarted;
  await page.getByRole("button", { name: "src/greeting.rs", exact: true }).click();
  await ready(page);
  release();
  await expect(page.locator("html")).toHaveAttribute("data-worker-file", "src/web/session.ts");
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  await expect(page.locator(".jj-file-heading h2")).toHaveText("src/greeting.rs");
  await expect(page.getByRole("button", { name: "src/greeting.rs", exact: true })).toHaveAttribute(
    "aria-current",
    "true",
  );
});

test("keeps a host's generic class and token names out of the review", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const fingerprint = () =>
    page.locator(".jj-review-workspace").evaluate((root) => {
      const nodes = [root, ...root.querySelectorAll("*")];
      return JSON.stringify({
        tokens: ["--jj-accent", "--jj-surface", "--jj-ink"].map((name) =>
          getComputedStyle(root).getPropertyValue(name).trim(),
        ),
        styles: nodes.map((node) => {
          const computed = getComputedStyle(node);
          return [
            computed.color,
            computed.backgroundColor,
            computed.display,
            computed.borderTopWidth,
          ];
        }),
      });
    });
  const before = await fingerprint();
  await page.addStyleTag({
    content: `
    .active, .label, .new, .primary { color: rgb(255, 0, 0); background: rgb(255, 0, 255); display: none; border-width: 9px; }
    * { --page: rgb(1, 2, 3); --surface: rgb(4, 5, 6); --ink: rgb(7, 8, 9); --accent: rgb(10, 11, 12); }
  `,
  });
  expect(await fingerprint()).toBe(before);
});

test("a very long diff renders its first rows until the reader asks for the rest", async ({
  page,
}) => {
  await page.goto("/");
  await ready(page);
  const lines = Array.from({ length: 6000 }, (_, line) => `line ${line}\n`).join("");
  await bundle(page, lines);
  const rows = page.locator(".jj-diff-row");
  await expect(rows).toHaveCount(2000);
  const group = page.getByRole("checkbox", { name: /Change group 1/ });
  await expect(group).toBeDisabled();
  await page.getByRole("button", { name: "Show the rest of this diff" }).click();
  await expect(rows).toHaveCount(6002);
  await expect(group).toBeEnabled();
});

test("the row limit stops before a change group rather than inside it", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  const before = Array.from({ length: 3000 }, (_, line) => `line ${line}\n`);
  const after = before.map((text, line) =>
    line < 700 || (line >= 800 && line < 2000) ? `edited ${text}` : text,
  );
  await page.getByLabel("Import change", { exact: true }).setInputFiles({
    name: "change.json",
    mimeType: "application/json",
    buffer: Buffer.from(
      JSON.stringify({
        schemaVersion: 1,
        repositoryId: "test/repository",
        changeId: "stable-change",
        title: "Two groups",
        files: [
          { path: "a.txt", reviewIdentity: "1", before: before.join(""), after: after.join("") },
        ],
      }),
    ),
  });
  await ready(page);
  const groups = page.locator(".jj-group-toggle");
  await expect(groups).toHaveCount(1);
  await expect(groups).toBeEnabled();
  expect(await page.locator(".jj-diff-row").count()).toBeLessThan(2000);
  await page.getByRole("button", { name: "Show the rest of this diff" }).click();
  await expect(groups).toHaveCount(2);
});
