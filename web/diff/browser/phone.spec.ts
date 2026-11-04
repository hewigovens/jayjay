import { expect, test, type Page } from "@playwright/test";
import type { PullManifest } from "../src/github/PullManifest";

const sha = (digit: string) => digit.repeat(40);

const manifest: PullManifest = {
  repository: "octo/demo",
  number: 7,
  title: "Teach the greeter a second language",
  body: "",
  author: null,
  baseSha: sha("a"),
  headSha: sha("b"),
  files: [
    { path: "src/greet.rs", kind: "modified", added: 1, removed: 1, sha: "c1", oldSha: "d1" },
  ],
  commits: [
    { sha: sha("e"), title: "Add the greeting", author: "octo", timestampMillis: 0 },
    { sha: sha("b"), title: "Translate the greeting", author: "octo", timestampMillis: 0 },
  ],
  stack: [
    { number: 6, title: "Introduce the greeter", state: "merged" },
    { number: 7, title: "Teach the greeter a second language", state: "open" },
  ],
};

async function openPull(page: Page) {
  await page.route("**/api/octo/demo/pull/7", (route) => route.fulfill({ json: manifest }));
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({
      body: route.request().url().includes(sha("a")) ? "fn hello() {}\n" : "fn hola() {}\n",
    }),
  );
  await page.goto("/octo/demo/pull/7");
  await expect(page.getByRole("heading", { name: manifest.title })).toBeVisible();
}

async function expectNoSidewaysScroll(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth),
  ).toBeLessThanOrEqual(0);
}

test("the landing page fits a phone", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("button", { name: "Open" })).toBeVisible();
  await expectNoSidewaysScroll(page);
});

test("header actions are touch-sized and pickers open as a sheet", async ({ page }) => {
  await openPull(page);
  await expectNoSidewaysScroll(page);
  const actions = page.locator(".jj-change-actions .jj-tool");
  expect(await actions.count()).toBeGreaterThanOrEqual(4);
  for (const box of await actions.evaluateAll((tools) =>
    tools.map((tool) => tool.getBoundingClientRect()),
  )) {
    expect(box.width).toBeGreaterThanOrEqual(40);
    expect(box.height).toBeGreaterThanOrEqual(40);
  }

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
