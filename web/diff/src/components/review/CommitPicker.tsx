import type { Labels } from "@jayjay/review-web/types";
import type { PullCommit } from "../../github/PullManifest";
import { CommitIcon } from "../icons";
import { Picker } from "./Picker";

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
  return (
    <Picker
      label={labels.pullCommits}
      autoFocus={autoFocus}
      button={
        <>
          <CommitIcon />
          <span>{position ? `${position}/${commits.length}` : commits.length}</span>
        </>
      }
      items={[
        {
          key: "",
          current: !selected,
          content: <span className="picker-title">{labels.pullAllCommits}</span>,
          pick: () => onSelect(undefined),
        },
        ...[...commits].reverse().map((commit) => ({
          key: commit.sha,
          current: commit.sha === selected,
          pick: () => onSelect(commit.sha),
          content: (
            <>
              <code className="picker-ref">{commit.sha.slice(0, 7)}</code>
              <span className="picker-title">{commit.title}</span>
              <span className="picker-meta">
                {commit.author}
                {commit.timestampMillis > 0 && (
                  <>
                    {" "}
                    ·{" "}
                    <time dateTime={new Date(commit.timestampMillis).toISOString()}>
                      {ago(commit.timestampMillis)}
                    </time>
                  </>
                )}
              </span>
            </>
          ),
        })),
      ]}
    />
  );
}
