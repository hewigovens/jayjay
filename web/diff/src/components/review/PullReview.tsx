import { useEffect, useRef, useState } from "react";
import {
  ReviewWorkspace,
  ToolButton,
  ToolLink,
  ShortcutsDialog,
  KeyboardIcon,
  SidebarIcon,
} from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import type { AppearanceSettings } from "../../AppearanceSettings";
import { atCommit, gitHubUrl, isPullRequest, type ReviewRef } from "../../github/ReviewRef";
import { usePullReview } from "../../hooks/usePullReview";
import { useReviewMode } from "../../hooks/useReviewMode";
import { GitHubIcon, RefreshIcon, SignOutIcon } from "../icons";
import { SignInPanel } from "../sign-in/SignInPanel";
import { CommitPicker } from "./CommitPicker";
import { CommitSidebar } from "./CommitSidebar";
import { DiffThemePicker } from "./DiffThemePicker";
import { ShareNotesButton } from "./ShareNotesButton";
import { StackPicker } from "./StackPicker";
import { SyncViewedButton } from "./SyncViewedButton";

const sidebarKey = "jayjay.diff.commitSidebar";

export function PullReview({
  client,
  labels,
  pull,
  onOpen,
  themes,
  selectTheme,
}: {
  themes: AppearanceSettings;
  selectTheme: (themes: AppearanceSettings) => void;
  client: LocalReviewClient;
  labels: Labels;
  pull: ReviewRef;
  onOpen: (pull: ReviewRef) => void;
}) {
  const [reviewMode, toggleReviewMode] = useReviewMode();
  const {
    review,
    refresh,
    shown,
    commits,
    stack,
    signedIn,
    needsSignIn,
    retryable,
    syncing,
    syncFailure,
    signOut,
    notes,
    onSignedIn,
    enableSync,
  } = usePullReview(client, labels, pull, reviewMode);
  const [shortcuts, setShortcuts] = useState(false);
  const [filesSlot, setFilesSlot] = useState<HTMLElement | null>(null);
  const picked = useRef<"commits" | "stack">(undefined);
  const [sidebar, setSidebar] = useState(() => {
    try {
      return localStorage.getItem(sidebarKey) !== "hidden";
    } catch {
      return true;
    }
  });
  function toggleSidebar() {
    setSidebar(!sidebar);
    try {
      localStorage.setItem(sidebarKey, sidebar ? "hidden" : "shown");
    } catch {
      return;
    }
  }
  const openCommit = (sha: string | undefined) => onOpen(atCommit(pull, sha));
  // The picker that opened this review has taken its focus back; a later remount, as on rotation, must not. Skipped while another pull request loads, when no picker is mounted to take it.
  useEffect(() => {
    if (shown !== undefined) picked.current = undefined;
  }, [shown]);

  return (
    <>
      {review.result && (
        <nav className={`pull-app-bar ${sidebar && commits.length > 1 ? "sidebar-open" : ""}`}>
          <div className="pull-app-lead">
            <span className="pull-files-slot" ref={setFilesSlot} />
            {commits.length > 1 && (
              <ToolButton
                className="commit-sidebar-toggle"
                aria-pressed={sidebar}
                label={labels.pullCommits}
                onClick={toggleSidebar}
              >
                <SidebarIcon />
              </ToolButton>
            )}
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
            {reviewMode && shown === "" && isPullRequest(pull) && (
              <SyncViewedButton
                labels={labels}
                syncing={syncing}
                failure={syncFailure}
                onEnabled={enableSync}
              />
            )}
          </div>
          <div className="pull-app-title">
            <ToolLink href={gitHubUrl(pull)} label={labels.pullOnGitHub}>
              <GitHubIcon />
            </ToolLink>
            <a className="pull-repo" href={`https://github.com/${pull.owner}/${pull.repo}`}>
              {pull.owner}/{pull.repo}
            </a>
          </div>
          <div className="pull-app-actions">
            <DiffThemePicker labels={labels} themes={themes} select={selectTheme} />
            <ToolButton
              label={labels.shortcuts}
              aria-keyshortcuts="?"
              onClick={() => setShortcuts(true)}
            >
              <KeyboardIcon />
            </ToolButton>
            {signedIn && (
              <ToolButton label={labels.pullSignOut} onClick={signOut}>
                <SignOutIcon />
              </ToolButton>
            )}
          </div>
        </nav>
      )}
      {shortcuts && <ShortcutsDialog labels={labels} onClose={() => setShortcuts(false)} />}
      <div className="pull-body">
        {review.result && commits.length > 1 && (
          <CommitSidebar
            open={sidebar}
            commits={commits}
            selected={pull.commit}
            labels={labels}
            onSelect={openCommit}
          />
        )}
        <ReviewWorkspace
          key={shown ?? ""}
          review={review}
          diffThemes={themes}
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
          statusBar={false}
          filesSlot={filesSlot}
          filesHeader={
            commits.length > 1 && (
              <CommitPicker
                commits={commits}
                selected={pull.commit}
                labels={labels}
                autoFocus={picked.current === "commits"}
                onSelect={(sha) => {
                  picked.current = "commits";
                  openCommit(sha);
                }}
              />
            )
          }
          reviewMode={reviewMode}
          onReviewMode={toggleReviewMode}
          descriptionLabel={
            isPullRequest(pull) && shown === "" ? labels.pullDescription : undefined
          }
          descriptionFirst={isPullRequest(pull) && !pull.commit}
          onRetry={
            retryable
              ? () => (review.error === "worker-stopped" ? location.reload() : void refresh())
              : null
          }
          errorActions={
            needsSignIn && !signedIn && <SignInPanel labels={labels} onSignedIn={onSignedIn} />
          }
          headerActions={
            <>
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
      </div>
    </>
  );
}
