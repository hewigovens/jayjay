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
  initialBody = "",
  labels,
  disabled,
  save,
  cancel,
}: {
  target: { anchor: Pick<DiffAnchor, "side" | "line"> };
  initialBody?: string;
  labels: Labels;
  disabled: boolean;
  save: (body: string) => Promise<boolean>;
  cancel: () => void;
}) {
  const bodyId = useId();
  const [body, setBody] = useState(initialBody);
  return (
    <form
      className="jj-note-composer"
      onSubmit={(event) => {
        event.preventDefault();
        if (!disabled && body.trim()) void save(body);
      }}
      onKeyDown={(event) => {
        if (event.nativeEvent.isComposing) return;
        if (event.key === "Escape" && !disabled) {
          event.preventDefault();
          event.stopPropagation();
          cancel();
        }
        if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
          event.preventDefault();
          event.stopPropagation();
          if (!disabled && body.trim() && !event.repeat) event.currentTarget.requestSubmit();
        }
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
        disabled={disabled}
        autoFocus
        onChange={(event) => setBody(event.target.value)}
      />
      <div className="jj-note-actions">
        <button type="button" disabled={disabled} onClick={cancel}>
          {labels.cancel}
        </button>
        <button className="jj-primary" disabled={disabled || !body.trim()}>
          {labels.saveNote}
        </button>
      </div>
    </form>
  );
}
