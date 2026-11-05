import { expect, test } from "vitest";
import { Exports } from "../worker/Exports";
import type { NotesExport } from "../src/github/NotesExport";

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

function exports() {
  const stored = new Map<string, string>();
  const kv = {
    put: async (key: string, value: string) => void stored.set(key, value),
    get: async (key: string) => stored.get(key) ?? null,
  };
  return new Exports(kv);
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

const { review: _review, ...fields } = body;
const legacy = { ...fields, repository: "o/r", number: 7 };

test("serves unexpired legacy exports and accepts posts from older clients", async () => {
  const id = "a".repeat(32);
  const stored = new Exports({
    get: async () => JSON.stringify(legacy),
    put: async () => {},
  });
  expect(await (await stored.read(id)).text()).toContain("https://github.com/o/r/pull/7");
  expect(await (await stored.read(`${id}.json`)).json()).toEqual(legacy);

  const store = exports();
  const response = await store.create(
    new Request("https://x/api/exports", {
      method: "POST",
      body: JSON.stringify(legacy),
    }),
  );
  expect(response.status).toBe(201);
  const { url } = (await response.json()) as { url: string };
  const key = url.split("/").at(-1)!;
  expect(await (await store.read(`${key}.json`)).json()).toEqual(body);
  expect(await (await store.read(key)).text()).toContain("https://github.com/o/r/pull/7");
});

test("rejects invalid legacy review identities", async () => {
  for (const fields of [
    { repository: "o/r/extra", number: 7 },
    { repository: "../r", number: 7 },
    { repository: "o/r#2", number: 7 },
    { repository: "o/r", number: 0 },
    { repository: "o/r", number: "7" },
    { repository: "o/r", number: 1.5 },
  ]) {
    const response = await exports().create(
      new Request("https://x/api/exports", {
        method: "POST",
        body: JSON.stringify({ ...legacy, ...fields }),
      }),
    );
    expect(response.status).toBe(400);
  }
});
