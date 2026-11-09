import { expect, test } from "@playwright/test";
import { manifest } from "./pullFixture";

test("files without a text diff say why", async ({ page }) => {
  const files = [
    { path: "data.bin", kind: "modified", added: 0, removed: 0, sha: "b1", binary: true },
    {
      path: "run.sh",
      kind: "modified",
      added: 0,
      removed: 0,
      sha: null,
      oldMode: "100644",
      newMode: "100755",
    },
    { path: "pkg/__init__.py", kind: "added", added: 0, removed: 0, sha: "e69de29" },
  ];
  await page.route("**/api/octo/demo/pull/7", (route) =>
    route.fulfill({ json: { ...manifest, files } }),
  );
  await page.route("https://raw.githubusercontent.com/**", (route) => {
    const url = route.request().url();
    if (url.endsWith("/data.bin")) return route.fulfill({ body: Buffer.from([0, 1, 2, 0]) });
    return route.fulfill({ body: url.endsWith("/run.sh") ? "echo hi\n" : "" });
  });
  await page.goto("/octo/demo/pull/7");
  const pane = (path: string) => page.locator(`.jj-diff-scroll[data-path="${path}"]`);
  await expect(pane("data.bin")).toHaveText("Binary file not shown.");
  await expect(pane("run.sh").locator(".jj-notice")).toHaveText(
    "File mode changed from 100644 to 100755.",
  );
  await pane("pkg/__init__.py").scrollIntoViewIfNeeded();
  await expect(pane("pkg/__init__.py")).toHaveText("This file is empty.");
});
