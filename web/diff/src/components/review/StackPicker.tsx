import type { Labels } from "@jayjay/review-web/types";
import type { PullStackEntry } from "../../github/PullManifest";
import { StackIcon } from "../icons";
import { Picker } from "./Picker";

export function StackPicker({
  stack,
  number,
  labels,
  autoFocus,
  onSelect,
}: {
  stack: PullStackEntry[];
  number: number;
  labels: Labels;
  autoFocus: boolean;
  onSelect: (number: number) => void;
}) {
  const state = { open: "", merged: labels.pullMerged, closed: labels.pullClosed };
  return (
    <Picker
      label={labels.pullStack}
      autoFocus={autoFocus}
      button={
        <>
          <StackIcon />
          <span>{`${stack.findIndex((entry) => entry.number === number) + 1}/${stack.length}`}</span>
        </>
      }
      // Top first, like a change log.
      items={[...stack].reverse().map((entry) => ({
        key: String(entry.number),
        current: entry.number === number,
        pick: () => onSelect(entry.number),
        content: (
          <>
            <code className="picker-ref">#{entry.number}</code>
            <span className="picker-title">{entry.title}</span>
            {state[entry.state] && <span className="picker-meta">{state[entry.state]}</span>}
          </>
        ),
      }))}
    />
  );
}
