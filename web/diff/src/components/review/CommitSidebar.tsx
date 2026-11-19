import type { Labels } from "@jayjay/review-web/types";
import type { PullCommit } from "../../github/PullManifest";
import { commitItems } from "./commitItems";

export function CommitSidebar({
  open,
  commits,
  selected,
  labels,
  onSelect,
}: {
  open: boolean;
  commits: PullCommit[];
  selected: string | undefined;
  labels: Labels;
  onSelect: (sha: string | undefined) => void;
}) {
  return (
    <aside className={`commit-sidebar ${open ? "open" : ""}`} aria-label={labels.pullCommits}>
      <ol>
        {commitItems(commits, selected, labels, onSelect).map((item) => (
          <li key={item.key}>
            <button
              className="picker-row"
              aria-current={item.current}
              onClick={() => {
                if (!item.current) item.pick();
              }}
            >
              {item.content}
            </button>
          </li>
        ))}
      </ol>
    </aside>
  );
}
