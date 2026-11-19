import { ChevronIcon } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import type { PullCommit } from "../../github/PullManifest";
import { CommitIcon } from "../icons";
import { commitItems } from "./commitItems";
import { Picker } from "./Picker";

export function CommitPicker({
  commits,
  selected,
  labels,
  autoFocus,
  onSelect,
}: {
  commits: PullCommit[];
  selected: string | undefined;
  labels: Labels;
  autoFocus: boolean;
  onSelect: (sha: string | undefined) => void;
}) {
  const position = commits.findIndex((commit) => commit.sha === selected) + 1;
  const current = commits[position - 1]?.title ?? labels.pullAllCommits;
  return (
    <Picker
      className="commit-picker"
      label={`${labels.pullCommits}: ${current}`}
      autoFocus={autoFocus}
      button={
        <>
          <CommitIcon />
          <span className="commit-current">{current}</span>
          <span>{position ? `${position}/${commits.length}` : commits.length}</span>
          <ChevronIcon open />
        </>
      }
      items={commitItems(commits, selected, labels, onSelect)}
    />
  );
}
