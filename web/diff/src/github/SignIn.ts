export interface DeviceCode {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  interval: number;
  expiresIn: number;
}

export type TokenPoll =
  | { token: string; scope: string }
  | { error: "pending" | "slow-down" | "expired" | "denied" | "unavailable" };
