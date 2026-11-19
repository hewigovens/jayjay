import type { Labels } from "@jayjay/review-web/types";
import type { PullCommit } from "../../github/PullManifest";
import { useHoverIntent } from "../../hooks/useHoverIntent";
import { commitItems } from "./commitItems";

export function CommitSidebar({
  open,
  commits,
  selected,
  labels,
  onSelect,
  onWarm,
}: {
  open: boolean;
  commits: PullCommit[];
  selected: string | undefined;
  labels: Labels;
  onSelect: (sha: string | undefined) => void;
  onWarm: (sha: string) => void;
}) {
  const intent = useHoverIntent();
  return (
    <aside className={`commit-sidebar ${open ? "open" : ""}`} aria-label={labels.pullCommits}>
      <ol>
        {commitItems(commits, selected, labels, onSelect, onWarm).map((item) => (
          <li key={item.key}>
            <button
              className="picker-row"
              aria-current={item.current}
              {...intent(item.warm)}
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
