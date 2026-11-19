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
  const picker = page.locator(".commit-picker");
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

test("a pull request with a body opens on its description, and J moves to the first file", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ json: { ...manifest, body: "Greets in Spanish too." } }),
  );
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({ body: route.request().url().includes(sha("a")) ? "a\n" : "b\n" }),
  );
  await page.goto("/octo/demo/pull/7");
  const body = page.locator(".jj-description-body");
  await expect(body).toHaveText("Greets in Spanish too.");
  await expect(
    page.getByRole("button", { name: "Pull request description", exact: true }),
  ).toHaveAttribute("aria-current", "true");
  await page.keyboard.press("j");
  await expect(body).toHaveCount(0);
  await expect(page.locator(".jj-file-heading h2")).toHaveText("src/greet.rs");
});
