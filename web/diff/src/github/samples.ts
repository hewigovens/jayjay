import type { PullRequestRef } from "./ReviewRef";

export interface PullSample extends PullRequestRef {
  title: string;
}

export const samples: PullSample[] = [
  {
    owner: "hewigovens",
    repo: "jayjay",
    number: 453,
    title: "fix(sync): follow the configured remote instead of assuming origin",
  },
  {
    owner: "jj-vcs",
    repo: "jj",
    number: 10654,
    title: "Let terms cancel out in directory/file conflicts",
  },
  {
    owner: "zed-industries",
    repo: "zed",
    number: 65970,
    title: "gpui: Make `Modifiers::secondary` web platform aware",
  },
];
