import { useRef, useState } from "react";
import {
  ReviewWorkspace,
  ToolButton,
  ToolLink,
  ShortcutsDialog,
  KeyboardIcon,
} from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { gitHubUrl, isPullRequest, type ReviewRef } from "../../github/ReviewRef";
import { usePullReview } from "../../hooks/usePullReview";
import { GitHubIcon, RefreshIcon, SignOutIcon } from "../icons";
import { SignInPanel } from "../sign-in/SignInPanel";
import { CommitPicker } from "./CommitPicker";
import { ShareNotesButton } from "./ShareNotesButton";
import { StackPicker } from "./StackPicker";
import { SyncViewedButton } from "./SyncViewedButton";

export function PullReview({
  client,
  labels,
  pull,
  onOpen,
}: {
  client: LocalReviewClient;
  labels: Labels;
  pull: ReviewRef;
  onOpen: (pull: ReviewRef) => void;
}) {
  const {
    review,
    refresh,
    shown,
    commits,
    stack,
    signedIn,
    needsSignIn,
    syncing,
    syncFailed,
    signOut,
    notes,
    onSignedIn,
    enableSync,
  } = usePullReview(client, labels, pull);
  const [shortcuts, setShortcuts] = useState(false);
  const picked = useRef<"commits" | "stack">(undefined);

  return (
    <>
      {review.result && (
        <nav className="pull-app-bar">
          <a href="/" className="pull-home">
            <img src="/favicon.svg" alt="" width="20" height="20" />
            JayJay Diff
          </a>
          <div className="pull-app-actions">
            <ToolButton
              label={labels.shortcuts}
              aria-keyshortcuts="?"
              onClick={() => setShortcuts(true)}
            >
              <KeyboardIcon />
            </ToolButton>
            {!review.error && (
              <ToolButton
                label={labels.pullRefresh}
                disabled={review.pending}
                onClick={() => void refresh()}
              >
                <RefreshIcon />
              </ToolButton>
            )}
            <ShareNotesButton labels={labels} notes={notes} />
            {shown === "" && isPullRequest(pull) && (
              <SyncViewedButton
                labels={labels}
                syncing={syncing}
                failed={syncFailed}
                onEnabled={enableSync}
              />
            )}
            {signedIn && (
              <ToolButton label={labels.pullSignOut} onClick={signOut}>
                <SignOutIcon />
              </ToolButton>
            )}
            <ToolLink href={gitHubUrl(pull)} label={labels.pullOnGitHub}>
              <GitHubIcon />
            </ToolLink>
          </div>
        </nav>
      )}
      {shortcuts && <ShortcutsDialog labels={labels} onClose={() => setShortcuts(false)} />}
      <ReviewWorkspace
        key={shown ?? ""}
        review={review}
        hideChangeId={isPullRequest(pull) && shown === ""}
        onShowShortcuts={() => setShortcuts(true)}
        labels={labels}
        brand={
          <a className="pull-brand" href="/" aria-label="JayJay Diff" title="JayJay Diff">
            <img src="/favicon.svg" alt="" width="28" height="28" />
          </a>
        }
        avatars
        fill
        onRetry={() => (review.error === "worker-stopped" ? location.reload() : void refresh())}
        errorActions={
          needsSignIn && !signedIn && <SignInPanel labels={labels} onSignedIn={onSignedIn} />
        }
        headerActions={
          <>
            {commits.length > 1 && (
              <CommitPicker
                commits={commits}
                selected={pull.commit}
                labels={labels}
                autoFocus={picked.current === "commits"}
                onSelect={(sha) => {
                  picked.current = "commits";
                  onOpen({ ...pull, commit: sha });
                }}
              />
            )}
            {stack && isPullRequest(pull) && (
              <StackPicker
                stack={stack}
                number={pull.number}
                labels={labels}
                autoFocus={picked.current === "stack"}
                onSelect={(number) => {
                  picked.current = "stack";
                  onOpen({ owner: pull.owner, repo: pull.repo, number });
                }}
              />
            )}
            {review.error && !review.result && (
              <a className="pull-error-link" href={gitHubUrl(pull)}>
                {labels.pullOnGitHub}
              </a>
            )}
          </>
        }
      />
    </>
  );
}
