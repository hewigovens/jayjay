// @vitest-environment jsdom
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import { defaultOptions, type Labels, type ReviewResult } from "@jayjay/review-web/types";
import { usePullReview } from "../src/hooks/usePullReview";
import type { ReviewRef } from "../src/github/ReviewRef";

function result(session: number): ReviewResult {
  return {
    session,
    file: 0,
    diff: null,
    groupStates: [],
    options: defaultOptions,
    elapsed: 0,
    pack: "base",
    syntaxUnavailable: false,
    overview: {
      repositoryId: "github.com/o/r",
      changeId: String(session),
      title: `Review ${session}`,
      description: "",
      author: "",
      files: [],
      tree: [],
      notes: [],
      counts: { files: "0", reviewed: "0", filtered: [] },
    },
  };
}

test("sharing follows the displayed session while another commit loads", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const first = "a".repeat(40);
  const second = "b".repeat(40);
  let finish: (response: Response) => void = () => {};
  const fetcher = vi
    .fn()
    .mockResolvedValueOnce(
      Response.json({ title: "Review 1", files: [], commits: [], headSha: first }),
    )
    .mockImplementation(
      () =>
        new Promise<Response>((resolve) => {
          finish = resolve;
        }),
    );
  vi.stubGlobal("fetch", fetcher);
  let session = 0;
  const client = { open: async () => result(++session) } as unknown as LocalReviewClient;
  const labels = {} as Labels;
  let state: ReturnType<typeof usePullReview>;
  function Host({ target }: { target: ReviewRef }) {
    state = usePullReview(client, labels, target, true);
    return null;
  }
  const root = createRoot(document.createElement("div"));
  const target: ReviewRef = { owner: "o", repo: "r", number: 7, commit: first };
  const firstExport = { review: `/o/r/pull/7/commits/${first}`, headSha: first, title: "Review 1" };
  try {
    await act(async () => {
      root.render(createElement(Host, { target }));
    });
    expect(state!.notes()).toMatchObject(firstExport);
    await act(async () => {
      root.render(createElement(Host, { target: { ...target, commit: undefined } }));
    });
    expect(state!.review.pending).toBe(true);
    expect(state!.notes()).toMatchObject(firstExport);
    await act(async () => {
      finish(Response.json({ title: "Review 2", files: [], commits: [], headSha: second }));
    });
    expect(state!.notes()).toMatchObject({
      review: "/o/r/pull/7",
      headSha: second,
      title: "Review 2",
    });
  } finally {
    await act(async () => {
      root.unmount();
    });
    vi.unstubAllGlobals();
  }
});

test("a commit keeps the pull request's newer commit list until a refresh", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const [x, y, z, w] = ["a", "b", "c", "d"].map((digit) => digit.repeat(40));
  const commits = (...shas: string[]) =>
    shas.map((sha) => ({ sha, title: sha, author: "u", timestampMillis: 0 }));
  const manifest = (...shas: string[]) =>
    Response.json({ title: "T", files: [], commits: commits(...shas), headSha: x });
  const fetcher = vi
    .fn()
    .mockResolvedValueOnce(manifest(x, y, z))
    .mockResolvedValueOnce(manifest(x, y))
    .mockResolvedValueOnce(manifest(x, y, z, w));
  vi.stubGlobal("fetch", fetcher);
  let session = 0;
  const client = { open: async () => result(++session) } as unknown as LocalReviewClient;
  const labels = {} as Labels;
  let state: ReturnType<typeof usePullReview>;
  function Host({ target }: { target: ReviewRef }) {
    state = usePullReview(client, labels, target, true);
    return null;
  }
  const root = createRoot(document.createElement("div"));
  const shas = () => state!.commits.map((commit) => commit.sha);
  try {
    await act(async () => {
      root.render(createElement(Host, { target: { owner: "o", repo: "r", number: 7 } }));
    });
    await act(async () => {
      root.render(createElement(Host, { target: { owner: "o", repo: "r", number: 7, commit: x } }));
    });
    expect(shas()).toEqual([x, y, z]);
    await act(async () => {
      void state!.refresh();
    });
    expect(shas()).toEqual([x, y, z, w]);
  } finally {
    await act(async () => {
      root.unmount();
    });
    vi.unstubAllGlobals();
  }
});
