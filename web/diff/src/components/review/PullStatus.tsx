import type { Labels } from "@jayjay/review-web/types";
import type { PullApproval, PullState } from "../../github/PullManifest";
import { ApprovedIcon, ChangesRequestedIcon, ClosedIcon, MergedIcon } from "../icons";

export function PullStatus({
  state,
  approval,
  labels,
}: {
  state: PullState | undefined;
  approval: PullApproval | undefined;
  labels: Labels;
}) {
  const approved = approval?.state === "approved";
  return (
    <>
      {state === "merged" && (
        <span className="pull-status pull-status-merged">
          <MergedIcon />
          <span>{labels.pullMerged}</span>
        </span>
      )}
      {state === "closed" && (
        <span className="pull-status pull-status-closed">
          <ClosedIcon />
          <span>{labels.pullClosed}</span>
        </span>
      )}
      {approval && (
        <span
          className={`pull-status pull-status-${approval.state}`}
          title={approval.reviewers.join(", ")}
        >
          {approved ? <ApprovedIcon /> : <ChangesRequestedIcon />}
          <span>{approved ? labels.pullApproved : labels.pullChangesRequested}</span>
        </span>
      )}
    </>
  );
}
