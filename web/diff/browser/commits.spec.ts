import { expect, test } from "@playwright/test";
import { manifest, openPull, sha } from "./pullFixture";

test("the commit sidebar opens commits in place of the picker, and stays hidden once turned off", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await page.route(`**/api/octo/demo/pull/7/commits/${sha("e")}`, (route) =>
    route.fulfill({ json: { ...manifest, title: "Add the greeting", commit: sha("e") } }),
  );
  await openPull(page);
  const sidebar = page.getByRole("complementary", { name: "Commits", exact: true });
  const picker = page.locator(".jj-commit-picker");
  await expect(picker).toBeHidden();
  await expect(sidebar.getByRole("button", { name: "All commits" })).toHaveAttribute(
    "aria-current",
    "true",
  );
  await sidebar.getByRole("button", { name: /Add the greeting/ }).click();
  await expect(page).toHaveURL(new RegExp(`/octo/demo/pull/7/commits/${sha("e")}$`));
  await expect(page.getByRole("heading", { level: 1, name: /Add the greeting/ })).toBeVisible();
  await expect(sidebar.getByRole("button", { name: /Add the greeting/ })).toHaveAttribute(
    "aria-current",
    "true",
  );

  await page.locator(".pull-app-bar").getByRole("button", { name: "Commits", exact: true }).click();
  await expect(sidebar).toBeHidden();
  await expect(picker).toBeVisible();
  await page.reload();
  await expect(picker).toBeVisible();
  await expect(sidebar).toBeHidden();
});

test("a pull request's body opens collapsed and a commit's message opens expanded", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ json: { ...manifest, body: "Greets in Spanish too." } }),
  );
  await page.route(`**/api/octo/demo/pull/7/commits/${sha("e")}`, (route) =>
    route.fulfill({
      json: { ...manifest, title: "Add the greeting", body: "Says hello.", commit: sha("e") },
    }),
  );
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({ body: route.request().url().includes(sha("a")) ? "a\n" : "b\n" }),
  );
  await page.goto("/octo/demo/pull/7");
  const description = page.locator(".jj-change-description");
  await expect(description).toHaveText("Greets in Spanish too.");
  await expect(description).not.toHaveClass(/jj-expanded/);
  await page
    .getByRole("complementary", { name: "Commits", exact: true })
    .getByRole("button", { name: /Add the greeting/ })
    .click();
  await expect(description).toHaveText("Says hello.");
  await expect(description).toHaveClass(/jj-expanded/);
});
