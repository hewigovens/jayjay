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

export type ReviewRef = PullRequestRef | CompareRef;

const name = /^(?!\.{1,2}$)[A-Za-z0-9_.-]+$/;
// A branch, tag, or commit of the same repository; a fork's `owner:branch` and git's `..` are not supported.
const ref = /^(?!\/)(?!.*\/$)(?!.*\.\.)[A-Za-z0-9_./@+-]+$/;

// Accepts GitHub pull and compare URLs (any scheme, `/files`, `/commits/<sha>`, query, fragment), this app's paths, and `owner/repo#1`.
export function parseReviewRef(input: string): ReviewRef | null {
  const text = input
    .trim()
    .replace(/^https?:\/\//, "")
    .replace(/^(www\.)?github\.com\//, "")
    .replace(/^\//, "");
  const compare =
    /^([^/#?\s]+)\/([^/#?\s]+)\/compare\/([^?#\s]+?)(?:\/commits\/([0-9a-f]{40}))?\/?(?:[?#]\S*)?$/.exec(
      text,
    );
  if (compare) {
    const [, owner, repo, range, commit] = compare;
    const refs = range.split("...");
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

export function isPullRequest(target: ReviewRef): target is PullRequestRef {
  return "number" in target;
}

export function compareRange({ base, head }: CompareRef) {
  return base ? `${base}...${head}` : head;
}

// Also GitHub's path, except for a comparison's commit.
export function reviewPath(target: ReviewRef) {
  const { owner, repo, commit } = target;
  const what = isPullRequest(target) ? `pull/${target.number}` : `compare/${compareRange(target)}`;
  return `/${owner}/${repo}/${what}${commit ? `/commits/${commit}` : ""}`;
}

export function gitHubUrl(target: ReviewRef) {
  if (!isPullRequest(target) && target.commit)
    return `https://github.com/${target.owner}/${target.repo}/commit/${target.commit}`;
  return `https://github.com${reviewPath(target)}`;
}

export function reviewName(target: ReviewRef) {
  return `${target.owner}/${target.repo}${isPullRequest(target) ? `#${target.number}` : ` ${compareRange(target)}`}`;
}
