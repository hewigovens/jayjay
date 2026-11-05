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
    state = usePullReview(client, labels, target);
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
