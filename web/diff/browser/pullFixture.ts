import { expect, type Page } from "@playwright/test";
import type { PullManifest } from "../src/github/PullManifest";

export const sha = (digit: string) => digit.repeat(40);

export const manifest: PullManifest = {
  repository: "octo/demo",
  number: 7,
  title: "Teach the greeter a second language",
  timestampMillis: 1793448000000,
  body: "",
  author: null,
  baseSha: sha("a"),
  headSha: sha("b"),
  files: [
    { path: "src/greet.rs", kind: "modified", added: 1, removed: 1, sha: "c1", oldSha: "d1" },
  ],
  commits: [
    { sha: sha("e"), title: "Add the greeting", author: "octo", timestampMillis: 0 },
    { sha: sha("b"), title: "Translate the greeting", author: "octo", timestampMillis: 0 },
  ],
  stack: [
    { number: 6, title: "Introduce the greeter", state: "merged" },
    { number: 7, title: "Teach the greeter a second language", state: "open" },
  ],
};

export async function openPull(page: Page) {
  await page.route("**/api/octo/demo/pull/7", (route) => route.fulfill({ json: manifest }));
  await page.route("https://raw.githubusercontent.com/**", (route) =>
    route.fulfill({
      body: route.request().url().includes(sha("a")) ? "fn hello() {}\n" : "fn hola() {}\n",
    }),
  );
  await page.goto("/octo/demo/pull/7");
  await expect(page.getByRole("heading", { name: `${manifest.title} #7` })).toBeVisible();
}
