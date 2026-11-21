/** @jsxRuntime automatic */
import { ChevronIcon, CommitIcon } from "../icons";
import { commitItems, type CommitListProps } from "./commitItems";
import { Picker } from "./Picker";

/** The commit shown, as one row that opens the list; for `ReviewWorkspace`'s `filesHeader`. */
export function CommitPicker({ autoFocus, ...list }: CommitListProps & { autoFocus?: boolean }) {
  const { commits, selected, labels } = list;
  const position = commits.findIndex((commit) => commit.id === selected) + 1;
  const current = commits[position - 1]?.title ?? labels.pullAllCommits;
  return (
    <Picker
      className="jj-commit-picker"
      label={`${labels.pullCommits}: ${current}`}
      autoFocus={autoFocus}
      button={
        <>
          <CommitIcon />
          <span className="jj-commit-current">{current}</span>
          <span>{position ? `${position}/${commits.length}` : commits.length}</span>
          <ChevronIcon open />
        </>
      }
      items={commitItems(list)}
    />
  );
}
