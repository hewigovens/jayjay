import { expect, test } from "@playwright/test";
import { openPull } from "./pullFixture";

test.use({ storageState: { cookies: [], origins: [] } });

test("review mode starts off without marks, and turning it on brings them back", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await openPull(page);
  const files = page.locator(".jj-file-column");
  const mark = page.getByRole("checkbox", { name: /^Mark reviewed/ });
  await expect(files.locator(".jj-kind-badge")).toHaveAttribute("title", "Modified");
  await expect(mark).toHaveCount(0);
  await expect(page.locator(".jj-group-toggle")).toHaveCount(0);
  const sync = page.getByRole("button", { name: /Viewed checkboxes/ });
  await expect(sync).toHaveCount(0);

  await files.getByRole("button", { name: "Turn on review mode", exact: true }).click();
  await expect(mark).toHaveCount(2);
  await expect(page.locator(".jj-group-toggle")).toHaveCount(1);
  await expect(sync).toHaveCount(1);
  await page.reload();
  await expect(
    files.getByRole("button", { name: "Turn off review mode", exact: true }),
  ).toBeVisible();
});
