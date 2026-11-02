/** @jsxRuntime automatic */
import type { Labels } from "../../model/diff";
import type { ReviewNote } from "../../model/review";
import { MiddlePath } from "../files/MiddlePath";

export function NoteBubble({
  item,
  labels,
  disabled,
  resolve,
  locate,
}: {
  item: ReviewNote;
  labels: Labels;
  disabled: boolean;
  resolve: (id: string) => void;
  locate?: () => void;
}) {
  const { note, status } = item;
  return (
    <article className={`jj-review-note jj-${status}`}>
      <div className="jj-note-heading">
        <span className="jj-note-status">{labels[status]}</span>
        {locate && (
          <button className="jj-note-location" title={`${note.path}:${note.line}`} onClick={locate}>
            <MiddlePath path={note.path} />
            <span>:{note.line}</span>
          </button>
        )}
      </div>
      <p>{note.body}</p>
      {status === "stale" && note.anchor_excerpt && <blockquote>{note.anchor_excerpt}</blockquote>}
      {!note.resolved && (
        <div className="jj-note-actions">
          <button disabled={disabled} onClick={() => resolve(note.id)}>
            {labels.resolve}
          </button>
        </div>
      )}
    </article>
  );
}
