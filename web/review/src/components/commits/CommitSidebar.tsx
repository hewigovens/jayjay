/** @jsxRuntime automatic */
import { useHoverIntent } from "../../hooks/useHoverIntent";
import { commitItems, type CommitListProps } from "./commitItems";

/** The commits as a column beside `ReviewWorkspace`, under the same `.jayjay-review` ancestor; phones hide it. */
export function CommitSidebar({ open, ...list }: CommitListProps & { open: boolean }) {
  const intent = useHoverIntent();
  return (
    <aside
      className={`jj-commit-sidebar ${open ? "jj-open" : ""}`}
      aria-label={list.labels.pullCommits}
    >
      <ol>
        {commitItems(list).map((item) => (
          <li key={item.key}>
            <button
              className="jj-picker-row"
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
