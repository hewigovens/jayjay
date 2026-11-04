import { useRef } from "react";
import { ReviewWorkspace, ToolButton, ToolLink } from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { gitHubUrl, isPullRequest, type ReviewRef } from "../../github/ReviewRef";
import { usePullReview } from "../../hooks/usePullReview";
import { GitHubIcon, SignOutIcon } from "../icons";
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
  const picked = useRef<"commits" | "stack">(undefined);

  return (
    <ReviewWorkspace
      key={shown ?? ""}
      review={review}
      labels={labels}
      savedLabel={labels.savedLocally}
      avatars
      fill
      onRetry={() => (review.error === "worker-stopped" ? location.reload() : review.retry())}
      footer={
        needsSignIn &&
        !signedIn &&
        !review.result && <SignInPanel labels={labels} onSignedIn={onSignedIn} />
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
          {review.result && <ShareNotesButton labels={labels} notes={notes} />}
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
        </>
      }
    />
  );
}
