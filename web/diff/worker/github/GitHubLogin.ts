import type { DeviceCode, TokenPoll } from "../../src/github/SignIn";

const grant = "urn:ietf:params:oauth:grant-type:device_code";

interface DeviceReply {
  device_code?: string;
  user_code?: string;
  verification_uri?: string;
  interval?: number;
  expires_in?: number;
  access_token?: string;
  scope?: unknown;
  error?: string;
}

// GitHub's device-flow endpoints have no CORS, so the Worker relays them.
export class GitHubLogin {
  constructor(
    private clientId: string,
    private fetcher: typeof fetch = fetch.bind(globalThis),
  ) {}

  // `write` adds public_repo for marking files viewed; neither scope reads private repositories, so manifests stay safe to cache.
  async start(write = false): Promise<DeviceCode | null> {
    const reply = await this.post("https://github.com/login/device/code", {
      client_id: this.clientId,
      scope: write ? "public_repo" : "",
    });
    if (!reply?.device_code || !reply.user_code || !reply.verification_uri || !reply.expires_in)
      return null;
    return {
      deviceCode: reply.device_code,
      userCode: reply.user_code,
      verificationUri: reply.verification_uri,
      interval: reply.interval ?? 5,
      expiresIn: reply.expires_in,
    };
  }

  async poll(deviceCode: string): Promise<TokenPoll> {
    const reply = await this.post("https://github.com/login/oauth/access_token", {
      client_id: this.clientId,
      device_code: deviceCode,
      grant_type: grant,
    });
    if (reply?.access_token)
      return {
        token: reply.access_token,
        scope: typeof reply.scope === "string" ? reply.scope : "",
      };
    switch (reply?.error) {
      case "authorization_pending":
        return { error: "pending" };
      case "slow_down":
        return { error: "slow-down" };
      case "expired_token":
        return { error: "expired" };
      case "access_denied":
        return { error: "denied" };
      default:
        return { error: "unavailable" };
    }
  }

  private async post(url: string, form: Record<string, string>): Promise<DeviceReply | null> {
    const response = await this.fetcher(url, {
      method: "POST",
      headers: { accept: "application/json", "user-agent": "jayjay-pr" },
      body: new URLSearchParams(form),
    });
    return (await response.json().catch(() => null)) as DeviceReply | null;
  }
}
