import { expect, test } from "@playwright/test";
import { bundle, ready } from "./support";

test("loads the author's avatar only after opting in", async ({ page }) => {
  const png = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
    "base64",
  );
  const lookups: string[] = [];
  page.on("request", (request) => {
    if (/github|gravatar/.test(new URL(request.url()).hostname)) lookups.push(request.url());
  });
  await page.route("https://api.github.com/user/49699333", (route) =>
    route.fulfill({ json: { avatar_url: "https://avatars.githubusercontent.com/in/29110?v=4" } }),
  );
  await page.route("https://avatars.githubusercontent.com/**", (route) =>
    route.fulfill({ contentType: "image/png", body: png }),
  );
  await page.route("https://gravatar.com/**", (route) => route.fulfill({ status: 404 }));
  await page.goto("/");
  await ready(page);
  const author = (authorEmail: string) =>
    page.getByLabel("Import change", { exact: true }).setInputFiles({
      name: "change.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          schemaVersion: 1,
          repositoryId: "test/repository",
          changeId: "avatar-change",
          title: "Avatar",
          author: "dependabot",
          authorEmail,
          files: [{ path: "a.txt", reviewIdentity: "identity", before: "a\n", after: "b\n" }],
        }),
      ),
    });
  await author("49699333+dependabot[bot]@users.noreply.github.com");
  await ready(page);
  await expect(page.locator(".jj-change-header .jj-avatar")).toHaveText("D");
  expect(lookups).toEqual([]);
  await page.getByText("Local review data", { exact: true }).click();
  await page
    .getByLabel("Load author avatars from Gravatar, GitHub, or GitLab", { exact: true })
    .check();
  await expect(page.locator(".jj-change-header .jj-avatar img.jj-loaded")).toHaveAttribute(
    "src",
    "https://avatars.githubusercontent.com/in/29110?v=4&size=96",
  );
  await page.reload();
  await ready(page);
  await expect(page.locator(".jj-change-header .jj-avatar img.jj-loaded")).toHaveCount(1);
  await author("dev@example.com");
  await ready(page);
  await expect(page.locator(".jj-change-header .jj-avatar")).toHaveText("D");
  await expect(page.locator(".jj-change-header .jj-avatar img")).toHaveCount(0);
});

test("imports another change when the stored change cannot open", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.evaluate(
    () =>
      new Promise<void>((resolve, reject) => {
        const open = indexedDB.open("jayjay-review-tests");
        open.onsuccess = () => {
          const transaction = open.result.transaction("data", "readwrite");
          transaction.objectStore("data").put("{}", "active-bundle");
          transaction.oncomplete = () => {
            open.result.close();
            resolve();
          };
          transaction.onabort = () => reject(transaction.error);
        };
        open.onerror = () => reject(open.error);
      }),
  );
  await page.reload();
  await expect(page.getByRole("alert")).toBeVisible();
  await bundle(page);
  await expect(page.locator(".jj-change-header h1")).toHaveText("Whitespace review");
});

test("flags changed review data", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await bundle(page);
  await page.getByRole("button", { name: "Add note: After 1", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Write a review note", exact: true })
    .fill("Inspect the duplicate line");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-review-note.jj-current")).toContainText(
    "Inspect the duplicate line",
  );
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  await bundle(page, "different\n", "identity-2");
  await expect(page.locator(".jj-review-note.jj-stale")).toContainText(
    "Inspect the duplicate line",
  );
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed", exact: true }),
  ).not.toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: a.txt", exact: true }),
  ).toHaveJSProperty("indeterminate", true);
});
