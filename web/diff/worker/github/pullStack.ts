import type { PullStackEntry } from "../../src/github/PullManifest";
import type { GitHubPull } from "./GitHub";

// Bounds the API calls a stack costs: one per step each way.
const maxStack = 8;

export async function pullStack(
  get: (path: string) => Promise<GitHubPull[]>,
  repository: string,
  pull: GitHubPull,
): Promise<PullStackEntry[] | undefined> {
  const owner = repository.split("/")[0];
  const seen = new Set([pull.number]);
  const step = async (path: string) => {
    const found = await get(`/repos/${repository}/pulls?${path}&per_page=2`);
    const next = found.length === 1 ? found[0] : undefined;
    if (!next || seen.has(next.number)) return undefined;
    seen.add(next.number);
    return next;
  };
  const below = async () => {
    const pulls: GitHubPull[] = [];
    for (
      let base = pull.base;
      base.ref !== pull.base.repo.default_branch && pulls.length < maxStack;
    ) {
      const next = await step(`state=all&head=${encodeURIComponent(`${owner}:${base.ref}`)}`);
      if (!next) break;
      pulls.unshift(next);
      base = next.base;
    }
    return pulls;
  };
  const above = async () => {
    const pulls: GitHubPull[] = [];
    // Only a branch of this repository can be another pull request's base.
    for (let head = pull.head; head.repo?.full_name === repository && pulls.length < maxStack;) {
      const next = await step(`state=open&base=${encodeURIComponent(head.ref)}`);
      if (!next) break;
      pulls.push(next);
      head = next.head;
    }
    return pulls;
  };
  const [lower, upper] = await Promise.all([below(), above()]);
  const stack = [...lower, pull, ...upper];
  return stack.length > 1 ? stack.map(stackEntry) : undefined;
}

function stackEntry({ number, title, state, merged_at }: GitHubPull): PullStackEntry {
  return { number, title, state: merged_at ? "merged" : state };
}
