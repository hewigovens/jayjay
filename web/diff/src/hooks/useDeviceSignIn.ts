import { useEffect, useRef, useState } from "react";
import { GitHubSignIn } from "../github/GitHubSignIn";
import type { DeviceCode } from "../github/SignIn";

export function useDeviceSignIn(write: boolean, onSignedIn: () => void) {
  const [code, setCode] = useState<DeviceCode>();
  const [failed, setFailed] = useState(false);
  const cancel = useRef<AbortController>(undefined);
  useEffect(() => () => cancel.current?.abort(), []);

  async function start() {
    cancel.current?.abort();
    const controller = new AbortController();
    cancel.current = controller;
    setFailed(false);
    try {
      const next = await GitHubSignIn.start(write, controller.signal);
      setCode(next);
      await GitHubSignIn.wait(next, controller.signal);
      onSignedIn();
    } catch {
      if (controller.signal.aborted) return;
      setFailed(true);
    }
    setCode(undefined);
  }

  function stop() {
    cancel.current?.abort();
    setCode(undefined);
  }

  return { code, failed, start, stop };
}
