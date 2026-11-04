import { expect, test } from "vitest";
import { gitHubUrl, parseReviewRef, reviewPath } from "../src/github/ReviewRef";

test("reads GitHub URLs, app paths, and short references", () => {
  const pull = { owner: "jj-vcs", repo: "jj", number: 42 };
  for (const input of [
    "https://github.com/jj-vcs/jj/pull/42",
    " github.com/jj-vcs/jj/pull/42/files?w=1#diff ",
    "/jj-vcs/jj/pull/42",
    "jj-vcs/jj/pulls/42",
    "jj-vcs/jj#42",
  ])
    expect(parseReviewRef(input), input).toEqual(pull);
  const commit = "09f2cf76eccb6061937a49111593fc457c26378e";
  for (const input of [
    `https://github.com/jj-vcs/jj/pull/42/commits/${commit}`,
    `/jj-vcs/jj/pull/42/commits/${commit}?diff=split`,
  ])
    expect(parseReviewRef(input), input).toEqual({ ...pull, commit });
  expect(parseReviewRef("/jj-vcs/jj/pull/42/commits")).toEqual(pull);
});

test("reads comparisons of branches, with or without a base", () => {
  const compare = { owner: "o", repo: "r", base: "main", head: "feature/x" };
  for (const input of [
    "https://github.com/o/r/compare/main...feature/x",
    "github.com/o/r/compare/main...feature/x?expand=1",
    "/o/r/compare/main...feature/x/",
  ])
    expect(parseReviewRef(input), input).toEqual(compare);
  expect(parseReviewRef("o/r/compare/v1.2")).toEqual({ owner: "o", repo: "r", head: "v1.2" });
  const commit = "09f2cf76eccb6061937a49111593fc457c26378e";
  const withCommit = parseReviewRef(`/o/r/compare/main...feature/x/commits/${commit}`);
  expect(withCommit).toEqual({ ...compare, commit });
  expect(reviewPath(withCommit!)).toBe(`/o/r/compare/main...feature/x/commits/${commit}`);
  expect(gitHubUrl(withCommit!)).toBe(`https://github.com/o/r/commit/${commit}`);
});

test("rejects anything that is not one pull request or comparison", () => {
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
    "o/r/compare/",
    "o/r/compare/main..feature",
    "o/r/compare/a...b...c",
    "o/r/compare/...b",
    "o/r/compare/main...user:branch",
    "o/r/compare/a/../b",
  ])
    expect(parseReviewRef(input), input).toBeNull();
});
