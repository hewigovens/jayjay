export interface PullManifest {
  repository: string;
  number?: number;
  // A comparison's `base...head`, with the default branch filled in as base.
  range?: string;
  title: string;
  timestampMillis?: number;
  body: string;
  author: { login: string; id: number } | null;
  // The merge base, or a single commit's first parent.
  baseSha: string;
  headSha: string;
  files: PullFile[];
  // Oldest first; GitHub lists at most 250.
  commits: PullCommit[];
  commit?: string;
  // Bottom first, this pull request included.
  stack?: PullStackEntry[];
}

export interface PullStackEntry {
  number: number;
  title: string;
  state: "open" | "closed" | "merged";
}

export interface PullCommit {
  sha: string;
  title: string;
  // The GitHub login, or the commit's author name without one.
  author: string;
  timestampMillis: number;
}

export interface PullFile {
  path: string;
  oldPath?: string;
  kind: "added" | "modified" | "removed" | "renamed" | "copied";
  added: number;
  removed: number;
  sha: string | null;
  oldSha?: string;
  binary?: true;
  oldMode?: string;
  newMode?: string;
}

export type PullError = "not-found" | "rate-limited" | "unauthorized" | "too-large" | "unavailable";

// The bundle loader's limit in web/review/wasm/src/bundle.rs.
export const maxFiles = 2000;
