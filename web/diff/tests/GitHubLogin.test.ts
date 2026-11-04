import { expect, test } from "vitest";
import { GitHubLogin } from "../worker/github/GitHubLogin";

test("relays GitHub's device flow with the app's client ID, asking for public_repo only to write", async () => {
  const bodies: string[] = [];
  const replies = [
    {
      device_code: "dc",
      user_code: "ABCD-1234",
      verification_uri: "https://github.com/login/device",
      interval: 5,
      expires_in: 900,
    },
    { error: "authorization_pending" },
    { error: "slow_down" },
    { error: "access_denied" },
    { access_token: "gho_x", scope: "public_repo" },
  ];
  const login = new GitHubLogin("client", async (_, init) => {
    bodies.push(String(init?.body));
    return Response.json(replies.shift());
  });
  expect(await login.start()).toEqual({
    deviceCode: "dc",
    userCode: "ABCD-1234",
    verificationUri: "https://github.com/login/device",
    interval: 5,
    expiresIn: 900,
  });
  expect(bodies[0]).toBe("client_id=client&scope=");
  const polls = [];
  for (let i = 0; i < 4; i++) polls.push(await login.poll("dc"));
  expect(polls).toEqual([
    { error: "pending" },
    { error: "slow-down" },
    { error: "denied" },
    { token: "gho_x", scope: "public_repo" },
  ]);
  expect(bodies[1]).toContain("device_code=dc");
  await login.start(true).catch(() => {});
  expect(bodies[5]).toBe("client_id=client&scope=public_repo");
});
