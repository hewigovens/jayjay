import type { PullRequestRef } from "./PullRequestRef";

export interface PullSample extends PullRequestRef {
  title: string;
  files: number;
  added: number;
  removed: number;
}

export const samples: PullSample[] = [
  {
    owner: "hewigovens",
    repo: "jayjay",
    number: 453,
    title: "fix(sync): follow the configured remote instead of assuming origin",
    files: 26,
    added: 255,
    removed: 75,
  },
  {
    owner: "jj-vcs",
    repo: "jj",
    number: 10570,
    title: "git push: don't validate commits that are on the remote as a tag",
    files: 3,
    added: 40,
    removed: 6,
  },
  {
    owner: "zed-industries",
    repo: "zed",
    number: 66419,
    title: "Introduce an application HTTP client crate",
    files: 195,
    added: 709,
    removed: 671,
  },
];
