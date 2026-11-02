/** @jsxRuntime automatic */
import { useEffect, useRef } from "react";
import type { Labels } from "../../model/diff";

export function ShortcutsDialog({ labels, onClose }: { labels: Labels; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    dialog.current?.showModal();
  }, []);
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
    <dialog
      ref={dialog}
      className="jj-shortcuts"
      aria-label={labels.shortcuts}
      onClose={onClose}
      onClick={(event) => {
        if (event.target === dialog.current) dialog.current.close();
      }}
    >
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
    </dialog>
  );
}
