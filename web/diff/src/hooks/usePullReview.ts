import { useEffect, useMemo, useRef, useState } from "react";
import { useReview } from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels, ReviewBackend } from "@jayjay/review-web/types";
import { GitHubSignIn } from "../github/GitHubSignIn";
import { notesExport } from "../github/NotesExport";
import { PullLoader, PullLoadError } from "../github/PullLoader";
import type { PullCommit, PullError, PullStackEntry } from "../github/PullManifest";
import {
  atCommit,
  isPullRequest,
  reviewKind,
  reviewName,
  reviewPath,
  type ReviewRef,
} from "../github/ReviewRef";
import { AppRestrictedError, ViewedSync } from "../github/ViewedSync";

export type SyncFailure = "failed" | "restricted";

export function usePullReview(client: LocalReviewClient, labels: Labels, pull: ReviewRef) {
  const path = reviewPath({ ...pull, commit: undefined });
  // Read on each open, so picking a commit reopens the review in place and keeps the last one on screen until it is ready.
  const wanted = useRef(pull.commit);
  wanted.current = pull.commit;
  const opened = useRef(new Map<number, { target: ReviewRef; headSha: string; title: string }>());
  const [commits, setCommits] = useState<PullCommit[]>([]);
  const [stack, setStack] = useState<PullStackEntry[]>();
  const prefetch = useRef<AbortController>(undefined);
  const [signedIn, setSignedIn] = useState(() => GitHubSignIn.token !== null);
  const [needsSignIn, setNeedsSignIn] = useState(false);
  const [failure, setFailure] = useState<PullError>();
  const [syncing, setSyncing] = useState(() => GitHubSignIn.canWrite);
  const [syncFailure, setSyncFailure] = useState<SyncFailure>();
  const sync = useRef<ViewedSync | null>(null);
  // Set when the reader turns syncing on, so files they reviewed before then get checked on GitHub too.
  const pushReviewed = useRef(false);
  useEffect(() => () => prefetch.current?.abort(), []);

  function syncFailed(error: unknown) {
    setSyncFailure(error instanceof AppRestrictedError ? "restricted" : "failed");
  }

  function signOut() {
    GitHubSignIn.signOut();
    setSignedIn(false);
    setSyncing(false);
  }

  const backend = useMemo(() => {
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

    function prefetchAround(index: number) {
      prefetch.current?.abort();
      if (!loader) return;
      const controller = new AbortController();
      prefetch.current = controller;
      void loader.prefetch(index, supply, controller.signal);
    }

    async function localized<T>(work: () => Promise<T>) {
      try {
        const result = await work();
        setFailure(undefined);
        return result;
      } catch (error) {
        if (!(error instanceof PullLoadError)) throw error;
        // A revoked sign-in falls back to the site's limit until the reader signs in again.
        if (error.code === "unauthorized") signOut();
        setNeedsSignIn(error.code === "rate-limited" || error.code === "unauthorized");
        setFailure(error.code);
        const kind = reviewKind(atCommit(pull, wanted.current));
        throw new Error(labels[`${kind}Error.${error.code}`] ?? labels[`pullError.${error.code}`]);
      }
    }

    return {
      open: (signal?: AbortSignal, fresh = false) =>
        localized(async () => {
          prefetch.current?.abort();
          const commit = wanted.current;
          const target = atCommit(pull, commit);
          const next = await PullLoader.open(target, GitHubSignIn.token, signal, undefined, fresh);
          // The worker holds one session, which an abandoned open must not replace.
          signal?.throwIfAborted();
          const result = await client.open(next.bundle());
          loader = next;
          session = result.session;
          supplied = new Map();
          opened.current.set(session, {
            target,
            headSha: next.manifest.headSha,
            title: next.manifest.title,
          });
          setCommits(next.manifest.commits);
          // Only the whole pull request's manifest follows the stack; a commit keeps what it showed.
          if (!commit) setStack(next.manifest.stack);
          return result;
        }),
      render: (selection) =>
        localized(async () => {
          prefetchAround(selection.file);
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
              .then((written) => written && setSyncFailure(undefined), syncFailed);
          return result;
        }),
      image: async (selection, side) =>
        (await loader?.content(selection.file))?.images?.[side] ?? null,
    } satisfies ReviewBackend;
    // The path identifies the pull request; a new `pull` object for the same path must not reload it.
  }, [client, labels, path]);
  const review = useReview(backend);
  const latest = useRef(review);
  latest.current = review;
  const session = review.result?.session;
  const displayed = session === undefined ? undefined : opened.current.get(session);
  const shown = displayed && (displayed.target.commit ?? "");

  function refresh() {
    const { result, selected, options } = review;
    const path = result?.overview.files[selected]?.path;
    return review.run(async (signal) => {
      const next = await backend.open(signal, true);
      if (!next.overview.files.length) return { ...next, options };
      const file = Math.max(
        0,
        next.overview.files.findIndex((entry) => entry.path === path),
      );
      return backend.render({ session: next.session, file, options });
    });
  }

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
    })().catch((error: unknown) => {
      if (!controller.signal.aborted) syncFailed(error);
    });
    return () => controller.abort();
  }, [syncing, session]);
  useEffect(() => {
    document.title = `${reviewName(pull)} · JayJay Diff`;
  }, [path]);

  return {
    review,
    refresh,
    shown,
    commits,
    stack,
    signedIn,
    needsSignIn,
    // Retrying can't make a missing, private, or oversized review appear.
    retryable: failure !== "not-found" && failure !== "too-large",
    syncing,
    syncFailure,
    signOut,
    notes: () =>
      review.result && displayed
        ? notesExport(
            review.result.overview,
            reviewPath(displayed.target),
            displayed.headSha,
            displayed.title,
          )
        : null,
    onSignedIn: () => {
      setSignedIn(true);
      setNeedsSignIn(false);
      void refresh();
    },
    enableSync: () => {
      pushReviewed.current = true;
      setSyncFailure(undefined);
      setSignedIn(true);
      setSyncing(true);
    },
  };
}
