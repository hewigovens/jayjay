import type { CommitEntry } from "@jayjay/review-web/types";
import type { PullCommit } from "./PullManifest";

const shortSha = 7;

// The fewest leading characters that tell a commit from the others listed, as jj highlights them.
function uniquePrefix(sha: string, commits: PullCommit[]) {
  let length = 1;
  while (
    length < shortSha &&
    commits.some((other) => other.sha !== sha && other.sha.startsWith(sha.slice(0, length)))
  )
    length++;
  return length;
}

export function commitEntries(commits: PullCommit[]): CommitEntry[] {
  return commits.map((commit) => ({
    id: commit.sha,
    ref: commit.sha.slice(0, shortSha),
    refPrefix: uniquePrefix(commit.sha, commits),
    title: commit.title,
    author: commit.author,
    avatar: commit.avatar ? { kind: "url", value: commit.avatar } : null,
    timestampMillis: commit.timestampMillis,
  }));
}
