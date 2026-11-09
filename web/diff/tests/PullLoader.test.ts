import { expect, test } from "vitest";
import type { ReviewBundle } from "@jayjay/review-web/types";
import type { PullFile, PullManifest } from "../src/github/PullManifest";
import { PullLoader } from "../src/github/PullLoader";

const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0, 1]);
const file = (path: string, kind: PullFile["kind"], extra: Partial<PullFile> = {}): PullFile => ({
  path,
  kind,
  added: 1,
  removed: 1,
  sha: `sha-${path}`,
  ...extra,
});

function open(files: PullFile[], raw: (url: string) => Promise<Response>) {
  const manifest: PullManifest = {
    repository: "o/r",
    number: 7,
    title: "T",
    timestampMillis: 1793448000000,
    body: "B",
    author: { login: "u", id: 9 },
    baseSha: "base",
    headSha: "head",
    files,
    commits: [],
  };
  return PullLoader.open({ owner: "o", repo: "r", number: 7 }, null, undefined, async (input) => {
    const url = String(input);
    return url === "/api/o/r/pull/7" ? Response.json(manifest) : raw(url);
  });
}

function blobs(contents: Record<string, BodyInit>, requests: string[]) {
  return async (url: string) => {
    requests.push(url);
    const key = url.replace("https://raw.githubusercontent.com/o/r/", "");
    return key in contents ? new Response(contents[key]) : new Response("", { status: 404 });
  };
}

test("opens a bundle from the manifest and loads each file's sides on request", async () => {
  const requests: string[] = [];
  const loader = await open(
    [
      file("src/new name.rs", "renamed", { oldPath: "old name.rs" }),
      file("gone.txt", "removed", { sha: null }),
      file("logo.png", "modified"),
      file("submodule", "modified"),
    ],
    blobs(
      {
        "base/old%20name.rs": "fn a() {}\n",
        "head/src/new%20name.rs": "fn b() {}\n",
        "base/gone.txt": "bye\n",
        "head/logo.png": png,
        "base/logo.png": png,
      },
      requests,
    ),
  );

  const bundle = JSON.parse(loader.bundle()) as ReviewBundle;
  expect(bundle).toMatchObject({
    repositoryId: "github.com/o/r",
    changeId: "pull/7",
    title: "T #7",
    timestampMillis: 1793448000000,
    description: "B",
    author: "u",
    authorEmail: "9+u@users.noreply.github.com",
  });
  expect(bundle.files[0]).toMatchObject({
    path: "src/new name.rs",
    oldPath: "old name.rs",
    kind: "renamed",
  });
  expect(requests).toHaveLength(0);
  expect(await loader.content(0)).toMatchObject({ before: "fn a() {}\n", after: "fn b() {}\n" });
  expect(bundle.files[1]).toEqual({
    path: "gone.txt",
    reviewIdentity: "removed||base|",
    kind: "removed",
    added: 1,
    removed: 1,
  });
  expect(bundle.files[2].reviewIdentity).toMatch(/^modified\|\|base\|./);
  expect(requests).toHaveLength(2);

  expect(await loader.content(1)).toEqual({ before: "bye\n", after: "", images: null });
  const logo = await loader.content(2);
  expect([logo.before, logo.after, logo.images?.new?.type]).toEqual(["", "", "image/png"]);
  expect(await loader.content(3)).toMatchObject({ before: "", after: "" });
  await loader.content(1);
  expect(requests).toHaveLength(7);
});

test("prefetches the files near the open one, a few at a time, and skips large and binary ones", async () => {
  const files = Array.from({ length: 2000 }, (_, index) =>
    file(
      `f${index}`,
      "modified",
      index === 52 ? { added: 40_000 } : index === 54 ? { added: 0, removed: 0, binary: true } : {},
    ),
  );
  let active = 0;
  let peak = 0;
  const fetched: string[] = [];
  const loader = await open(files, async (url) => {
    fetched.push(url);
    peak = Math.max(peak, ++active);
    await new Promise((resolve) => setTimeout(resolve, 1));
    active--;
    return new Response("x\n");
  });
  await loader.content(51);
  const loaded: number[] = [];
  await loader.prefetch(50, (index) => loaded.push(index), new AbortController().signal);
  expect(loaded.sort((a, b) => a - b)).toEqual([48, 49, 53, 55, 56, 57, 58, 59, 60]);
  expect(fetched).toHaveLength(20);
  expect(await loader.content(54)).toEqual({ before: "", after: "", images: null });
  expect(fetched).toHaveLength(20);
  expect(peak).toBeLessThanOrEqual(8);
});
