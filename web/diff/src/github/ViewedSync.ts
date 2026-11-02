import type { PullRequestRef } from "./PullRequestRef";

const filesQuery = `query($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) { pullRequest(number: $number) { id
    files(first: 100, after: $after) { pageInfo { hasNextPage endCursor } nodes { path viewerViewedState } } } } }`;
const mark =
  "mutation($id: ID!, $path: String!) { markFileAsViewed(input: { pullRequestId: $id, path: $path }) { clientMutationId } }";
const unmark =
  "mutation($id: ID!, $path: String!) { unmarkFileAsViewed(input: { pullRequestId: $id, path: $path }) { clientMutationId } }";

interface FilesPage {
  repository: {
    pullRequest: {
      id: string;
      files: {
        pageInfo: { hasNextPage: boolean; endCursor: string };
        nodes: { path: string; viewerViewedState: string }[];
      };
    } | null;
  } | null;
}

export class ViewedSync {
  private writes = new Map<string, Promise<void>>();

  private constructor(
    private id: string,
    private viewed: Map<string, boolean>,
    private token: string,
    private fetcher: typeof fetch,
  ) {}

  static async load(
    pull: PullRequestRef,
    token: string,
    signal?: AbortSignal,
    fetcher: typeof fetch = fetch.bind(globalThis),
  ) {
    const viewed = new Map<string, boolean>();
    let id = "";
    let after: string | null = null;
    do {
      const page: FilesPage = await graphql(
        fetcher,
        token,
        filesQuery,
        { owner: pull.owner, name: pull.repo, number: pull.number, after },
        signal,
      );
      const pullRequest = page.repository?.pullRequest;
      if (!pullRequest) throw new Error("not-found");
      id = pullRequest.id;
      // DISMISSED means viewed before the file last changed, which GitHub shows unchecked.
      for (const node of pullRequest.files.nodes)
        viewed.set(node.path, node.viewerViewedState === "VIEWED");
      after = pullRequest.files.pageInfo.hasNextPage ? pullRequest.files.pageInfo.endCursor : null;
    } while (after);
    return new ViewedSync(id, viewed, token, fetcher);
  }

  isViewed(path: string) {
    return this.viewed.get(path) === true;
  }

  // Queued behind the file's earlier writes so they reach GitHub in order.
  set(path: string, viewed: boolean) {
    const write = (this.writes.get(path) ?? Promise.resolve())
      .catch(() => {})
      .then(() => this.write(path, viewed));
    this.writes.set(path, write);
    void write
      .catch(() => {})
      .finally(() => {
        if (this.writes.get(path) === write) this.writes.delete(path);
      });
    return write;
  }

  private async write(path: string, viewed: boolean) {
    if (!this.viewed.has(path) || this.isViewed(path) === viewed) return;
    this.viewed.set(path, viewed);
    try {
      await graphql(this.fetcher, this.token, viewed ? mark : unmark, { id: this.id, path });
    } catch (error) {
      this.viewed.set(path, !viewed);
      throw error;
    }
  }
}

async function graphql<T>(
  fetcher: typeof fetch,
  token: string,
  query: string,
  variables: Record<string, unknown>,
  signal?: AbortSignal,
): Promise<T> {
  const response = await fetcher("https://api.github.com/graphql", {
    method: "POST",
    signal,
    headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
    body: JSON.stringify({ query, variables }),
  });
  if (!response.ok) throw new Error(`GitHub GraphQL ${response.status}`);
  const body = (await response.json()) as {
    data?: T;
    errors?: { type?: string; message: string }[];
  };
  if (body.errors?.length || !body.data)
    throw new Error(body.errors?.[0]?.type ?? body.errors?.[0]?.message ?? "GitHub GraphQL error");
  return body.data;
}
