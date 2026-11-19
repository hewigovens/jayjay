import { expect, test, type Page } from "@playwright/test";
import { ready } from "./support";

// Each file with a token its grammar must produce.
const files = {
  "src/lib.rs": ["pub fn answer() -> u32 {\n    42\n}\n", "Keyword"],
  "web/App.tsx": [
    'export function App() {\n  return <main className="app">{1}</main>;\n}\n',
    "Keyword",
  ],
  "README.md": ["# Title\n\nSome *emphasis* and `code`.\n", "Type"],
  "app/Main.kt": [
    'fun main() {\n    val greeting = "hello"\n    println(greeting)\n}\n',
    "Keyword",
  ],
  "lib/main.dart": ["void main() {\n  final count = 3;\n  print('count $count');\n}\n", "Keyword"],
};

test("highlights files whose grammars link into the engine at run time", async ({ page }) => {
  await page.goto("/");
  await ready(page);
  await page.getByLabel("Import change", { exact: true }).setInputFiles({
    name: "change.json",
    mimeType: "application/json",
    buffer: Buffer.from(
      JSON.stringify({
        schemaVersion: 1,
        repositoryId: "test/syntax",
        changeId: "syntax-change",
        title: "Languages",
        files: Object.entries(files).map(([path, [after]]) => ({
          path,
          reviewIdentity: path,
          before: "",
          after,
        })),
      }),
    ),
  });
  await ready(page);
  for (const [path, [, token]] of Object.entries(files)) {
    await page.getByRole("button", { name: path, exact: true }).click();
    await ready(page);
    await expect(page.locator(".jj-file-heading h2")).toHaveText(path);
    await expect(page.locator(".jj-review-detail .jj-notice")).toHaveCount(0);
    await expect(page.locator(`.jj-diff-cell .jj-token-${token}`).first(), path).toBeVisible();
  }
});

const notice = (page: Page) => page.locator(".jj-review-detail .jj-notice");

async function open(page: Page, path: string) {
  await page.getByRole("button", { name: path, exact: true }).click();
  await ready(page);
  await expect(page.locator(".jj-file-heading h2")).toHaveText(path);
}

// A relocatable module whose language function traps, as a scanner calling abort would.
const trappingGrammar = Buffer.from([
  0x00,
  0x61,
  0x73,
  0x6d,
  0x01,
  0x00,
  0x00,
  0x00,
  0x00,
  0x0f,
  0x08,
  ...Buffer.from("dylink.0"),
  0x01,
  0x04,
  0x00,
  0x00,
  0x00,
  0x00,
  0x01,
  0x05,
  0x01,
  0x60,
  0x00,
  0x01,
  0x7f,
  0x03,
  0x02,
  0x01,
  0x00,
  0x07,
  0x14,
  0x01,
  0x10,
  ...Buffer.from("tree_sitter_json"),
  0x00,
  0x00,
  0x0a,
  0x05,
  0x01,
  0x03,
  0x00,
  0x00,
  0x0b,
]);

test("a grammar that cannot link is not loaded again", async ({ page }) => {
  await page.route("**/assets/json-*.wasm", (route) =>
    route.fulfill({ contentType: "application/wasm", body: Buffer.from("\0asm\x01\0\0\0") }),
  );
  const worker = page.waitForEvent("worker");
  await page.goto("/");
  await ready(page);
  await (
    await worker
  ).evaluate(() => {
    const compile = WebAssembly.compile;
    Object.assign(self, { compiled: 0 });
    WebAssembly.compile = (bytes) => {
      Object.assign(self, { compiled: Reflect.get(self, "compiled") + 1 });
      return compile(bytes);
    };
  });
  for (const path of ["config/settings.json", "src/greeting.rs", "config/settings.json"]) {
    await open(page, path);
    await expect(notice(page)).toHaveCount(path.endsWith(".json") ? 1 : 0);
  }
  expect(await (await worker).evaluate(() => Reflect.get(self, "compiled"))).toBe(1);
});

test("a page served for a grammar is not cached, so the grammar loads later", async ({ page }) => {
  await page.route("**/assets/json-*.wasm", (route) =>
    route.fulfill({ contentType: "text/html", body: "<!doctype html><title>JayJay</title>" }),
  );
  await page.goto("/");
  await ready(page);
  await open(page, "config/settings.json");
  await expect(notice(page)).toHaveCount(1);
  const cached = await page.evaluate(async () =>
    (await (await caches.open("jayjay-syntax-v1")).keys()).map((request) => request.url),
  );
  expect(cached.some((url) => url.includes("/json-"))).toBe(false);
  expect(cached.some((url) => url.includes("/rust-"))).toBe(true);
  await page.unroute("**/assets/json-*.wasm");
  await open(page, "src/greeting.rs");
  await open(page, "config/settings.json");
  await expect(notice(page)).toHaveCount(0);
  await expect(page.locator(".jj-diff-cell .jj-token-Number").first()).toBeVisible();
});

test("a trap in grammar code restarts the engine and keeps the review working", async ({
  page,
}) => {
  await page.route("**/assets/json-*.wasm", (route) =>
    route.fulfill({ contentType: "application/wasm", body: trappingGrammar }),
  );
  await page.goto("/");
  await ready(page);
  await open(page, "config/settings.json");
  await expect(notice(page)).toContainText("Syntax highlighting is unavailable");
  await page.getByRole("checkbox", { name: "Mark reviewed", exact: true }).click();
  await ready(page);
  await expect(
    page.getByRole("checkbox", { name: "Mark reviewed: config/settings.json", exact: true }),
  ).toBeChecked();
  await open(page, "src/web/session.ts");
  await expect(notice(page)).toHaveCount(0);
  await expect(page.locator(".jj-diff-cell .jj-token-Keyword").first()).toBeVisible();
});
