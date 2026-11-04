import { Dialog } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import type { DeviceCode } from "../../github/SignIn";

export function DeviceCodeDialog({
  code,
  title,
  detail,
  labels,
  onCancel,
}: {
  code: DeviceCode;
  title: string;
  detail?: string;
  labels: Labels;
  onCancel: () => void;
}) {
  return (
    <Dialog className="device-code" labelledBy="device-code-title" onClose={onCancel}>
      <h2 id="device-code-title">{title}</h2>
      {detail && <p>{detail}</p>}
      <p>{labels.pullDeviceCode}</p>
      <code className="sign-in-code">{code.userCode}</code>
      <a
        className="sign-in-button"
        href={code.verificationUri}
        target="_blank"
        rel="noreferrer"
        autoFocus
        onClick={() => void navigator.clipboard?.writeText(code.userCode).catch(() => {})}
      >
        {labels.pullCopyOpen}
      </a>
      <p className="sign-in-note" aria-live="polite">
        {labels.pullWaiting}
      </p>
      <button className="device-code-cancel" onClick={onCancel}>
        {labels.cancel}
      </button>
    </Dialog>
  );
}
