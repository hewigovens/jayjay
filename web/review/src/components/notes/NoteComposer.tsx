/** @jsxRuntime automatic */
import { useId, useState } from "react";
import type { DiffAnchor, Labels } from "../../model/diff";
import { lineLabel } from "../diff/DiffPane";

export interface NoteTarget {
  file: number;
  path: string;
  anchor: DiffAnchor;
  ignoreWhitespace: boolean;
  text: string | undefined;
}

export function NoteComposer({
  target,
  labels,
  disabled,
  save,
  cancel,
}: {
  target: NoteTarget;
  labels: Labels;
  disabled: boolean;
  save: (body: string) => Promise<boolean>;
  cancel: () => void;
}) {
  const bodyId = useId();
  const [body, setBody] = useState("");
  return (
    <form
      className="jj-note-composer"
      onSubmit={(event) => {
        event.preventDefault();
        void save(body);
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") cancel();
      }}
    >
      <div className="jj-note-heading">
        <label htmlFor={bodyId}>{labels.noteBody}</label>
        <span className="jj-note-line">
          {lineLabel(labels, target.anchor.side, target.anchor.line)}
        </span>
      </div>
      <textarea
        id={bodyId}
        value={body}
        maxLength={16000}
        autoFocus
        onChange={(event) => setBody(event.target.value)}
      />
      <div className="jj-note-actions">
        <button type="button" onClick={cancel}>
          {labels.cancel}
        </button>
        <button className="jj-primary" disabled={disabled || !body.trim()}>
          {labels.saveNote}
        </button>
      </div>
    </form>
  );
}
