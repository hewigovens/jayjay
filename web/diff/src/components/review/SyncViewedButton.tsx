import { ToolButton, ToolLink } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import { SyncIcon } from "../icons";
import { DeviceCodeDialog } from "../sign-in/DeviceCodeDialog";
import { useDeviceSignIn } from "../../hooks/useDeviceSignIn";
import type { SyncFailure } from "../../hooks/usePullReview";

export function SyncViewedButton({
  labels,
  syncing,
  failure,
  onEnabled,
}: {
  labels: Labels;
  syncing: boolean;
  failure: SyncFailure | undefined;
  onEnabled: () => void;
}) {
  const { code, failed: signInFailed, start, stop } = useDeviceSignIn(true, onEnabled);

  if (syncing && failure === "restricted")
    return (
      <ToolLink
        href="https://github.com/settings/applications"
        target="_blank"
        rel="noreferrer"
        className="pull-sync-failed"
        label={labels.pullSyncViewedRestricted}
      >
        <SyncIcon />
      </ToolLink>
    );
  if (syncing) {
    const label = failure ? labels.pullSyncViewedFailed : labels.pullSyncingViewed;
    return (
      <span
        className={`jj-tool pull-synced ${failure ? "pull-sync-failed" : ""}`}
        role="img"
        title={label}
        aria-label={label}
      >
        <SyncIcon />
      </span>
    );
  }
  return (
    <span className="sync-viewed" aria-live="polite">
      {code && (
        <DeviceCodeDialog
          code={code}
          title={labels.pullSyncViewed}
          detail={labels.pullSyncViewedScope}
          labels={labels}
          onCancel={stop}
        />
      )}
      {signInFailed && (
        <span role="alert" className="share-failed">
          {labels.pullSignInFailed}
        </span>
      )}
      <ToolButton label={labels.pullSyncViewed} onClick={() => void start()}>
        <SyncIcon />
      </ToolButton>
    </span>
  );
}
