/** @jsxRuntime automatic */
import { useId, useState } from "react";
import { lineLabel } from "../diff/DiffPane";
import type { DiffAnchor, Labels } from "../../model/diff";
import { MiddlePath } from "../files/MiddlePath";
import type { ReviewNote } from "../../model/review";

export interface NoteTarget { file: number; path: string; anchor: DiffAnchor; ignoreWhitespace: boolean; text: string | undefined }

export const anchorKey = (path: string, side: string, line: number) => `${path}|${side}:${line}`;

export function NoteBubble({ item, labels, disabled, resolve, locate }: {
  item: ReviewNote; labels: Labels; disabled: boolean; resolve: (id: string) => void;
  locate?: () => void;
}) {
  const { note, status } = item;
  return <article className={`jj-review-note jj-${status}`}>
    <div className="jj-note-heading">
      <span className="jj-note-status">{labels[status]}</span>
      {locate && <button className="jj-note-location" title={`${note.path}:${note.line}`} onClick={locate}>
        <MiddlePath path={note.path} /><span>:{note.line}</span></button>}
    </div>
    <p>{note.body}</p>
    {status === "stale" && note.anchor_excerpt && <blockquote>{note.anchor_excerpt}</blockquote>}
    {!note.resolved && <div className="jj-note-actions"><button disabled={disabled} onClick={() => resolve(note.id)}>{labels.resolve}</button></div>}
  </article>;
}

export function NoteComposer({ target, labels, disabled, save, cancel }: {
  target: NoteTarget; labels: Labels; disabled: boolean; save: (body: string) => Promise<boolean>; cancel: () => void;
}) {
  const bodyId = useId();
  const [body, setBody] = useState("");
  return <form className="jj-note-composer" onSubmit={event => { event.preventDefault(); void save(body); }}
    onKeyDown={event => { if (event.key === "Escape") cancel(); }}>
    <div className="jj-note-heading"><label htmlFor={bodyId}>{labels.noteBody}</label><span className="jj-note-line">{lineLabel(labels, target.anchor.side, target.anchor.line)}</span></div>
    <textarea id={bodyId} value={body} maxLength={16000} autoFocus onChange={event => setBody(event.target.value)} />
    <div className="jj-note-actions"><button type="button" onClick={cancel}>{labels.cancel}</button>
      <button className="jj-primary" disabled={disabled || !body.trim()}>{labels.saveNote}</button></div>
  </form>;
}

export function NotesBanner({ items, labels, disabled, resolve, locate }: {
  items: ReviewNote[]; labels: Labels; disabled: boolean; resolve: (id: string) => void; locate: (item: ReviewNote) => void;
}) {
  if (!items.length) return null;
  return <section className="jj-notes-banner" aria-label={labels.notes}>
    {items.map(item => <NoteBubble key={item.note.id} item={item} labels={labels} disabled={disabled} resolve={resolve} locate={() => locate(item)} />)}
  </section>;
}
