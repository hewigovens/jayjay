export interface PullRequestRef {
  owner: string;
  repo: string;
  number: number;
  commit?: string;
}

const name = /^(?!\.{1,2}$)[A-Za-z0-9_.-]+$/;

// Accepts GitHub pull URLs (any scheme, `/files`, `/commits/<sha>`, query, fragment), this app's paths, and `owner/repo#1`.
export function parsePullRequest(input: string): PullRequestRef | null {
  const text = input
    .trim()
    .replace(/^https?:\/\//, "")
    .replace(/^(www\.)?github\.com\//, "")
    .replace(/^\//, "");
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

export function pullPath({ owner, repo, number, commit }: PullRequestRef) {
  return `/${owner}/${repo}/pull/${number}${commit ? `/commits/${commit}` : ""}`;
}
