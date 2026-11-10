import { expect, test } from "vitest";
import { Exports } from "../worker/Exports";
import { maxExportBytes, type NotesExport } from "../src/github/NotesExport";

const body: NotesExport = {
  review: "/o/r/pull/7",
  title: "Fix it",
  headSha: "abc",
  notes: [
    {
      path: "src/a.rs",
      side: "new",
      line: 3,
      status: "current",
      excerpt: "let x = 1;",
      body: "Why one?\nCheck the caller.",
    },
    { path: "src/a.rs", side: "old", line: 9, status: "stale", excerpt: "x()", body: "Gone?" },
  ],
  files: [
    { path: "src/a.rs", review: "reviewed" },
    { path: "b.md", review: "changed_since_review" },
    { path: "c.md", review: "unreviewed" },
  ],
};

function exports(allow: (key: string) => boolean = () => true) {
  const stored = new Map<string, string>();
  const kv = {
    put: async (key: string, value: string) => void stored.set(key, value),
    get: async (key: string) => stored.get(key) ?? null,
  };
  return new Exports(kv, { limit: async ({ key }) => ({ success: allow(key) }) });
}

test("stores an export under an unguessable link that serves Markdown and JSON", async () => {
  const store = exports();
  const created = await store.create(
    new Request("https://diff.hewig.dev/api/exports", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  );
  expect(created.status).toBe(201);
  const { url } = (await created.json()) as { url: string };
  const id = /\/api\/exports\/([0-9a-f]{32})$/.exec(url)![1];

  const markdown = await (await store.read(id)).text();
  expect(markdown).toContain("# Review notes for o/r#7: Fix it");
  expect(markdown).toContain(
    "### src/a.rs\n\n- new line 3: `let x = 1;`\n  > Why one?\n  > Check the caller.",
  );
  expect(markdown).toContain("- old line 9 (stale): `x()`");
  expect(markdown).toContain("1 of 3 files reviewed");
  expect(markdown).toContain("- src/a.rs: reviewed\n- b.md: changed since review");
  expect(await (await store.read(`${id}.json`)).json()).toEqual(body);
  expect((await store.read("0".repeat(32))).status).toBe(404);
});

test("refuses bodies that are not an export or are too large", async () => {
  const post = (text: string) =>
    exports().create(new Request("https://x/api/exports", { method: "POST", body: text }));
  expect((await post("{}")).status).toBe(400);
  expect((await post(JSON.stringify({ ...body, review: "/../etc/pull/1" }))).status).toBe(400);
  expect((await post(JSON.stringify({ ...body, title: "x".repeat(1024 * 1024) }))).status).toBe(
    413,
  );
});

test("refuses an oversized body without buffering past the limit", async () => {
  const lying = new Request("https://x/api/exports", {
    method: "POST",
    headers: { "content-length": String(2 * maxExportBytes) },
    body: "{}",
  });
  expect((await exports().create(lying)).status).toBe(413);

  let pulled = 0;
  const chunk = new Uint8Array(64 * 1024);
  const oversized = new ReadableStream<Uint8Array>({
    pull(controller) {
      pulled += chunk.byteLength;
      if (pulled > 4 * maxExportBytes) controller.close();
      else controller.enqueue(chunk);
    },
  });
  const unsized = new Request("https://x/api/exports", {
    method: "POST",
    body: oversized,
    duplex: "half",
  } as RequestInit);
  expect((await exports().create(unsized)).status).toBe(413);
  expect(pulled).toBeLessThan(maxExportBytes + 4 * chunk.byteLength);
});

test("rate-limits exports per client address", async () => {
  const keys: string[] = [];
  const store = exports((key) => keys.push(key) <= 1);
  const post = () =>
    store.create(
      new Request("https://x/api/exports", {
        method: "POST",
        headers: { "cf-connecting-ip": "203.0.113.9" },
        body: JSON.stringify(body),
      }),
    );
  expect((await post()).status).toBe(201);
  expect((await post()).status).toBe(429);
  expect(keys).toEqual(["203.0.113.9", "203.0.113.9"]);
});
