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

async function open(blobs: Record<string, BodyInit>, files: PullFile[], requests: string[] = []) {
  const manifest: PullManifest = {
    repository: "o/r",
    number: 7,
    title: "T",
    body: "B",
    author: { login: "u", id: 9 },
    baseSha: "base",
    headSha: "head",
    files,
    commits: [],
  };
  return PullLoader.open({ owner: "o", repo: "r", number: 7 }, null, undefined, async (input) => {
    const url = String(input);
    if (url === "/api/o/r/pull/7") return Response.json(manifest);
    requests.push(url);
    const key = url.replace("https://raw.githubusercontent.com/o/r/", "");
    return key in blobs ? new Response(blobs[key]) : new Response("", { status: 404 });
  });
}

test("opens a bundle from the manifest and loads each file's sides on request", async () => {
  const requests: string[] = [];
  const loader = await open(
    {
      "base/old%20name.rs": "fn a() {}\n",
      "head/src/new%20name.rs": "fn b() {}\n",
      "base/gone.txt": "bye\n",
      "head/logo.png": png,
      "base/logo.png": png,
    },
    [
      file("src/new name.rs", "renamed", { oldPath: "old name.rs" }),
      file("gone.txt", "removed", { sha: null }),
      file("logo.png", "modified"),
      file("submodule", "modified"),
    ],
    requests,
  );

  const first = new Map([[0, await loader.content(0)]]);
  const bundle = JSON.parse(loader.bundle(first)) as ReviewBundle;
  expect(bundle).toMatchObject({
    repositoryId: "github.com/o/r",
    changeId: "pull/7",
    title: "T",
    description: "B",
    author: "u",
    authorEmail: "9+u@users.noreply.github.com",
  });
  expect(bundle.files[0]).toMatchObject({
    path: "src/new name.rs",
    oldPath: "old name.rs",
    kind: "renamed",
    before: "fn a() {}\n",
    after: "fn b() {}\n",
  });
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

test("prefetches only the files nobody asked for yet", async () => {
  const loader = await open({ "base/a": "1", "head/a": "2", "base/b": "3", "head/b": "4" }, [
    file("a", "modified"),
    file("b", "modified"),
  ]);
  await loader.content(0);
  const loaded: number[] = [];
  await loader.prefetch((index) => loaded.push(index), new AbortController().signal);
  expect(loaded).toEqual([1]);
});
