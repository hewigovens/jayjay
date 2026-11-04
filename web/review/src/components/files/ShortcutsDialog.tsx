/** @jsxRuntime automatic */
import type { Labels } from "../../model/diff";
import { Dialog } from "../Dialog";

export function ShortcutsDialog({ labels, onClose }: { labels: Labels; onClose: () => void }) {
  const rows: [string[], string][] = [
    [["Space"], labels.shortcutMark],
    [["J", "↓"], labels.shortcutNext],
    [["K", "↑"], labels.shortcutPrevious],
    [["N"], labels.shortcutNextUnreviewed],
    [["P"], labels.shortcutPreviousUnreviewed],
    [["/"], labels.filterFiles],
    [["?"], labels.shortcuts],
  ];
  return (
    <Dialog className="jj-shortcuts" label={labels.shortcuts} onClose={onClose}>
      <h2>{labels.shortcuts}</h2>
      <dl>
        {rows.map(([keys, label]) => (
          <div key={label}>
            <dt>
              {keys.map((key) => (
                <kbd key={key}>{key}</kbd>
              ))}
            </dt>
            <dd>{label}</dd>
          </div>
        ))}
      </dl>
    </Dialog>
  );
}
