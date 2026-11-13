import { expect, test } from "vitest";
import type { ReviewBundle } from "@jayjay/review-web/types";
import { parsePatch, PatchError } from "../src/Patch";

const mail = `From 1a2b3c4 Mon Sep 17 00:00:00 2001
From: Jay Bird <jay@example.com>
Date: Sun, 1 Nov 2026 00:00:00 +0000
Subject: [PATCH 1/1] Fix the thing

---
`;
const header = (path: string) =>
  `diff --git a/${path} b/${path}\nindex 1111111..2222222 100644\n--- a/${path}\n+++ b/${path}\n`;
const modified = `${header("src/lib.rs")}@@ -3,3 +3,4 @@ fn top() {
 three
-four
+FOUR
+four and a half
 five
@@ -10,2 +11,2 @@
 ten
-eleven
\\ No newline at end of file
+eleven
`;
const added = `diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..3333333
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+one
+two
-- 
2.50.0
`;

async function parse(text: string, name?: string) {
  const patch = await parsePatch(text, "fallback", name);
  return { patch, bundle: JSON.parse(patch.bundle) as ReviewBundle };
}

test("rebuilds each file's two sides around its hunks, at their own line numbers", async () => {
  const { patch, bundle } = await parse(mail + modified + added, "fix.patch");

  expect(patch.title).toBe("Fix the thing");
  expect(bundle).toMatchObject({
    changeId: "fix.patch",
    author: "Jay Bird",
    authorEmail: "jay@example.com",
    timestampMillis: Date.parse("2026-11-01T00:00:00Z"),
    files: [
      { path: "src/lib.rs", kind: "modified", added: 3, removed: 2 },
      { path: "new.txt", kind: "added", added: 2, removed: 0 },
    ],
  });
  expect(patch.bundle).not.toContain("four and a half");
  expect(patch.contents).toEqual([
    {
      before: "⋯\n⋯\nthree\nfour\nfive\n⋯\n⋯\n⋯\n⋯\nten\neleven",
      after: "⋯\n⋯\nthree\nFOUR\nfour and a half\nfive\n⋯\n⋯\n⋯\n⋯\nten\neleven\n",
    },
    { before: "", after: "one\ntwo\n" },
  ]);
});

test("keeps blank-line and line-ending changes apart from the lines a patch omits", async () => {
  const blank = `${header("a.txt")}@@ -2 +1,0 @@\n-\n@@ -5,0 +5 @@\n+\n`;
  const endings = `${header("b.txt")}@@ -1 +1 @@\n-same\r\n+same\n`;
  const saved = (modified + added).replaceAll("\n", "\r\n");

  expect((await parse(blank + endings)).patch.contents).toEqual([
    { before: "⋯\n\n⋯\n⋯\n⋯\n", after: "⋯\n⋯\n⋯\n⋯\n\n" },
    { before: "same\r\n", after: "same\n" },
  ]);
  expect((await parse(saved)).patch.contents).toEqual(
    (await parse(modified + added)).patch.contents,
  );
});

test("keeps a file's review identity until its diff changes", async () => {
  const first = (await parse(modified + added)).bundle;
  const second = (await parse(modified + added.replace("+two", "+2"))).bundle;

  expect(second.changeId).not.toBe(first.changeId);
  expect(second.files[0].reviewIdentity).toBe(first.files[0].reviewIdentity);
  expect(second.files[1].reviewIdentity).not.toBe(first.files[1].reviewIdentity);
});

test("rejects what it can't show faithfully", async () => {
  for (const [text, code] of [
    ["--- a\n+++ b\n", "invalid"],
    [`${header("a")}@@ -1,3 +1,3 @@\n-a\n+b\n`, "invalid"],
    [`${header("a")}@@ -1 +1 @@\nBAD\n`, "invalid"],
    [header("a"), "invalid"],
    [`${header("a")}@@ -1 +1\n`, "invalid"],
    [`${header("a")}@@ -5 +5 @@\n-a\n+b\n@@ -2 +2 @@\n-c\n+d\n`, "invalid"],
    [modified + modified, "series"],
    [`${header("a")}@@ -999999999 +999999999 @@\n-a\n+b\n`, "large"],
  ] as const)
    await expect(parsePatch(text, "t"), text).rejects.toEqual(new PatchError(code));
});
