/** @jsxRuntime automatic */
import type { CommitEntry } from "../../model/commits";
import type { Labels } from "../../model/diff";
import { Avatar } from "../Avatar";
import type { PickerItem } from "./Picker";

export interface CommitListProps {
  /** Oldest first; they are listed newest first. */
  commits: CommitEntry[];
  /** The id of the commit shown; `undefined` is all of them together. */
  selected: string | undefined;
  /** Offers all the commits together above the list. */
  all?: boolean;
  labels: Labels;
  onSelect: (id: string | undefined) => void;
  /** Starts loading a commit once a pointer rests on its row or the keyboard focuses it. */
  onWarm?: (id: string) => void;
}

const units: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 31_536e6],
  ["month", 2_592e6],
  ["week", 6_048e5],
  ["day", 864e5],
  ["hour", 36e5],
  ["minute", 6e4],
];
const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

function ago(millis: number) {
  const elapsed = millis - Date.now();
  const [unit, size] =
    units.find(([, size]) => Math.abs(elapsed) >= size) ?? units[units.length - 1];
  return relative.format(Math.round(elapsed / size), unit);
}

export function commitItems({
  commits,
  selected,
  all = false,
  labels,
  onSelect,
  onWarm,
}: CommitListProps): PickerItem[] {
  const together: PickerItem[] = all
    ? [
        {
          key: "",
          current: selected === undefined,
          content: (
            <span className="jj-picker-title jj-picker-all">
              {labels.pullAllCommits}
              <span>{commits.length}</span>
            </span>
          ),
          pick: () => onSelect(undefined),
        },
      ]
    : [];
  return [
    ...together,
    ...[...commits].reverse().map((commit) => ({
      key: commit.id,
      current: commit.id === selected,
      pick: () => onSelect(commit.id),
      warm: onWarm && commit.id !== selected ? () => onWarm(commit.id) : undefined,
      content: (
        <>
          <span className="jj-picker-title">{commit.title}</span>
          <span className="jj-picker-meta">
            <code className="jj-picker-ref">
              <b>{commit.ref.slice(0, commit.refPrefix)}</b>
              {commit.ref.slice(commit.refPrefix)}
            </code>
            <span aria-hidden="true">·</span>
            <Avatar source={commit.avatar ?? null} name={commit.author} />
            <span className="jj-picker-author">{commit.author}</span>
            {commit.timestampMillis > 0 && (
              <>
                <span aria-hidden="true">·</span>
                <time dateTime={new Date(commit.timestampMillis).toISOString()}>
                  {ago(commit.timestampMillis)}
                </time>
              </>
            )}
            {commit.note && (
              <>
                <span aria-hidden="true">·</span>
                <span>{commit.note}</span>
              </>
            )}
          </span>
        </>
      ),
    })),
  ];
}
