import { expect, test } from "vitest";
import { parsePullRequest } from "../src/github/PullRequestRef";

test("reads GitHub URLs, app paths, and short references", () => {
  const pull = { owner: "jj-vcs", repo: "jj", number: 42 };
  for (const input of [
    "https://github.com/jj-vcs/jj/pull/42",
    " github.com/jj-vcs/jj/pull/42/files?w=1#diff ",
    "/jj-vcs/jj/pull/42",
    "jj-vcs/jj/pulls/42",
    "jj-vcs/jj#42",
  ])
    expect(parsePullRequest(input), input).toEqual(pull);
  const commit = "09f2cf76eccb6061937a49111593fc457c26378e";
  for (const input of [
    `https://github.com/jj-vcs/jj/pull/42/commits/${commit}`,
    `/jj-vcs/jj/pull/42/commits/${commit}?diff=split`,
  ])
    expect(parsePullRequest(input), input).toEqual({ ...pull, commit });
  expect(parsePullRequest("/jj-vcs/jj/pull/42/commits")).toEqual(pull);
});

test("rejects anything that is not one pull request", () => {
  for (const input of [
    "",
    "/",
    "https://gitlab.com/a/b/pull/1",
    "jj-vcs/jj/issues/42",
    "jj-vcs/jj/pull/0",
    "a/../pull/1",
    "../a/pull/1",
    "a/b c/pull/1",
    "jj-vcs/jj/pull/99999999999999999",
  ])
    expect(parsePullRequest(input), input).toBeNull();
});
