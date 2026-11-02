import { expect, test } from "vitest";
import { ViewedSync } from "../src/github/ViewedSync";

const pull = { owner: "o", repo: "r", number: 7 };
const page = (nodes: [string, string][], endCursor: string | null) => ({
  data: {
    repository: {
      pullRequest: {
        id: "PR_1",
        files: {
          pageInfo: { hasNextPage: endCursor !== null, endCursor },
          nodes: nodes.map(([path, viewerViewedState]) => ({ path, viewerViewedState })),
        },
      },
    },
  },
});

test("reads Viewed boxes across pages and writes only the ones that differ", async () => {
  const requests: { query: string; variables: Record<string, unknown> }[] = [];
  const replies: unknown[] = [
    page(
      [
        ["a.rs", "VIEWED"],
        ["b.rs", "DISMISSED"],
      ],
      "c1",
    ),
    page([["c.rs", "UNVIEWED"]], null),
    { data: { markFileAsViewed: { clientMutationId: null } } },
    { errors: [{ type: "INSUFFICIENT_SCOPES", message: "no" }] },
  ];
  const sync = await ViewedSync.load(pull, "t", undefined, async (_, init) => {
    requests.push(JSON.parse(String(init?.body)));
    return Response.json(replies.shift());
  });
  expect(["a.rs", "b.rs", "c.rs"].map((path) => sync.isViewed(path))).toEqual([true, false, false]);
  expect(requests[1].variables.after).toBe("c1");

  await sync.set("a.rs", true);
  await sync.set("unknown.rs", true);
  await sync.set("b.rs", true);
  expect(requests).toHaveLength(3);
  expect(requests[2].query).toContain("markFileAsViewed");
  expect(requests[2].variables).toEqual({ id: "PR_1", path: "b.rs" });
  expect(sync.isViewed("b.rs")).toBe(true);

  await expect(sync.set("b.rs", false)).rejects.toThrow("INSUFFICIENT_SCOPES");
  expect(sync.isViewed("b.rs")).toBe(true);
});

test("writes a file's Viewed changes one at a time, in order", async () => {
  const requests: string[] = [];
  const pending: (() => void)[] = [];
  const sync = await ViewedSync.load(pull, "t", undefined, async (_, init) => {
    const { query } = JSON.parse(String(init?.body)) as { query: string };
    if (query.startsWith("query")) return Response.json(page([["a.rs", "UNVIEWED"]], null));
    requests.push(query.includes("unmark") ? "unmark" : "mark");
    await new Promise<void>((resolve) => pending.push(resolve));
    return Response.json({ data: {} });
  });

  const marked = sync.set("a.rs", true);
  const unmarked = sync.set("a.rs", false);
  await new Promise((resolve) => setTimeout(resolve));
  expect(requests).toEqual(["mark"]);
  pending.shift()!();
  await marked;
  await new Promise((resolve) => setTimeout(resolve));
  expect(requests).toEqual(["mark", "unmark"]);
  pending.shift()!();
  await unmarked;
  expect(sync.isViewed("a.rs")).toBe(false);
});
