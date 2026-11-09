import {
  maxFiles,
  type PullCommit,
  type PullError,
  type PullManifest,
} from "../../src/github/PullManifest";
import {
  compareRange,
  isCommit,
  isPullRequest,
  type CommitRef,
  type CompareRef,
  type PullRequestRef,
  type ReviewRef,
} from "../../src/github/ReviewRef";
import { diffFiles } from "./GitHubDiff";
import { pullStack } from "./pullStack";

export interface GitHubPull {
  number: number;
  title: string;
  state: "open" | "closed";
  merged_at: string | null;
  body: string | null;
  user: GitHubUser | null;
  created_at: string;
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
// GitHub lists at most this many of a pull request's or comparison's commits.
const maxCommits = 250;

export class GitHub {
  constructor(
    private token: string | undefined,
    private fetcher: typeof fetch = fetch.bind(globalThis),
  ) {}

  manifest(target: ReviewRef): Promise<PullManifest> {
    if (isCommit(target)) return this.singleCommitManifest(target);
    if (!isPullRequest(target)) return this.compareManifest(target);
    return target.commit
      ? this.pullCommitManifest(target, target.commit)
      : this.pullManifest(target);
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
      timestampMillis: Date.parse(pull.created_at) || undefined,
      body: pull.body ?? "",
      author: pull.user && { login: pull.user.login, id: pull.user.id },
      baseSha: compare.merge_base_commit.sha,
      headSha: pull.head.sha,
      files,
      commits: commits.map(pullCommit),
      ...(stack && { stack }),
    };
  }

  private async pullCommitManifest(
    { owner, repo, number }: PullRequestRef,
    sha: string,
  ): Promise<PullManifest> {
    const repository = `${owner}/${repo}`;
    const pull = await this.pull(repository, number);
    return this.commitManifest(repository, await this.commits(repository, number, pull), sha, {
      number,
    });
  }

  private async singleCommitManifest({ owner, repo, sha }: CommitRef): Promise<PullManifest> {
    const repository = `${owner}/${repo}`;
    const [, commit] = await Promise.all([
      this.publicRepository(repository),
      this.get<GitHubCommit>(`/repos/${repository}/commits/${sha}`),
    ]);
    return this.commitManifest(repository, [commit], commit.sha, {});
  }

  private async compareManifest(target: CompareRef): Promise<PullManifest> {
    const repository = `${target.owner}/${target.repo}`;
    const { default_branch } = await this.publicRepository(repository);
    const range = compareRange({ ...target, base: target.base ?? default_branch });
    const path = range.split("/").map(encodeURIComponent).join("/");
    const [comparison, files] = await Promise.all([
      this.comparison(repository, path),
      target.commit ? [] : this.diff(`/${repository}/compare/${path}.diff`),
    ]);
    if (target.commit)
      return this.commitManifest(repository, comparison.commits, target.commit, { range });
    if (files.length > maxFiles) throw new GitHubError("too-large", 413);
    return {
      repository,
      range,
      title: range,
      body: "",
      author: null,
      baseSha: comparison.mergeBase,
      headSha: comparison.head,
      files,
      commits: comparison.commits.map(pullCommit),
    };
  }

  private async commitManifest(
    repository: string,
    commits: GitHubCommit[],
    sha: string,
    of: Pick<PullManifest, "number" | "range">,
  ): Promise<PullManifest> {
    const commit = commits.find((commit) => commit.sha === sha);
    if (!commit) throw new GitHubError("not-found", 404);
    const files = await this.diff(`/${repository}/commit/${sha}.diff`);
    if (files.length > maxFiles) throw new GitHubError("too-large", 413);
    const [title, ...body] = commit.commit.message.split("\n");
    return {
      repository,
      ...of,
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

  private async comparison(repository: string, range: string) {
    type Page = {
      total_commits: number;
      merge_base_commit: { sha: string };
      commits: GitHubCommit[];
    };
    const page = (size: number, index: number) =>
      this.get<Page>(`/repos/${repository}/compare/${range}?per_page=${size}&page=${index}`);
    const first = await page(perPage, 1);
    const listed = Math.min(first.total_commits, maxCommits);
    const rest = await Promise.all(
      Array.from({ length: Math.ceil(listed / perPage) - 1 }, (_, index) =>
        page(perPage, index + 2),
      ),
    );
    const commits = [first, ...rest].flatMap((page) => page.commits).slice(0, maxCommits);
    // A comparison past maxCommits lists its head only on a later page.
    const last =
      first.total_commits > listed
        ? (await page(1, first.total_commits)).commits[0]
        : commits.at(-1);
    // With no commits of its own, the head is the merge base.
    return {
      mergeBase: first.merge_base_commit.sha,
      head: last?.sha ?? first.merge_base_commit.sha,
      commits,
    };
  }

  private async publicRepository(repository: string) {
    const found = await this.get<{ private: boolean; default_branch: string }>(
      `/repos/${repository}`,
    );
    if (found.private) throw new GitHubError("not-found", 404);
    return found;
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
