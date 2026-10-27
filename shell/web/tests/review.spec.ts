import { readFile } from "node:fs/promises";
import { expect, test, type Page } from "@playwright/test";

async function ready(page: Page) {
  await expect(page.locator(".review-workspace")).toHaveAttribute("aria-busy", "false");
  await expect(page.getByRole("alert")).toHaveCount(0);
}

async function bundle(page: Page, after = "foo \nfoo\nbar\n", identity = "identity-1") {
  await page.getByLabel("Import change", { exact: true }).setInputFiles({ name: "change.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify({
    schemaVersion: 1, repositoryId: "test/repository", changeId: "stable-change", title: "Whitespace review",
    files: [{ path: "a.txt", reviewIdentity: identity, before: "foo\nbar\n", after }],
  })) });
  await ready(page);
}

test("loads syntax on demand, navigates files, and persists group marks and notes", async ({ page }) => {
  const wasm: string[] = [];
  const errors: string[] = [];
  page.on("response", response => { if (response.url().endsWith(".wasm")) wasm.push(response.url()); });
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("/"); await ready(page);
  expect(wasm.length).toBeGreaterThan(0);
  expect(wasm.every(url => /jj_diff_(base|rust)_bg/.test(url))).toBe(true);
  await page.getByRole("button", { name: "Showing files as a tree", exact: true }).click();
  await expect(page.getByRole("button", { name: "src/greeting.rs", exact: true })).toHaveAttribute("aria-current", "true");
  await page.getByRole("button", { name: "Showing files as a flat list", exact: true }).click();
  await page.getByRole("checkbox", { name: /Change group 1/ }).click(); await ready(page);
  await expect(page.getByRole("checkbox", { name: /Change group 1/ })).toBeChecked();
  await expect(page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true })).toHaveJSProperty("indeterminate", true);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Check input handling. 日本語 <img src=x>");
  await page.getByRole("button", { name: "Save note", exact: true }).click(); await ready(page);
  await expect(page.locator(".review-note.current")).toContainText("Check input handling. 日本語 <img src=x>");
  await expect(page.locator(".review-note img")).toHaveCount(0);
  await page.reload(); await ready(page);
  await expect(page.getByRole("checkbox", { name: /Change group 1/ })).toBeChecked();
  await expect(page.locator(".review-note.current")).toContainText("Check input handling");
  await page.getByText("Local review data", { exact: true }).click();
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export review state", exact: true }).click();
  const state = JSON.parse(await readFile((await (await download).path())!,  "utf8"));
  expect(state.notes[0].created_at_ms).toBeGreaterThan(Date.now() - 60_000);
  expect(state.notes[0].ignore_whitespace).toBe(false);
  expect(Object.keys(state.reviewed)).toContain("control-room-preview|src/greeting.rs");
  await page.getByLabel("Import review state", { exact: true }).setInputFiles({ name: "review_store.json", mimeType: "application/json", buffer: Buffer.from('{"reviewed":{},"notes":[]}') });
  await ready(page);
  await expect(page.locator(".review-note")).toHaveCount(0);
  await page.getByRole("button", { name: "Undo import", exact: true }).click(); await ready(page);
  await expect(page.locator(".review-note.current")).toContainText("Check input handling");
  await page.getByRole("button", { name: "Resolve", exact: true }).click(); await ready(page);
  await expect(page.locator(".review-note")).toHaveCount(0);
  await page.getByRole("button", { name: "Resolved: After 5", exact: true }).click();
  await expect(page.locator(".review-note.resolved")).toContainText("Check input handling");
  expect(errors).toEqual([]);
});

test("reviews a file without syntax when its language pack cannot load", async ({ page }) => {
  await page.route("**/jj_diff_data_bg*.wasm", route => route.abort("failed"));
  await page.goto("/"); await ready(page);
  await page.getByRole("button", { name: "config/settings.json", exact: true }).click(); await ready(page);
  await expect(page.locator(".review-detail .notice")).toContainText("Syntax highlighting is unavailable");
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click(); await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed: config/settings.json", exact: true })).toBeChecked();
});

test("wraps long lines instead of widening the diff", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await bundle(page, `foo\n${"wrapped ".repeat(80)}\nbar\n`);
  for (const layout of ["unified", "split"]) {
    if (layout === "split") { await page.locator(".header-actions > button.header-action").click(); await ready(page); }
    const scroll = page.locator(".diff-scroll");
    expect(await scroll.evaluate(element => element.scrollWidth - element.clientWidth), layout).toBeLessThanOrEqual(1);
    const cell = page.locator(".diff-cell", { hasText: "wrapped wrapped" }).last();
    expect((await cell.boundingBox())!.height, layout).toBeGreaterThan(19 * 2);
  }
});

test("loads the author's avatar only after opting in", async ({ page }) => {
  const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==", "base64");
  const lookups: string[] = [];
  page.on("request", request => { if (/github|gravatar/.test(new URL(request.url()).hostname)) lookups.push(request.url()); });
  await page.route("https://api.github.com/user/49699333", route => route.fulfill({ json: { avatar_url: "https://avatars.githubusercontent.com/in/29110?v=4" } }));
  await page.route("https://avatars.githubusercontent.com/**", route => route.fulfill({ contentType: "image/png", body: png }));
  await page.route("https://gravatar.com/**", route => route.fulfill({ status: 404 }));
  await page.goto("/"); await ready(page);
  const author = (authorEmail: string) => page.getByLabel("Import change", { exact: true }).setInputFiles({ name: "change.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify({
    schemaVersion: 1, repositoryId: "test/repository", changeId: "avatar-change", title: "Avatar", author: "dependabot", authorEmail,
    files: [{ path: "a.txt", reviewIdentity: "identity", before: "a\n", after: "b\n" }],
  })) });
  await author("49699333+dependabot[bot]@users.noreply.github.com"); await ready(page);
  await expect(page.locator(".change-header .avatar")).toHaveText("D");
  expect(lookups).toEqual([]);
  await page.getByText("Local review data", { exact: true }).click();
  await page.getByLabel("Load author avatars from Gravatar, GitHub, or GitLab", { exact: true }).check();
  await expect(page.locator(".change-header .avatar img.loaded")).toHaveAttribute("src", "https://avatars.githubusercontent.com/in/29110?v=4&size=96");
  await page.reload(); await ready(page);
  await expect(page.locator(".change-header .avatar img.loaded")).toHaveCount(1);
  await author("dev@example.com"); await ready(page);
  await expect(page.locator(".change-header .avatar")).toHaveText("D");
  await expect(page.locator(".change-header .avatar img")).toHaveCount(0);
});

test("marking a file in the list keeps the open file", async ({ page }) => {
  await page.goto("/"); await ready(page);
  const cargo = page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true });
  await cargo.click(); await ready(page);
  await expect(cargo).toBeChecked();
  await expect(page.locator(".file-heading h2")).toHaveText("src/greeting.rs");
});

test("imports another change when the stored change cannot open", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await page.evaluate(() => new Promise<void>((resolve, reject) => {
    const open = indexedDB.open("jayjay-review-tests");
    open.onsuccess = () => {
      const transaction = open.result.transaction("data", "readwrite");
      transaction.objectStore("data").put("{}", "active-bundle");
      transaction.oncomplete = () => { open.result.close(); resolve(); };
      transaction.onabort = () => reject(transaction.error);
    };
    open.onerror = () => reject(open.error);
  }));
  await page.reload();
  await expect(page.getByRole("alert")).toBeVisible();
  await bundle(page);
  await expect(page.locator(".change-header h1")).toHaveText("Whitespace review");
});

test("keeps the notes filter reachable after the last note is resolved", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Check this");
  await page.getByRole("button", { name: "Save note", exact: true }).click(); await ready(page);
  const filter = page.getByRole("button", { name: "With notes", exact: true });
  await filter.click();
  await page.getByRole("button", { name: "Resolve", exact: true }).click(); await ready(page);
  await filter.click();
  await expect(page.getByRole("button", { name: "Cargo.toml", exact: true })).toBeVisible();
});

test("retries against the newly imported change", async ({ page }) => {
  await page.addInitScript(() => {
    let lose = false;
    const lost = new Set<number>();
    (window as unknown as { loseNextMutation: () => void }).loseNextMutation = () => { lose = true; };
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string; id?: number }) {
        if (message.kind === "mutate" && lose) { lose = false; lost.add(message.id!); }
        super.postMessage(message);
      }
      set onmessage(handler: (event: MessageEvent) => void) {
        super.onmessage = event => handler(lost.delete(event.data.id) ? new MessageEvent("message", { data: { id: event.data.id, error: "Response lost" } }) : event);
      }
    };
  });
  await page.goto("/"); await ready(page);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click(); await ready(page);
  await bundle(page);
  await page.evaluate(() => (window as unknown as { loseNextMutation: () => void }).loseNextMutation());
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Response lost");
  await page.getByRole("button", { name: "Retry", exact: true }).click(); await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
});

test.describe("phone portrait", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test("lists every file's diff in one scroll with its own mark", async ({ page }) => {
    await page.goto("/"); await ready(page);
    await expect(page.locator(".file-column")).toHaveCount(0);
    await expect(page.locator(".file-section")).toHaveCount(5);
    const toggle = page.getByRole("button", { name: "Expand description", exact: true });
    await toggle.click();
    await expect(page.getByRole("button", { name: "Collapse description", exact: true })).toHaveAttribute("aria-expanded", "true");
    const cargo = page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true });
    await cargo.scrollIntoViewIfNeeded();
    await expect(page.locator('.diff-scroll[data-path="Cargo.toml"] .diff-cell').first()).toBeVisible();
    await cargo.tap(); await ready(page);
    await expect(cargo).toBeChecked();
    await expect(page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true })).not.toBeChecked();
    await page.reload(); await ready(page);
    await expect(page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true })).toBeChecked();
  });

  test("importing review state refreshes every loaded file's marks", async ({ page }) => {
    await page.goto("/"); await ready(page);
    const cargo = page.locator(".file-section", { has: page.locator('.diff-scroll[data-path="Cargo.toml"]') });
    await cargo.scrollIntoViewIfNeeded();
    const group = cargo.getByRole("checkbox", { name: /Change group 1/ });
    await group.tap(); await ready(page);
    await expect(group).toBeChecked();
    await page.getByText("Local review data", { exact: true }).click();
    await page.getByLabel("Import review state", { exact: true }).setInputFiles({ name: "review.json", mimeType: "application/json", buffer: Buffer.from('{"reviewed":{},"notes":[]}') });
    await ready(page);
    await expect(group).not.toBeChecked();
  });

  test("a lazy render that predates a mark does not overwrite it", async ({ page }) => {
    await page.addInitScript(() => {
      let held: number | undefined;
      const state = window as unknown as { releaseRender: () => void };
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        postMessage(message: { kind?: string; file?: number; id?: number }) {
          if (message.kind === "render" && message.file === 4 && held === undefined) held = message.id;
          super.postMessage(message);
        }
        set onmessage(handler: (event: MessageEvent) => void) {
          super.onmessage = event => {
            if (event.data.id !== held) return handler(event);
            state.releaseRender = () => handler(event);
          };
        }
      };
    });
    await page.goto("/"); await ready(page);
    await page.waitForFunction(() => "releaseRender" in window);
    const cargo = page.getByRole("checkbox", { name: "Mark reviewed: Cargo.toml", exact: true });
    await cargo.tap(); await ready(page);
    await expect(cargo).toBeChecked();
    await page.evaluate(async () => {
      (window as unknown as { releaseRender: () => void }).releaseRender();
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    });
    const section = page.locator(".file-section", { has: page.locator('.diff-scroll[data-path="Cargo.toml"]') });
    await expect(section.getByRole("checkbox", { name: /Change group 1/ })).toBeChecked();
  });

  test("retries a file whose diff failed to load", async ({ page }) => {
    await page.addInitScript(() => {
      const failed = new Set<number>();
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        postMessage(message: { kind?: string; id?: number; file?: number }) {
          if (message.kind === "render" && message.file === 4 && !failed.size) failed.add(message.id!);
          super.postMessage(message);
        }
        set onmessage(handler: (event: MessageEvent) => void) {
          super.onmessage = event => handler(failed.has(event.data.id) ? new MessageEvent("message", { data: { id: event.data.id, error: "Render lost" } }) : event);
        }
      };
    });
    await page.goto("/"); await ready(page);
    const cargo = page.locator(".file-section", { has: page.locator('.diff-scroll[data-path="Cargo.toml"]') });
    await cargo.scrollIntoViewIfNeeded();
    await cargo.getByRole("button", { name: "Retry", exact: true }).click();
    await expect(cargo.locator(".diff-cell").first()).toBeVisible();
  });
});

test("uses the recorded whitespace mode and flags changed review data", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await bundle(page);
  await page.getByLabel("Ignore whitespace", { exact: true }).check(); await ready(page);
  await page.getByRole("button", { name: "Add note: After 2", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Inspect the duplicate line");
  await page.getByRole("button", { name: "Save note", exact: true }).click(); await ready(page);
  await page.getByLabel("Ignore whitespace", { exact: true }).uncheck(); await ready(page);
  await expect(page.locator(".review-note.current")).toContainText("Inspect the duplicate line");
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click(); await ready(page);
  await bundle(page, "different\n", "identity-2");
  await expect(page.locator(".review-note.stale")).toContainText("Inspect the duplicate line");
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).not.toBeChecked();
  await expect(page.getByRole("checkbox", { name: "Mark reviewed: a.txt", exact: true })).toHaveJSProperty("indeterminate", true);
});

test("ignores a slow syntax result after selecting another file", async ({ page }) => {
  await page.addInitScript(() => {
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        this.addEventListener("message", event => {
          const result = event.data.result;
          const file = result?.overview?.files[result.file]?.path;
          if (file) document.documentElement.dataset.workerFile = file;
        });
      }
    };
  });
  let release: () => void = () => {};
  const blocked = new Promise<void>(resolve => { release = resolve; });
  let requested: () => void = () => {};
  const requestStarted = new Promise<void>(resolve => { requested = resolve; });
  await page.route("**/jj_diff_web_bg*.wasm", async route => { requested(); await blocked; await route.continue(); });
  await page.goto("/"); await ready(page);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click();
  await requestStarted;
  await page.getByRole("button", { name: "src/greeting.rs", exact: true }).click(); await ready(page);
  release();
  await expect(page.locator("html")).toHaveAttribute("data-worker-file", "src/web/session.ts");
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
  await expect(page.locator(".file-heading h2")).toHaveText("src/greeting.rs");
  await expect(page.getByRole("button", { name: "src/greeting.rs", exact: true })).toHaveAttribute("aria-current", "true");
});

test("keeps the file selected until an in-flight note save is acknowledged", async ({ page }) => {
  await page.addInitScript(() => {
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string }) {
        if (message.kind !== "mutate") return super.postMessage(message);
        (window as unknown as { releaseMutation: () => void }).releaseMutation = () => super.postMessage(message);
      }
    };
  });
  await page.goto("/"); await ready(page);
  await page.getByRole("button", { name: "Add note: After 5", exact: true }).click();
  await page.getByRole("textbox", { name: "Write a review note", exact: true }).fill("Saved once");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await page.waitForFunction(() => "releaseMutation" in window);
  await page.getByRole("button", { name: "src/web/session.ts", exact: true }).click();
  await page.evaluate(() => (window as unknown as { releaseMutation: () => void }).releaseMutation());
  await ready(page);
  await expect(page.getByRole("textbox", { name: "Write a review note", exact: true })).toHaveCount(0);
  await expect(page.locator(".file-heading h2")).toHaveText("src/greeting.rs");
  await expect(page.locator(".review-note", { hasText: "Saved once" })).toHaveCount(1);
});

test("retrying after a lost save response re-renders instead of writing again", async ({ page }) => {
  await page.addInitScript(() => {
    const counts = { mutate: 0 };
    (window as unknown as { counts: typeof counts }).counts = counts;
    const lost = new Set<number>();
    const OriginalWorker = window.Worker;
    window.Worker = class extends OriginalWorker {
      postMessage(message: { kind?: string; id?: number }) {
        if (message.kind === "mutate" && ++counts.mutate === 1) lost.add(message.id!);
        super.postMessage(message);
      }
      set onmessage(handler: (event: MessageEvent) => void) {
        super.onmessage = event => handler(lost.delete(event.data.id) ? new MessageEvent("message", { data: { id: event.data.id, error: "Response lost" } }) : event);
      }
    };
  });
  await page.goto("/"); await ready(page);
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Response lost");
  await page.getByRole("button", { name: "Retry", exact: true }).click(); await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
  expect(await page.evaluate(() => (window as unknown as { counts: { mutate: number } }).counts.mutate)).toBe(1);
});

test("serializes independent writes from two tabs", async ({ page, context }) => {
  await page.goto("/"); await ready(page);
  const other = await context.newPage(); await other.goto("/"); await ready(other);
  await Promise.all([
    page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true }).click(),
    other.getByRole("checkbox", { name: "Mark reviewed: src/web/session.ts", exact: true }).click(),
  ]);
  await ready(page); await ready(other);
  await page.reload(); await ready(page);
  await expect(page.getByRole("checkbox", { name: "Mark reviewed: src/greeting.rs", exact: true })).toBeChecked();
  await expect(page.getByRole("checkbox", { name: "Mark reviewed: src/web/session.ts", exact: true })).toBeChecked();
});

test("does not acknowledge a failed browser-storage write", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await page.evaluate(() => new Promise<void>((resolve, reject) => {
    const request = indexedDB.deleteDatabase("jayjay-review-tests");
    request.onsuccess = () => resolve(); request.onerror = () => reject(request.error);
  }));
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).not.toBeChecked();
  await expect(page.locator(".review-detail .result-footer")).not.toContainText("Saved on this device");
});


test("rejects malformed UTF-8 and empty imports without replacing review data", async ({ page }) => {
  await page.goto("/"); await ready(page);
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click(); await ready(page);
  for (const buffer of [Buffer.from([0xff]), Buffer.alloc(0)]) for (const label of ["Import change", "Import review state"]) {
    if (label === "Import review state") await page.getByText("Local review data", { exact: true }).click();
    await page.getByLabel(label, { exact: true }).setInputFiles({ name: "invalid.json", mimeType: "application/json", buffer });
    await expect(page.getByRole("alert")).toBeVisible();
    await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
    await page.reload(); await ready(page);
    await expect(page.getByRole("checkbox", { name: "Mark reviewed", exact: true })).toBeChecked();
  }
});
