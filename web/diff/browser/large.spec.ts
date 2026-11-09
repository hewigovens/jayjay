import { expect, test } from "@playwright/test";
import { manifest } from "./pullFixture";

test("a review opens before its first file arrives", async ({ page }) => {
  const files = [
    { path: "Cargo.lock", kind: "modified", added: 9000, removed: 9000, sha: "l1", oldSha: "l0" },
    ...manifest.files,
  ];
  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ json: { ...manifest, files } }),
  );
  const lockfile: string[] = [];
  await page.route("https://raw.githubusercontent.com/**", (route) => {
    if (route.request().url().endsWith("/Cargo.lock")) {
      lockfile.push(route.request().url());
      return;
    }
    return route.fulfill({ body: "fn hola() {}\n" });
  });
  await page.goto("/octo/demo/pull/7");
  await expect(page.getByRole("heading", { name: `${manifest.title} #7` })).toBeVisible();
  await expect(page.locator('.jj-diff-scroll[data-path="Cargo.lock"]')).toHaveText(
    "Loading file and syntax…",
  );
  await expect(
    page.locator('.jj-diff-scroll[data-path="src/greet.rs"] .jj-diff-lines'),
  ).toBeVisible();
  expect(lockfile).toHaveLength(2);
});
