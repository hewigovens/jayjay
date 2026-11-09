export interface PullRequestRef {
  owner: string;
  repo: string;
  number: number;
  commit?: string;
}

export interface CompareRef {
  owner: string;
  repo: string;
  // Defaults to the repository's default branch.
  base?: string;
  head: string;
  commit?: string;
}

export interface CommitRef {
  owner: string;
  repo: string;
  sha: string;
  commit?: never;
}

export type ReviewRef = PullRequestRef | CompareRef | CommitRef;

const name = /^(?!\.{1,2}$)[A-Za-z0-9_.-]+$/;
// A branch, tag, or commit, optionally of a fork as `owner:branch` or `owner:repo:branch`; git's `..` is not one.
const ref = /^(?:[A-Za-z0-9-]+:(?:[A-Za-z0-9_.-]+:)?)?(?!\/)(?!.*\/$)(?!.*\.\.)[A-Za-z0-9_./@+-]+$/;

function gitHubPath(input: string) {
  return input
    .trim()
    .replace(/^https?:\/\//, "")
    .replace(/^(www\.)?github\.com\//, "")
    .replace(/^\//, "");
}

// Accepts GitHub pull, compare, and commit URLs (any scheme, `/files`, `/commits/<sha>`, query, fragment), this app's paths, and `owner/repo#1`.
export function parseReviewRef(input: string): ReviewRef | null {
  const text = gitHubPath(input);
  const single =
    /^([^/#?\s]+)\/([^/#?\s]+)\/commit\/([0-9a-f]{7,40})(?:\.(?:diff|patch))?(?:[/?#]\S*)?$/.exec(
      text,
    );
  if (single) {
    const [, owner, repo, sha] = single;
    return name.test(owner) && name.test(repo) ? { owner, repo, sha } : null;
  }
  const compare =
    /^([^/#?\s]+)\/([^/#?\s]+)\/compare\/([^?#\s]+?)(?:\/commits\/([0-9a-f]{40}))?\/?(?:[?#]\S*)?$/.exec(
      text,
    );
  if (compare) {
    const [, owner, repo, range, commit] = compare;
    let refs: string[];
    try {
      refs = range.split("...").map(decodeURIComponent);
    } catch {
      return null;
    }
    const [base, head] = refs.length === 2 ? refs : [undefined, refs[0]];
    if (
      !name.test(owner) ||
      !name.test(repo) ||
      refs.length > 2 ||
      !ref.test(head) ||
      (base !== undefined && !ref.test(base))
    )
      return null;
    return { owner, repo, ...(base && { base }), head, ...(commit && { commit }) };
  }
  const match =
    /^([^/#?\s]+)\/([^/#?\s]+)(?:\/pulls?\/|#)(\d+)(?:\/commits\/([0-9a-f]{40}))?(?:[/?#]\S*)?$/.exec(
      text,
    );
  if (!match) return null;
  const [, owner, repo, digits, commit] = match;
  const number = Number(digits);
  if (!name.test(owner) || !name.test(repo) || !Number.isSafeInteger(number) || number < 1)
    return null;
  return { owner, repo, number, ...(commit && { commit }) };
}

// The three-dot form of a two-dot comparison, which GitHub's API doesn't serve.
export function threeDotComparison(input: string): CompareRef | null {
  const range =
    /^([^/#?\s]+\/[^/#?\s]+\/compare\/)([^?#\s]+?)(?<!\.)\.\.(?!\.)([^?#\s]+?)\/?(?:[?#]\S*)?$/.exec(
      gitHubPath(input),
    );
  const target = range && parseReviewRef(`${range[1]}${range[2]}...${range[3]}`);
  return target && isCompare(target) ? target : null;
}

export function isPullRequest(target: ReviewRef): target is PullRequestRef {
  return "number" in target;
}

export function isCommit(target: ReviewRef): target is CommitRef {
  return "sha" in target;
}

function isCompare(target: ReviewRef): target is CompareRef {
  return "head" in target;
}

export function atCommit(target: ReviewRef, commit: string | undefined): ReviewRef {
  return isCommit(target) ? target : { ...target, commit };
}

export function compareRange({ base, head }: CompareRef) {
  return base ? `${base}...${head}` : head;
}

// Also GitHub's path, except for a comparison's commit.
export function reviewPath(target: ReviewRef) {
  const { owner, repo, commit } = target;
  const what = isPullRequest(target)
    ? `pull/${target.number}`
    : isCommit(target)
      ? `commit/${target.sha}`
      : `compare/${compareRange(target)}`;
  return `/${owner}/${repo}/${what}${commit ? `/commits/${commit}` : ""}`;
}

export function gitHubUrl(target: ReviewRef) {
  if (isCompare(target) && target.commit)
    return `https://github.com/${target.owner}/${target.repo}/commit/${target.commit}`;
  return `https://github.com${reviewPath(target)}`;
}

export function reviewName(target: ReviewRef) {
  const what = isPullRequest(target)
    ? `#${target.number}`
    : isCommit(target)
      ? `@${target.sha.slice(0, 7)}`
      : ` ${compareRange(target)}`;
  return `${target.owner}/${target.repo}${what}`;
}

export function reviewKind(target: ReviewRef): "pull" | "compare" | "commit" {
  return isCommit(target) || target.commit ? "commit" : isPullRequest(target) ? "pull" : "compare";
}
