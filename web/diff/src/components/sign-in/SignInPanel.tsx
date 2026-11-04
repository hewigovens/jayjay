import type { Labels } from "@jayjay/review-web/types";
import { DeviceCodeDialog } from "./DeviceCodeDialog";
import { useDeviceSignIn } from "../../hooks/useDeviceSignIn";

export function SignInPanel({ labels, onSignedIn }: { labels: Labels; onSignedIn: () => void }) {
  const { code, failed, start, stop } = useDeviceSignIn(false, onSignedIn);

  return (
    <section className="sign-in">
      {code && (
        <DeviceCodeDialog code={code} title={labels.pullSignIn} labels={labels} onCancel={stop} />
      )}
      <button className="sign-in-button" onClick={() => void start()}>
        {labels.pullSignIn}
      </button>
      {failed && (
        <p className="sign-in-note" role="alert">
          {labels.pullSignInFailed}
        </p>
      )}
    </section>
  );
}
