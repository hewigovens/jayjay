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

test.each(["pull", "compare"])(
  "sharing a %s follows the displayed session while another commit loads or fails",
  async (kind) => {
    vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
    const first = "a".repeat(40);
    const second = "b".repeat(40);
    let finish: (response: Response) => void = () => {};
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(
        Response.json({
          files: [],
          commits: [],
          headSha: first,
        }),
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
    const labels = { "pullError.unavailable": "Unavailable" } as Labels;
    let state: ReturnType<typeof usePullReview>;
    function Host({ target }: { target: ReviewRef }) {
      state = usePullReview(client, labels, target);
      return null;
    }
    const root = createRoot(document.createElement("div"));
    const target: ReviewRef =
      kind === "pull"
        ? { owner: "o", repo: "r", number: 7, commit: first }
        : { owner: "o", repo: "r", base: "main", head: "feature/x", commit: first };
    const path = kind === "pull" ? "/o/r/pull/7" : "/o/r/compare/main...feature/x";
    try {
      await act(async () => {
        root.render(createElement(Host, { target }));
      });
      expect(state!.notes()).toMatchObject({
        review: `${path}/commits/${first}`,
        headSha: first,
        title: "Review 1",
      });
      await act(async () => {
        root.render(createElement(Host, { target: { ...target, commit: second } }));
      });
      expect(state!.review.pending).toBe(true);
      expect(state!.notes()).toMatchObject({
        review: `${path}/commits/${first}`,
        headSha: first,
        title: "Review 1",
      });
      await act(async () => {
        finish(Response.json({ error: "unavailable" }, { status: 502 }));
      });
      expect(state!.review.error).toBe("Unavailable");
      expect(state!.notes()).toMatchObject({
        review: `${path}/commits/${first}`,
        headSha: first,
        title: "Review 1",
      });
      await act(async () => {
        root.render(createElement(Host, { target: { ...target, commit: undefined } }));
      });
      await act(async () => {
        finish(Response.json({ files: [], commits: [], headSha: second }));
      });
      expect(state!.notes()).toMatchObject({
        review: path,
        headSha: second,
        title: "Review 2",
      });
    } finally {
      await act(async () => {
        root.unmount();
      });
      vi.unstubAllGlobals();
    }
  },
);
