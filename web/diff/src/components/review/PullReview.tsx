import { useEffect, useMemo, useRef, useState } from "react";
import {
  CommitPicker,
  CommitSidebar,
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
import { commitEntries } from "../../github/commitEntries";
import { GitHubSignIn } from "../../github/GitHubSignIn";
import { PullLoader } from "../../github/PullLoader";
import {
  atCommit,
  gitHubUrl,
  isCommit,
  isPullRequest,
  type ReviewRef,
} from "../../github/ReviewRef";
import { usePullReview } from "../../hooks/usePullReview";
import { useReviewMode } from "../../hooks/useReviewMode";
import { GitHubIcon, RefreshIcon, SignOutIcon } from "../icons";
import { SignInPanel } from "../sign-in/SignInPanel";
import { DiffThemePicker } from "./DiffThemePicker";
import { PullStatus } from "./PullStatus";
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
    state,
    approval,
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
  const entries = useMemo(() => commitEntries(commits), [commits]);
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
  // A commit's manifest takes about a second cold, so it starts loading before the click, which can then find it in the HTTP cache.
  const warmed = useRef(new Set<string>());
  function warmCommit(sha: string) {
    if (warmed.current.has(sha)) return;
    warmed.current.add(sha);
    void PullLoader.open(atCommit(pull, sha), GitHubSignIn.token).catch(() => undefined);
  }
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
            commits={entries}
            selected={pull.commit}
            all
            labels={labels}
            onSelect={openCommit}
            onWarm={warmCommit}
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
                commits={entries}
                selected={pull.commit}
                all
                labels={labels}
                autoFocus={picked.current === "commits"}
                onSelect={(sha) => {
                  picked.current = "commits";
                  openCommit(sha);
                }}
                onWarm={warmCommit}
              />
            )
          }
          reviewMode={reviewMode}
          onReviewMode={toggleReviewMode}
          descriptionExpanded={Boolean(shown) || isCommit(pull)}
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
              {isPullRequest(pull) && (
                <PullStatus state={state} approval={approval} labels={labels} />
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
      </div>
    </>
  );
}
