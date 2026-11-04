import type { Labels } from "@jayjay/review-web/types";
import { SyncIcon } from "../icons";
import { DeviceCodeDialog } from "../sign-in/DeviceCodeDialog";
import { useDeviceSignIn } from "../../hooks/useDeviceSignIn";

export function SyncViewedButton({
  labels,
  syncing,
  failed,
  onEnabled,
}: {
  labels: Labels;
  syncing: boolean;
  failed: boolean;
  onEnabled: () => void;
}) {
  const { code, failed: signInFailed, start, stop } = useDeviceSignIn(true, onEnabled);

  if (syncing) {
    const label = failed ? labels.pullSyncViewedFailed : labels.pullSyncingViewed;
    return (
      <span
        className={`pull-link pull-synced ${failed ? "pull-sync-failed" : ""}`}
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
      <button
        className="pull-link"
        title={labels.pullSyncViewed}
        aria-label={labels.pullSyncViewed}
        onClick={() => void start()}
      >
        <SyncIcon />
      </button>
    </span>
  );
}
