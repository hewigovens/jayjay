import { expect, test, type Page } from "@playwright/test";
import { manifest, openPull, sha } from "./pullFixture";

async function expectNoSidewaysScroll(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth),
  ).toBeLessThanOrEqual(0);
}

test("the landing page fits a phone and starts no diff engine", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("button", { name: "Open", exact: true })).toBeVisible();
  await expectNoSidewaysScroll(page);
  expect(page.workers()).toHaveLength(0);
});

test("header actions are touch-sized and pickers open as a sheet", async ({ page }) => {
  await openPull(page);
  await expectNoSidewaysScroll(page);
  const actions = page.locator(".pull-app-actions .jj-tool");
  expect(await actions.count()).toBeGreaterThanOrEqual(4);
  for (const box of await actions.evaluateAll((tools) =>
    tools.map((tool) => tool.getBoundingClientRect()),
  )) {
    expect(box.width).toBeGreaterThanOrEqual(40);
    expect(box.height).toBeGreaterThanOrEqual(40);
  }

  await expect(page.locator(".jj-change-id")).toHaveCount(0);
  await expect(page.locator(".jj-byline time")).toBeVisible();
  await page
    .locator(".pull-app-actions")
    .getByRole("button", { name: "Keyboard shortcuts" })
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.keyboard.press("Escape");
  const toggle = page.getByRole("button", { name: "Commits" });
  await toggle.click();
  const menu = page.getByRole("list", { name: "Commits" });
  const sheet = await menu.boundingBox();
  const viewport = page.viewportSize()!;
  expect(sheet!.width).toBeGreaterThan(viewport.width - 32);
  expect(viewport.height - (sheet!.y + sheet!.height)).toBeLessThanOrEqual(16);
  const row = await menu.getByRole("button").first().boundingBox();
  expect(row!.height).toBeGreaterThanOrEqual(44);
  await page.keyboard.press("Escape");
  await expect(menu).toHaveCount(0);
  await expect(toggle).toBeFocused();
  await page.route("**/api/octo/demo/pull/7/commits/**", () => {});
  await toggle.click();
  await menu.getByRole("button", { name: /Translate the greeting/ }).click();
  await expect(page.locator(".jj-review-workspace")).toHaveAttribute("aria-busy", "true");
  await expect(page.locator(".jj-change-id")).toHaveCount(0);
});

test("the sign-in dialog closes on Escape or a backdrop tap, not a tap inside", async ({
  page,
}) => {
  let polls = 0;
  await page.route("**/api/login/device", (route) =>
    route.fulfill({
      json: {
        deviceCode: "device",
        userCode: "ABCD-1234",
        verificationUri: "https://github.com/login/device",
        interval: 1,
        expiresIn: 900,
      },
    }),
  );
  await page.route("**/api/login/token", (route) => {
    polls++;
    return route.fulfill({ json: { error: "pending" } });
  });
  await openPull(page);
  const sync = page.getByRole("button", { name: /Viewed/ });
  const dialog = page.getByRole("dialog");

  await sync.click();
  await page.getByText("ABCD-1234").click();
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);

  await sync.click();
  await expect(dialog).toBeVisible();
  await page.mouse.click(4, 4);
  await expect(dialog).toHaveCount(0);
  const stopped = polls;
  await page.waitForTimeout(1500);
  expect(polls).toBe(stopped);
});

test("sharing copies the link and offers manual copying only when clipboard access fails", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.addInitScript(() => {
    localStorage.setItem("jayjay.github.token", "test-token");
    Object.defineProperty(navigator, "clipboard", {
      value: {
        async write(items: ClipboardItem[]) {
          const text = await (await items[0].getType("text/plain")).text();
          sessionStorage.setItem("copied-link", text);
        },
      },
      configurable: true,
    });
  });
  const url = "https://diff.hewig.dev/api/exports/test-link";
  const exported: unknown[] = [];
  await page.route("**/api/exports", (route) => {
    exported.push(route.request().postDataJSON());
    return route.fulfill({ json: { url } });
  });
  await openPull(page);
  const share = page.getByRole("button", { name: "Share notes with an agent" });
  await share.click();
  await expect(page.getByRole("status")).toContainText("Link copied");
  expect(await page.evaluate(() => sessionStorage.getItem("copied-link"))).toBe(url);
  expect(exported).toMatchObject([{ title: manifest.title }]);
  await expect(page.locator(".share-url")).toHaveCount(0);

  await page.evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      value: { write: () => Promise.reject(new Error("Clipboard denied")) },
    });
  });
  await share.click();
  const manual = page.getByRole("textbox", { name: "Share link", exact: true });
  await expect(manual).toHaveValue(url);
  await expectNoSidewaysScroll(page);
  await expect(page.getByRole("status")).toContainText("copy it manually");
  await manual.focus();
  expect(
    await manual.evaluate((input: HTMLInputElement) => input.selectionEnd! - input.selectionStart!),
  ).toBe(url.length);

  await page.route("**/api/exports", (route) => route.fulfill({ status: 503 }));
  await share.click();
  await expect(page.getByRole("alert")).toHaveText("Couldn’t create the link.");
  await expect(manual).toHaveCount(0);
  await expectNoSidewaysScroll(page);
});

test("refresh reloads the diff while keeping the selected file, layout, and notes", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await openPull(page);
  await page.getByTitle("Switch to side-by-side").click();
  await page.getByRole("button", { name: "Add note: After 1", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Keep this note");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.locator(".jj-review-note")).toContainText("Keep this note");
  const fresh = {
    ...manifest,
    title: "Updated greeter",
    headSha: sha("f"),
    files: [
      { path: "README.md", kind: "added", added: 1, removed: 0, sha: "new" },
      ...manifest.files.map((file) => ({ ...file, sha: "c2", added: 2 })),
    ],
  };
  await page.route(
    `https://raw.githubusercontent.com/octo/demo/${sha("f")}/src/greet.rs`,
    (route) => route.fulfill({ body: "fn hola() {}\n// refreshed\n" }),
  );
  let refreshes = 0;
  await page.route("**/api/octo/demo/pull/7", async (route) => {
    expect(route.request().headers()["cache-control"]).toBe("no-cache");
    refreshes++;
    await route.fulfill({ json: fresh });
  });
  const refresh = page.getByRole("button", { name: "Refresh diff", exact: true });
  await refresh.click();
  await expect(page.getByRole("heading", { name: `${fresh.title} #7` })).toBeVisible();
  await expect(page.locator('.jj-diff-scroll[data-path="src/greet.rs"] .jj-split')).toBeVisible();
  await expect(page.locator(".jj-review-note")).toContainText("Keep this note");
  await expect(page.locator(".jj-diff-lines")).toContainText("// refreshed");
  expect(refreshes).toBe(1);
  await expect(page.getByRole("button", { name: /Sync.*Viewed/ })).toBeVisible();

  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ status: 503, json: { error: "rate-limited" } }),
  );
  await refresh.click();
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toBeVisible();
  await expect(
    page.getByRole("alert").getByRole("button", { name: "Sign in with GitHub", exact: true }),
  ).toBeVisible();
  await expect(refresh).toHaveCount(0);
  await page.route("**/api/octo/demo/pull/7", (route) => route.fulfill({ json: fresh }));
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(0);
  await expect(page.locator('.jj-diff-scroll[data-path="src/greet.rs"] .jj-split')).toBeVisible();
  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ json: { ...fresh, files: [] } }),
  );
  await refresh.click();
  await expect(page.locator(".jj-empty-diff")).toBeVisible();
  await page.route("**/api/octo/demo/pull/7", (route) => route.fulfill({ json: fresh }));
  await refresh.click();
  await expect(page.locator(".jj-split")).toBeVisible();
});

test("file list preference survives page reloads", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await openPull(page);
  const toggle = page.getByRole("button", { name: /^Showing files as/ });
  await expect(toggle).toHaveAttribute("aria-pressed", "true");
  await toggle.click();
  await page.reload();
  await expect(toggle).toHaveAttribute("aria-pressed", "false");
  await toggle.click();
  await page.reload();
  await expect(toggle).toHaveAttribute("aria-pressed", "true");
});
