import { expect, type Page } from "@playwright/test";

export async function ready(page: Page) {
  await expect(page.locator(".jj-review-workspace")).toHaveAttribute("aria-busy", "false");
  await expect(page.getByRole("alert")).toHaveCount(0);
}

export async function bundle(page: Page, after = "foo \nfoo\nbar\n", identity = "identity-1") {
  await page.getByLabel("Import change", { exact: true }).setInputFiles({
    name: "change.json",
    mimeType: "application/json",
    buffer: Buffer.from(
      JSON.stringify({
        schemaVersion: 1,
        repositoryId: "test/repository",
        changeId: "stable-change",
        title: "Whitespace review",
        files: [{ path: "a.txt", reviewIdentity: identity, before: "foo\nbar\n", after }],
      }),
    ),
  });
  await ready(page);
}
