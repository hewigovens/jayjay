import { Avatar } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import type { PullCommit } from "../../github/PullManifest";
import type { PickerItem } from "./Picker";

const shortSha = 7;
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

// The fewest leading characters that tell a commit from the others listed, as jj highlights them.
function uniquePrefix(sha: string, commits: PullCommit[]) {
  let length = 1;
  while (
    length < shortSha &&
    commits.some((other) => other.sha !== sha && other.sha.startsWith(sha.slice(0, length)))
  )
    length++;
  return length;
}

/** "All commits", then the commits newest first. */
export function commitItems(
  commits: PullCommit[],
  selected: string | undefined,
  labels: Labels,
  onSelect: (sha: string | undefined) => void,
  onWarm: (sha: string) => void,
): PickerItem[] {
  return [
    {
      key: "",
      current: !selected,
      content: (
        <span className="picker-title picker-all">
          {labels.pullAllCommits}
          <span>{commits.length}</span>
        </span>
      ),
      pick: () => onSelect(undefined),
    },
    ...[...commits].reverse().map((commit) => {
      const prefix = uniquePrefix(commit.sha, commits);
      return {
        key: commit.sha,
        current: commit.sha === selected,
        pick: () => onSelect(commit.sha),
        warm: commit.sha === selected ? undefined : () => onWarm(commit.sha),
        content: (
          <>
            <span className="picker-title">{commit.title}</span>
            <span className="picker-meta">
              <code className="picker-ref">
                <b>{commit.sha.slice(0, prefix)}</b>
                {commit.sha.slice(prefix, shortSha)}
              </code>
              <span aria-hidden="true">·</span>
              <Avatar
                source={commit.avatar ? { kind: "url", value: commit.avatar } : null}
                name={commit.author}
              />
              <span className="picker-author">{commit.author}</span>
              {commit.timestampMillis > 0 && (
                <>
                  <span aria-hidden="true">·</span>
                  <time dateTime={new Date(commit.timestampMillis).toISOString()}>
                    {ago(commit.timestampMillis)}
                  </time>
                </>
              )}
            </span>
          </>
        ),
      };
    }),
  ];
}
