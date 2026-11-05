/** @jsxRuntime automatic */
import type { Labels } from "../../model/diff";
import type { ReviewNote } from "../../model/review";
import { CheckIcon } from "../icons";
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
      {(locate || status !== "current") && (
        <div className="jj-note-heading">
          <span className="jj-note-status">{labels[status]}</span>
          {locate && (
            <button
              className="jj-note-location"
              title={`${note.path}:${note.line}`}
              onClick={locate}
            >
              <MiddlePath path={note.path} />
              <span>:{note.line}</span>
            </button>
          )}
        </div>
      )}
      <div className="jj-note-content">
        <p>{note.body}</p>
        {!note.resolved && (
          <button
            className="jj-note-resolve"
            disabled={disabled}
            aria-label={labels.resolve}
            title={labels.resolve}
            onClick={() => resolve(note.id)}
          >
            <CheckIcon />
          </button>
        )}
      </div>
      {status === "stale" && note.anchor_excerpt && <blockquote>{note.anchor_excerpt}</blockquote>}
    </article>
  );
}
