import { expect, test } from "@playwright/test";

const diff = `diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -40,7 +40,7 @@ fn main() {
 let a = 1;
 let b = 1;
 let c = 1;
-let old = 2;
+let new = 2;
 let d = 3;
 let e = 3;
 let f = 3;
diff --git a/docs/b.md b/docs/b.md
index 3333333..4444444 100644
--- a/docs/b.md
+++ b/docs/b.md
@@ -1 +1 @@
-old
+new
`;

test("a picked diff file opens in the browser at its own line numbers", async ({ page }) => {
  await page.goto("/");
  const sent: string[] = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.origin !== new URL(page.url()).origin || url.pathname.startsWith("/api/"))
      sent.push(request.url());
  });
  await page
    .locator("input[type=file]")
    .setInputFiles({ name: "change.diff", mimeType: "text/x-diff", buffer: Buffer.from(diff) });

  const pane = page.locator('.jj-diff-scroll[data-path="src/lib.rs"]');
  await expect(pane).toContainText("let new = 2;");
  await expect(pane.locator(".jj-line-number").first()).toHaveText("40");
  await expect(pane.locator(".jj-context-row")).toHaveText("↕ 39 unmodified lines");
  await expect(page.getByText("All lines")).toHaveCount(0);
  await expect(page).toHaveURL("/patch");
  const unopened = page
    .locator(".jj-file-column")
    .getByRole("checkbox", { name: "Mark reviewed: docs/b.md", exact: true });
  await page.getByRole("button", { name: "2 files", exact: true }).click();
  await unopened.click();
  await expect(unopened).toBeChecked();
  expect(sent).toEqual([]);
  const stored = await page.evaluate(async () => {
    const database = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open("jayjay-pr");
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    const values = database.transaction("data").objectStore("data").getAll();
    await new Promise((resolve) => (values.onsuccess = resolve));
    return JSON.stringify(values.result);
  });
  expect(stored).toContain("src/lib.rs");
  expect(stored).not.toContain("let new = 2;");

  await page.reload();
  await expect(page.getByRole("button", { name: "Open a diff or patch file" })).toBeVisible();
  await expect(page).toHaveURL("/");
});
