import { expect, test } from "@playwright/test";
import { manifest, sha } from "./pullFixture";

test("a GitHub page the site can't open says so and lists the forms it can", async ({ page }) => {
  await page.goto("/octo/demo/commits/main");
  const alert = page.getByRole("alert");
  await expect(alert).toContainText("JayJay Diff can’t open this GitHub page:");
  await expect(alert.locator("code")).toHaveText("https://github.com/octo/demo/commits/main");
  await expect(alert).toContainText("github.com/owner/repository/commit/1a2b3c4");
  await expect(page.getByRole("textbox", { name: "Pull request URL" })).toHaveValue(
    "https://github.com/octo/demo/commits/main",
  );
});

test("a two-dot comparison offers its three-dot form", async ({ page }) => {
  await page.route("**/api/octo/demo/compare/main...feature", (route) =>
    route.fulfill({ json: { ...manifest, number: undefined, range: "main...feature" } }),
  );
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({ body: "fn hola() {}\n" }),
  );
  await page.goto("/octo/demo/compare/main..feature");
  await expect(page.getByRole("alert")).toContainText("two-dot (..) comparisons");
  await page.getByRole("link", { name: "octo/demo main...feature" }).click();
  await expect(page).toHaveURL(/\/octo\/demo\/compare\/main\.\.\.feature$/);
  await expect(page.getByRole("heading", { name: manifest.title })).toBeVisible();
});

test("a commit URL opens that commit's diff", async ({ page }) => {
  await page.route("**/api/octo/demo/commit/abc1234", (route) =>
    route.fulfill({
      json: { ...manifest, number: undefined, title: "Translate the greeting", commit: sha("b") },
    }),
  );
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({ body: "fn hola() {}\n" }),
  );
  await page.goto("/octo/demo/commit/abc1234");
  await expect(page.getByRole("heading", { name: "Translate the greeting" })).toBeVisible();
  await expect(page.locator(".jj-diff-lines")).toBeVisible();
});

test("a missing or private comparison says so without offering a retry", async ({ page }) => {
  await page.route("**/api/octo/demo/compare/main...gone", (route) =>
    route.fulfill({ status: 404, json: { error: "not-found" } }),
  );
  await page.goto("/octo/demo/compare/main...gone");
  await expect(page.getByRole("alert")).toContainText(
    "This comparison doesn’t exist, or its repository is private. JayJay Diff opens public repositories only.",
  );
  await expect(page.getByRole("button", { name: "Retry" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Open on GitHub" })).toHaveAttribute(
    "href",
    "https://github.com/octo/demo/compare/main...gone",
  );
});
