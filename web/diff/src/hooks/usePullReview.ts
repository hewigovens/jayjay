import { useEffect, useMemo, useRef, useState } from "react";
import { useReview } from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels, ReviewBackend } from "@jayjay/review-web/types";
import { GitHubSignIn } from "../github/GitHubSignIn";
import { notesExport } from "../github/NotesExport";
import { PullLoader, PullLoadError, type FileContent } from "../github/PullLoader";
import type { PullCommit, PullStackEntry } from "../github/PullManifest";
import { isPullRequest, reviewName, reviewPath, type ReviewRef } from "../github/ReviewRef";
import { ViewedSync } from "../github/ViewedSync";

export function usePullReview(client: LocalReviewClient, labels: Labels, pull: ReviewRef) {
  const path = reviewPath({ ...pull, commit: undefined });
  // Read on each open, so picking a commit reopens the review in place and keeps the last one on screen until it is ready.
  const wanted = useRef(pull.commit);
  wanted.current = pull.commit;
  // The commit each opened session shows, "" for the whole pull request.
  const opened = useRef(new Map<number, string>());
  const [commits, setCommits] = useState<PullCommit[]>([]);
  const [stack, setStack] = useState<PullStackEntry[]>();
  const prefetch = useRef<AbortController>(undefined);
  const headSha = useRef("");
  const [signedIn, setSignedIn] = useState(() => GitHubSignIn.token !== null);
  const [needsSignIn, setNeedsSignIn] = useState(false);
  const [syncing, setSyncing] = useState(() => GitHubSignIn.canWrite);
  const [syncFailed, setSyncFailed] = useState(false);
  const sync = useRef<ViewedSync | null>(null);
  // Set when the reader turns syncing on, so files they reviewed before then get checked on GitHub too.
  const pushReviewed = useRef(false);
  useEffect(() => () => prefetch.current?.abort(), []);

  function signOut() {
    GitHubSignIn.signOut();
    setSignedIn(false);
    setSyncing(false);
  }

  const backend = useMemo<ReviewBackend>(() => {
    let loader: PullLoader | undefined;
    let session = 0;
    let supplied = new Map<number, Promise<void>>();

    // The review renders or marks a file only after its content has been handed over.
    function supply(index: number) {
      let pending = supplied.get(index);
      if (!pending && loader) {
        const opened = session;
        pending = loader
          .content(index)
          .then(({ before, after }) => client.load(opened, index, before, after));
        supplied.set(index, pending);
        pending.catch(() => supplied.delete(index));
      }
      return pending;
    }

    async function localized<T>(work: () => Promise<T>) {
      try {
        return await work();
      } catch (error) {
        if (!(error instanceof PullLoadError)) throw error;
        // A revoked sign-in falls back to the site's limit until the reader signs in again.
        if (error.code === "unauthorized") signOut();
        setNeedsSignIn(error.code === "rate-limited" || error.code === "unauthorized");
        throw new Error(labels[`pullError.${error.code}`]);
      }
    }

    return {
      open: (signal) =>
        localized(async () => {
          prefetch.current?.abort();
          const commit = wanted.current;
          const next = await PullLoader.open({ ...pull, commit }, GitHubSignIn.token, signal);
          const first = new Map<number, FileContent>();
          if (next.manifest.files.length) first.set(0, await next.content(0));
          // The worker holds one session, which an abandoned open must not replace.
          signal?.throwIfAborted();
          const result = await client.open(next.bundle(first));
          loader = next;
          session = result.session;
          supplied = new Map([[0, Promise.resolve()]]);
          opened.current.set(session, commit ?? "");
          headSha.current = next.manifest.headSha;
          setCommits(next.manifest.commits);
          // Only the whole pull request's manifest follows the stack; a commit keeps what it showed.
          if (!commit) setStack(next.manifest.stack);
          const controller = new AbortController();
          prefetch.current = controller;
          void loader.prefetch(supply, controller.signal);
          return result;
        }),
      render: (selection) =>
        localized(async () => {
          await supply(selection.file);
          return client.render(selection);
        }),
      mutate: (selection, action) =>
        localized(async () => {
          if ("file" in action) await supply(action.file);
          const result = await client.mutate(selection, action);
          const file =
            (action.kind === "setFile" || action.kind === "setGroup") &&
            result.overview.files[action.file];
          if (file)
            sync.current
              ?.set(file.path, file.review === "reviewed")
              .catch(() => setSyncFailed(true));
          return result;
        }),
      image: async (selection, side) =>
        (await loader?.content(selection.file))?.images?.[side] ?? null,
    };
    // The path identifies the pull request; a new `pull` object for the same path must not reload it.
  }, [client, labels, path]);
  const review = useReview(backend);
  const latest = useRef(review);
  latest.current = review;
  const session = review.result?.session;
  const shown = session === undefined ? undefined : opened.current.get(session);

  // A failed open of the same commit waits for Retry rather than looping.
  const asked = useRef(pull.commit);
  useEffect(() => {
    if (review.pending || shown === undefined || shown === (pull.commit ?? "")) return;
    if (review.error && asked.current === pull.commit) return;
    asked.current = pull.commit;
    void review.run((signal) => backend.open(signal));
  }, [pull.commit, shown, review.pending]);

  // Unchecking a Viewed box on GitHub leaves the local mark alone.
  useEffect(() => {
    const token = GitHubSignIn.token;
    sync.current = null;
    // GitHub's Viewed boxes belong to the whole pull request, not one commit.
    if (!syncing || shown !== "" || !token || !isPullRequest(pull)) return;
    const controller = new AbortController();
    void (async () => {
      const loaded = await ViewedSync.load(pull, token, controller.signal);
      if (controller.signal.aborted) return;
      sync.current = loaded;
      const push = pushReviewed.current;
      pushReviewed.current = false;
      for (const [index, file] of (latest.current.result?.overview.files ?? []).entries()) {
        if (controller.signal.aborted) return;
        if (loaded.isViewed(file.path)) {
          if (file.review !== "reviewed")
            await latest.current.mutate({ kind: "setFile", file: index, reviewed: true });
        } else if (push && file.review === "reviewed") await loaded.set(file.path, true);
      }
    })().catch(() => {
      if (!controller.signal.aborted) setSyncFailed(true);
    });
    return () => controller.abort();
  }, [syncing, session]);
  useEffect(() => {
    document.title = `${reviewName(pull)} · JayJay Diff`;
  }, [path]);

  return {
    review,
    shown,
    commits,
    stack,
    signedIn,
    needsSignIn,
    syncing,
    syncFailed,
    signOut,
    notes: () =>
      review.result && notesExport(review.result.overview, reviewPath(pull), headSha.current),
    onSignedIn: () => {
      setSignedIn(true);
      setNeedsSignIn(false);
      review.retry();
    },
    enableSync: () => {
      pushReviewed.current = true;
      setSyncFailed(false);
      setSignedIn(true);
      setSyncing(true);
    },
  };
}
