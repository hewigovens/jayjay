import { expect, test } from "vitest";
import { GitHub, GitHubError } from "../worker/github/GitHub";
import { diffFiles } from "../worker/github/GitHubDiff";

function github(routes: Record<string, unknown>, requests: string[] = []) {
  return new GitHub(undefined, async (input) => {
    const path = String(input).replace("https://api.github.com", "");
    requests.push(path);
    if (!(path in routes)) return new Response("{}", { status: 404 });
    return typeof routes[path] === "string"
      ? new Response(routes[path])
      : Response.json(routes[path]);
  });
}

const modified = (path: string, lines = "-a\n+b\n") =>
  `diff --git a/${path} b/${path}\nindex 1111111..2222222 100644\n--- a/${path}\n+++ b/${path}\n@@ -1 +1 @@\n${lines}`;

test("diffs against the merge base and reads the files from GitHub's diff", async () => {
  const requests: string[] = [];
  const manifest = await github(
    {
      "/repos/o/r/pulls/7": {
        title: "T",
        body: null,
        user: { login: "u", id: 9 },
        created_at: "2026-10-31T12:00:00Z",
        changed_files: 2,
        commits: 1,
        base: { sha: "tip", repo: { private: false } },
        head: { sha: "head" },
      },
      "/repos/o/r/pulls/7/commits?per_page=100&page=1": [
        {
          sha: "head",
          commit: {
            message: "Fix it\n\nBecause",
            author: { name: "U", date: "2026-11-01T00:00:00Z" },
          },
          author: null,
          parents: [{ sha: "tip" }],
        },
      ],
      "/repos/o/r/compare/tip...head?per_page=1": { merge_base_commit: { sha: "base" } },
      "https://github.com/o/r/pull/7.diff": modified("a.rs") + modified("b.rs"),
    },
    requests,
  ).manifest({ owner: "o", repo: "r", number: 7 });

  expect(manifest).toMatchObject({
    repository: "o/r",
    title: "T",
    timestampMillis: Date.parse("2026-10-31T12:00:00Z"),
    body: "",
    author: { login: "u", id: 9 },
    baseSha: "base",
    headSha: "head",
  });
  expect(manifest.files.map((file) => file.path)).toEqual(["a.rs", "b.rs"]);
  expect(manifest.commits).toEqual([
    {
      sha: "head",
      title: "Fix it",
      author: "U",
      timestampMillis: Date.parse("2026-11-01T00:00:00Z"),
    },
  ]);
  expect(requests).toHaveLength(4);
});

test("reads each file's kind, paths, line counts and blob IDs from a diff", () => {
  const files = diffFiles(
    [
      modified("src/lib.rs", "@@ -5,3 +5,4 @@\n context\n-old\n+new\n+--- not a header\n"),
      "diff --git a/old name.rs b/new name.rs\nsimilarity index 90%\nrename from old name.rs\nrename to new name.rs\nindex 3333333..4444444 100644\n--- a/old name.rs\n+++ b/new name.rs\n@@ -1 +1 @@\n-x\n+y\n",
      "diff --git a/moved.rs b/elsewhere/moved.rs\nsimilarity index 100%\nrename from moved.rs\nrename to elsewhere/moved.rs\n",
      "diff --git a/added.txt b/added.txt\nnew file mode 100644\nindex 0000000..5555555\n--- /dev/null\n+++ b/added.txt\n@@ -0,0 +1,2 @@\n+one\n+two\n",
      "diff --git a/gone.txt b/gone.txt\ndeleted file mode 100644\nindex 6666666..0000000\n--- a/gone.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-bye\n",
      "diff --git a/logo.png b/logo.png\nindex 7777777..8888888 100644\nBinary files a/logo.png and b/logo.png differ\n",
      'diff --git "a/caf\\303\\251.md" "b/caf\\303\\251.md"\nindex 9999999..aaaaaaa 100644\n',
      "diff --git a/run.sh b/run.sh\nold mode 100644\nnew mode 100755\n",
    ].join(""),
  );

  expect(files).toEqual([
    {
      path: "src/lib.rs",
      kind: "modified",
      added: 2,
      removed: 1,
      oldSha: "1111111",
      sha: "2222222",
    },
    {
      path: "new name.rs",
      oldPath: "old name.rs",
      kind: "renamed",
      added: 1,
      removed: 1,
      oldSha: "3333333",
      sha: "4444444",
    },
    {
      path: "elsewhere/moved.rs",
      oldPath: "moved.rs",
      kind: "renamed",
      added: 0,
      removed: 0,
      sha: null,
    },
    { path: "added.txt", kind: "added", added: 2, removed: 0, sha: "5555555" },
    { path: "gone.txt", kind: "removed", added: 0, removed: 1, oldSha: "6666666", sha: null },
    {
      path: "logo.png",
      kind: "modified",
      added: 0,
      removed: 0,
      oldSha: "7777777",
      sha: "8888888",
      binary: true,
    },
    {
      path: "caf\u00e9.md",
      kind: "modified",
      added: 0,
      removed: 0,
      oldSha: "9999999",
      sha: "aaaaaaa",
    },
    {
      path: "run.sh",
      kind: "modified",
      added: 0,
      removed: 0,
      sha: null,
      oldMode: "100644",
      newMode: "100755",
    },
  ]);
});

test("diffs one of the pull request's commits against its parent", async () => {
  const commit = (sha: string, parent: string, message: string) => ({
    sha,
    commit: { message, author: { name: "U", date: "2026-11-01T00:00:00Z" } },
    author: { login: "u", id: 9 },
    parents: [{ sha: parent }],
  });
  const routes = {
    "/repos/o/r/pulls/7": {
      title: "T",
      body: null,
      user: null,
      changed_files: 1,
      commits: 2,
      base: { sha: "tip", repo: { private: false } },
      head: { sha: "two" },
    },
    "/repos/o/r/pulls/7/commits?per_page=100&page=1": [
      commit("one", "tip", "First"),
      commit("two", "one", "Second\n\nWhy\nit matters"),
    ],
    "https://github.com/o/r/commit/two.diff": modified("a.rs"),
  };
  const manifest = await github(routes).manifest({
    owner: "o",
    repo: "r",
    number: 7,
    commit: "two",
  });

  expect(manifest).toMatchObject({
    title: "Second",
    body: "Why\nit matters",
    author: { login: "u", id: 9 },
    baseSha: "one",
    headSha: "two",
    commit: "two",
  });
  expect(manifest.files.map((file) => file.path)).toEqual(["a.rs"]);
  expect(manifest.commits.map((item) => item.sha)).toEqual(["one", "two"]);
  await expect(
    github(routes).manifest({ owner: "o", repo: "r", number: 7, commit: "elsewhere" }),
  ).rejects.toMatchObject({ code: "not-found" });
});

test("compares a branch with the default branch from their merge base", async () => {
  const commit = (sha: string, parent: string, message: string) => ({
    sha,
    commit: { message, author: { name: "U", date: "2026-11-01T00:00:00Z" } },
    author: null,
    parents: [{ sha: parent }],
  });
  const routes = {
    "/repos/o/r": { private: false, default_branch: "main" },
    "/repos/o/r/compare/main...feature/x?per_page=100&page=1": {
      total_commits: 2,
      merge_base_commit: { sha: "base" },
      commits: [commit("one", "base", "First"), commit("two", "one", "Second")],
    },
    "https://github.com/o/r/compare/main...feature/x.diff": modified("a.rs"),
    "https://github.com/o/r/commit/one.diff": modified("b.rs"),
  };
  const manifest = await github(routes).manifest({ owner: "o", repo: "r", head: "feature/x" });
  expect(manifest).toMatchObject({
    repository: "o/r",
    range: "main...feature/x",
    title: "main...feature/x",
    baseSha: "base",
    headSha: "two",
    author: null,
  });
  expect(manifest.files.map((file) => file.path)).toEqual(["a.rs"]);
  expect(manifest.commits.map((item) => item.sha)).toEqual(["one", "two"]);

  const first = await github(routes).manifest({
    owner: "o",
    repo: "r",
    base: "main",
    head: "feature/x",
    commit: "one",
  });
  expect(first).toMatchObject({
    range: "main...feature/x",
    title: "First",
    baseSha: "base",
    headSha: "one",
    commit: "one",
  });
  expect(first.files.map((file) => file.path)).toEqual(["b.rs"]);
  await expect(
    github({ ...routes, "/repos/o/r": { private: true, default_branch: "main" } }).manifest({
      owner: "o",
      repo: "r",
      head: "feature/x",
    }),
  ).rejects.toMatchObject({ code: "not-found" });
});

test("diffs a single commit against its first parent, and a fork's branch from the merge base", async () => {
  const routes = {
    "/repos/o/r": { private: false, default_branch: "main" },
    "/repos/o/r/commits/09f2cf7": {
      sha: "09f2cf76eccb",
      commit: { message: "Fix it\n\nBecause", author: { name: "U", date: "2026-11-01T00:00:00Z" } },
      author: null,
      parents: [{ sha: "parent" }],
    },
    "https://github.com/o/r/commit/09f2cf76eccb.diff": modified("a.rs"),
    "/repos/o/r/compare/main...user%3Afeature?per_page=100&page=1": {
      total_commits: 0,
      merge_base_commit: { sha: "base" },
      commits: [],
    },
    "https://github.com/o/r/compare/main...user%3Afeature.diff": modified("b.rs"),
  };
  const commit = await github(routes).manifest({ owner: "o", repo: "r", sha: "09f2cf7" });
  expect(commit).toMatchObject({
    title: "Fix it",
    body: "Because",
    baseSha: "parent",
    headSha: "09f2cf76eccb",
    commit: "09f2cf76eccb",
  });
  expect(commit.files.map((file) => file.path)).toEqual(["a.rs"]);
  const fork = await github(routes).manifest({ owner: "o", repo: "r", head: "user:feature" });
  expect(fork).toMatchObject({ range: "main...user:feature", baseSha: "base" });
  expect(fork.files.map((file) => file.path)).toEqual(["b.rs"]);
  await expect(
    github({ ...routes, "/repos/o/r": { private: true, default_branch: "main" } }).manifest({
      owner: "o",
      repo: "r",
      sha: "09f2cf7",
    }),
  ).rejects.toMatchObject({ code: "not-found" });
});

test("maps GitHub failures to the errors the page explains", async () => {
  const pull = { owner: "o", repo: "r", number: 1 };
  const failing = (response: Response) =>
    new GitHub(undefined, async () => response).manifest(pull);
  await expect(github({}).manifest(pull)).rejects.toMatchObject({ code: "not-found" });
  await expect(
    failing(new Response("", { status: 403, headers: { "x-ratelimit-remaining": "0" } })),
  ).rejects.toMatchObject({ code: "rate-limited" });
  await expect(failing(new Response("", { status: 500 }))).rejects.toBeInstanceOf(GitHubError);
  await expect(failing(new Response("", { status: 401 }))).rejects.toMatchObject({
    code: "unauthorized",
  });
  const pullWith = (fields: object) =>
    github({
      "/repos/o/r/pulls/1": {
        changed_files: 1,
        base: { sha: "a", repo: { private: false } },
        head: { sha: "b" },
        ...fields,
      },
      "https://github.com/o/r/pull/1.diff": "",
    }).manifest(pull);
  await expect(pullWith({ changed_files: 2001 })).rejects.toMatchObject({ code: "too-large" });
  await expect(pullWith({ base: { sha: "a", repo: { private: true } } })).rejects.toMatchObject({
    code: "not-found",
  });
});

test("follows a stack through base branches both ways", async () => {
  const pull = (number: number, base: string, head: string, extra: object = {}) => ({
    number,
    title: `P${number}`,
    state: "open",
    merged_at: null,
    user: null,
    body: null,
    changed_files: 0,
    commits: 0,
    base: { ref: base, sha: "s", repo: { private: false, default_branch: "main" } },
    head: { ref: head, sha: "h", repo: { full_name: "o/r" } },
    ...extra,
  });
  const manifest = await github({
    "/repos/o/r/pulls/2": pull(2, "one", "two"),
    "https://github.com/o/r/pull/2.diff": "",
    "/repos/o/r/compare/s...h?per_page=1": { merge_base_commit: { sha: "s" } },
    "/repos/o/r/pulls?state=all&head=o%3Aone&per_page=2": [
      pull(1, "main", "one", { state: "closed", merged_at: "2026-11-01T00:00:00Z" }),
    ],
    "/repos/o/r/pulls?state=open&base=two&per_page=2": [pull(3, "two", "three")],
    "/repos/o/r/pulls?state=open&base=three&per_page=2": [
      pull(4, "three", "four"),
      pull(5, "three", "five"),
    ],
  }).manifest({ owner: "o", repo: "r", number: 2 });

  expect(manifest.stack).toEqual([
    { number: 1, title: "P1", state: "merged" },
    { number: 2, title: "P2", state: "open" },
    { number: 3, title: "P3", state: "open" },
  ]);
});
