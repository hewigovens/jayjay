import { expect, test } from "vitest";
import {
  gitHubUrl,
  parseReviewRef,
  reviewKind,
  reviewName,
  reviewPath,
  threeDotComparison,
} from "../src/github/ReviewRef";

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

test("reads comparisons with a fork's branch", () => {
  for (const [input, head] of [
    ["github.com/o/r/compare/main...user:feature/x", "user:feature/x"],
    ["/o/r/compare/main...user:fork-name:feature", "user:fork-name:feature"],
    ["/o/r/compare/main...user%3Abranch", "user:branch"],
  ]) {
    const target = parseReviewRef(input);
    expect(target, input).toEqual({ owner: "o", repo: "r", base: "main", head });
    expect(parseReviewRef(reviewPath(target!))).toEqual(target);
  }
  expect(reviewKind(parseReviewRef("/o/r/compare/main...user:x")!)).toBe("compare");
});

test("reads commit URLs", () => {
  const sha = "09f2cf76eccb6061937a49111593fc457c26378e";
  for (const input of [
    `https://github.com/o/r/commit/${sha}`,
    `github.com/o/r/commit/${sha}.diff`,
    `/o/r/commit/${sha}?diff=split#diff-abc`,
  ])
    expect(parseReviewRef(input), input).toEqual({ owner: "o", repo: "r", sha });
  const short = parseReviewRef("o/r/commit/09f2cf7")!;
  expect(short).toEqual({ owner: "o", repo: "r", sha: "09f2cf7" });
  expect(reviewPath(short)).toBe("/o/r/commit/09f2cf7");
  expect(gitHubUrl(short)).toBe("https://github.com/o/r/commit/09f2cf7");
  expect(reviewName(short)).toBe("o/r@09f2cf7");
  expect(reviewKind(short)).toBe("commit");
  expect(reviewKind({ owner: "o", repo: "r", number: 1, commit: sha })).toBe("commit");
  expect(reviewKind({ owner: "o", repo: "r", number: 1 })).toBe("pull");
});

test("offers the three-dot form of a two-dot comparison", () => {
  expect(threeDotComparison("https://github.com/o/r/compare/main..feature/x?expand=1")).toEqual({
    owner: "o",
    repo: "r",
    base: "main",
    head: "feature/x",
  });
  for (const input of ["/o/r/compare/main...feature", "/o/r/compare/main", "/o/r/pulls"])
    expect(threeDotComparison(input), input).toBeNull();
});

test("rejects anything that is not one pull request, comparison, or commit", () => {
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
    "o/r/compare/a/../b",
    "o/r/commit/xyz1234",
    "o/r/commits/main",
    "o/r/pulls",
  ])
    expect(parseReviewRef(input), input).toBeNull();
});

test("decodes comparison refs once and round-trips request paths", () => {
  for (const input of [
    "https://github.com/o/r/compare/release%2Fv1...feature%2fx",
    "/o/r/compare/release%2Fv1...feature%2Fx?expand=1",
  ]) {
    const target = parseReviewRef(input);
    expect(target).toEqual({ owner: "o", repo: "r", base: "release/v1", head: "feature/x" });
    expect(parseReviewRef(reviewPath(target!))).toEqual(target);
  }
  const target = parseReviewRef("o/r/compare/feature%2Fx%2By%40z");
  expect(target).toEqual({ owner: "o", repo: "r", head: "feature/x+y@z" });
  expect(reviewPath(target!)).toBe("/o/r/compare/feature/x+y@z");
  for (const range of [
    "main...bad%",
    "main...%FF",
    "main...%252Fx",
    "main...%2E%2E/x",
    "main...x%3Fy",
    "main...x%23y",
  ]) {
    expect(parseReviewRef(`o/r/compare/${range}`)).toBeNull();
  }
});
