/** @jsxRuntime automatic */
import { useRef, useState } from "react";
import { NoteComposer } from "./NoteComposer";
import type { Labels } from "../../model/diff";
import type { ReviewNote } from "../../model/review";
import { CheckIcon, Icon } from "../icons";
import { MiddlePath } from "../files/MiddlePath";

export function NoteBubble({
  item,
  labels,
  disabled,
  resolve,
  update,
  locate,
}: {
  item: ReviewNote;
  labels: Labels;
  disabled: boolean;
  resolve: (id: string) => void;
  update: (id: string, body: string) => Promise<boolean>;
  locate?: () => void;
}) {
  const { note, status } = item;
  const summary = Boolean(locate) && (status === "stale" || status === "orphaned");
  const [editing, setEditing] = useState(false);
  const editButton = useRef<HTMLButtonElement>(null);
  function closeEditor() {
    setEditing(false);
    requestAnimationFrame(() => editButton.current?.focus());
  }
  if (editing)
    return (
      <NoteComposer
        target={{ anchor: note }}
        initialBody={note.body}
        labels={labels}
        disabled={disabled}
        save={async (body) => {
          const saved = await update(note.id, body);
          if (saved) closeEditor();
          return saved;
        }}
        cancel={closeEditor}
      />
    );
  return (
    <article className={`jj-review-note jj-${status} ${summary ? "jj-note-summary" : ""}`}>
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
        <p title={summary ? note.body : undefined}>{note.body}</p>
        <button
          ref={editButton}
          className="jj-note-action"
          disabled={disabled}
          aria-label={labels.editNote}
          title={labels.editNote}
          onClick={() => setEditing(true)}
        >
          <Icon size={12} d="M10 2l4 4M2 10l8-8 4 4-8 8H2Z" />
        </button>
        {!note.resolved && (
          <button
            className={summary ? "jj-note-resolve-link" : "jj-note-action"}
            disabled={disabled}
            aria-label={labels.resolve}
            title={labels.resolve}
            onClick={() => resolve(note.id)}
          >
            {summary ? labels.resolve : <CheckIcon />}
          </button>
        )}
      </div>
      {!summary && status === "stale" && note.anchor_excerpt && (
        <blockquote>{note.anchor_excerpt}</blockquote>
      )}
    </article>
  );
}
