import {
  maxFiles,
  type PullCommit,
  type PullError,
  type PullManifest,
} from "../../src/github/PullManifest";
import type { PullRequestRef } from "../../src/github/PullRequestRef";
import { diffFiles } from "./GitHubDiff";
import { pullStack } from "./pullStack";

export interface GitHubPull {
  number: number;
  title: string;
  state: "open" | "closed";
  merged_at: string | null;
  body: string | null;
  user: GitHubUser | null;
  changed_files: number;
  commits: number;
  base: { ref: string; sha: string; repo: { private: boolean; default_branch: string } };
  head: { ref: string; sha: string; repo: { full_name: string } | null };
}

interface GitHubUser {
  login: string;
  id: number;
}

interface GitHubCommit {
  sha: string;
  commit: { message: string; author: { name: string; date: string } | null };
  author: GitHubUser | null;
  parents: { sha: string }[];
}

export class GitHubError extends Error {
  constructor(
    readonly code: PullError,
    readonly status: number,
  ) {
    super(code);
  }
}

const perPage = 100;
// GitHub lists at most this many of a pull request's commits.
const maxCommits = 250;

export class GitHub {
  constructor(
    private token: string | undefined,
    private fetcher: typeof fetch = fetch.bind(globalThis),
  ) {}

  manifest(pull: PullRequestRef): Promise<PullManifest> {
    return pull.commit ? this.commitManifest(pull, pull.commit) : this.pullManifest(pull);
  }

  private async pullManifest({ owner, repo, number }: PullRequestRef): Promise<PullManifest> {
    const repository = `${owner}/${repo}`;
    const pull = await this.pull(repository, number);
    if (pull.changed_files > maxFiles) throw new GitHubError("too-large", 413);
    const [compare, commits, stack, files] = await Promise.all([
      this.get<{ merge_base_commit: { sha: string } }>(
        `/repos/${repository}/compare/${pull.base.sha}...${pull.head.sha}?per_page=1`,
      ),
      this.commits(repository, number, pull),
      // A stack is a convenience; failing to follow it still shows the pull request.
      pullStack((path) => this.get(path), repository, pull).catch(() => undefined),
      this.diff(`/${repository}/pull/${number}.diff`),
    ]);
    return {
      repository,
      number,
      title: pull.title,
      body: pull.body ?? "",
      author: pull.user && { login: pull.user.login, id: pull.user.id },
      baseSha: compare.merge_base_commit.sha,
      headSha: pull.head.sha,
      files,
      commits: commits.map(pullCommit),
      ...(stack && { stack }),
    };
  }

  private async commitManifest(
    { owner, repo, number }: PullRequestRef,
    sha: string,
  ): Promise<PullManifest> {
    const repository = `${owner}/${repo}`;
    const pull = await this.pull(repository, number);
    const commits = await this.commits(repository, number, pull);
    // Only the pull request's own commits open under its title.
    const commit = commits.find((commit) => commit.sha === sha);
    if (!commit) throw new GitHubError("not-found", 404);
    const files = await this.diff(`/${repository}/commit/${sha}.diff`);
    if (files.length > maxFiles) throw new GitHubError("too-large", 413);
    const [title, ...body] = commit.commit.message.split("\n");
    return {
      repository,
      number,
      title,
      body: body.join("\n").trim(),
      author: commit.author && { login: commit.author.login, id: commit.author.id },
      // A root commit has no parent; its files are all added, so nothing reads this side.
      baseSha: commit.parents[0]?.sha ?? "",
      headSha: sha,
      files,
      commits: commits.map(pullCommit),
      commit: sha,
    };
  }

  private async pull(repository: string, number: number) {
    const pull = await this.get<GitHubPull>(`/repos/${repository}/pulls/${number}`);
    // A reader's own token may see private repositories, whose data must not reach the shared edge cache.
    if (pull.base.repo.private) throw new GitHubError("not-found", 404);
    return pull;
  }

  private async commits(repository: string, number: number, pull: GitHubPull) {
    const pages = Math.ceil(Math.min(pull.commits, maxCommits) / perPage);
    const lists = await Promise.all(
      Array.from({ length: pages }, (_, index) =>
        this.get<GitHubCommit[]>(
          `/repos/${repository}/pulls/${number}/commits?per_page=${perPage}&page=${index + 1}`,
        ),
      ),
    );
    return lists.flat();
  }

  // github.com serves diffs outside the API rate limit.
  private async diff(path: string) {
    const response = await this.fetcher(`https://github.com${path}`, {
      headers: { "user-agent": "jayjay-pr" },
    });
    if (response.ok) return diffFiles(await response.text());
    if (response.status === 404) throw new GitHubError("not-found", 404);
    throw new GitHubError(
      response.status === 429 ? "rate-limited" : "unavailable",
      response.status === 429 ? 503 : 502,
    );
  }

  private async get<T>(path: string): Promise<T> {
    const response = await this.fetcher(`https://api.github.com${path}`, {
      headers: {
        accept: "application/vnd.github+json",
        "user-agent": "jayjay-pr",
        "x-github-api-version": "2022-11-28",
        ...(this.token && { authorization: `Bearer ${this.token}` }),
      },
    });
    if (response.ok) return (await response.json()) as T;
    if (response.status === 404 || response.status === 422) throw new GitHubError("not-found", 404);
    if (response.status === 401) throw new GitHubError("unauthorized", 401);
    if (
      response.status === 429 ||
      (response.status === 403 && response.headers.get("x-ratelimit-remaining") === "0")
    )
      throw new GitHubError("rate-limited", 503);
    throw new GitHubError("unavailable", 502);
  }
}

function pullCommit({ sha, commit, author }: GitHubCommit): PullCommit {
  return {
    sha,
    title: commit.message.split("\n", 1)[0],
    author: author?.login ?? commit.author?.name ?? "",
    timestampMillis: Date.parse(commit.author?.date ?? "") || 0,
  };
}
