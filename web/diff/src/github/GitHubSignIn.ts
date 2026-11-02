import type { DeviceCode, TokenPoll } from "./SignIn";

const tokenKey = "jayjay.github.token";
const scopeKey = "jayjay.github.scope";

export class GitHubSignIn {
  // Holds the sign-in when storage is unavailable, so it still lasts for this page.
  private static memory: { token: string; scope: string } | null = null;

  static get token(): string | null {
    try {
      return localStorage.getItem(tokenKey) ?? this.memory?.token ?? null;
    } catch {
      return this.memory?.token ?? null;
    }
  }

  // Marking files viewed needs public_repo, or repo where the app was granted it before.
  static get canWrite(): boolean {
    let scope = this.memory?.scope ?? "";
    try {
      scope = localStorage.getItem(scopeKey) ?? scope;
    } catch {
      /* Memory has it. */
    }
    return scope.split(/[\s,]+/).some((name) => name === "public_repo" || name === "repo");
  }

  static signOut() {
    this.memory = null;
    try {
      localStorage.removeItem(tokenKey);
      localStorage.removeItem(scopeKey);
    } catch {
      /* Nothing was stored. */
    }
  }

  static async start(write: boolean, signal?: AbortSignal): Promise<DeviceCode> {
    const response = await fetch("/api/login/device", {
      method: "POST",
      signal,
      body: JSON.stringify({ write }),
    });
    if (!response.ok) throw new Error("unavailable");
    return (await response.json()) as DeviceCode;
  }

  static async wait(code: DeviceCode, signal: AbortSignal): Promise<void> {
    let interval = code.interval;
    const deadline = Date.now() + code.expiresIn * 1000;
    while (Date.now() < deadline) {
      await new Promise((resolve, reject) => {
        const timer = setTimeout(resolve, interval * 1000);
        signal.addEventListener(
          "abort",
          () => {
            clearTimeout(timer);
            reject(signal.reason);
          },
          { once: true },
        );
      });
      const response = await fetch("/api/login/token", {
        method: "POST",
        signal,
        body: JSON.stringify({ deviceCode: code.deviceCode }),
      });
      const poll = (await response.json().catch(() => ({ error: "unavailable" }))) as TokenPoll;
      if ("token" in poll) {
        this.memory = { token: poll.token, scope: poll.scope };
        try {
          localStorage.setItem(tokenKey, poll.token);
          localStorage.setItem(scopeKey, poll.scope);
        } catch {
          /* Memory keeps it for this page. */
        }
        return;
      }
      if (poll.error === "slow-down") interval += 5;
      else if (poll.error !== "pending") throw new Error(poll.error);
    }
    throw new Error("expired");
  }
}
