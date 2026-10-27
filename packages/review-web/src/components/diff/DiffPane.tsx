/** @jsxRuntime automatic */
import { useRef, useState, type ReactNode } from "react";
import type { DiffAnchor, DiffCell, DiffView, Labels } from "../../model/diff";
import type { ReviewState } from "../../model/review";

interface ReviewControls {
  states: ReviewState[];
  disabled: boolean;
  toggle: (group: number) => void;
  /** Leave out to hide the per-line note buttons. */
  note?: (anchor: DiffAnchor) => void;
  /** Content shown under an anchored line, such as its notes or a note composer. */
  annotation?: (anchor: DiffAnchor) => ReactNode;
  /** A gutter marker for the line, such as collapsed resolved notes. */
  marker?: (anchor: DiffAnchor) => ReactNode;
}

export function lineLabel(labels: Labels, side: DiffAnchor["side"], line: number) {
  return `${side === "new" ? labels.after : labels.before} ${line}`;
}

/** On touch the first tap on a gutter only reveals its add-note badge; the next tap there opens the composer. */
interface GutterTouch { armed: string | null; arm: (key: string | null) => void; touch: { current: boolean } }

function Cell({ cell, split, labels, review, gutter }: { cell: DiffCell; split: boolean; labels: Labels; review?: ReviewControls; gutter: GutterTouch }) {
  const key = cell.anchor ? `${cell.anchor.side}:${cell.anchor.line}` : "";
  const marker = cell.style === "added" ? "+" : cell.style === "removed" ? "−" : " ";
  return <div className={`diff-cell ${cell.style}`} data-side={cell.anchor?.side} data-line={cell.anchor?.line} tabIndex={-1}>
    <span className="line-number" aria-hidden="true">{cell.oldLine}</span>
    {!split && <span className="line-number" aria-hidden="true">{cell.newLine}</span>}
    {review?.note && cell.anchor && <button className={`gutter-note ${gutter.armed === key ? "armed" : ""}`} disabled={review.disabled}
      aria-label={`${labels.addNote}: ${lineLabel(labels, cell.anchor.side, cell.anchor.line)}`} title={labels.addNote}
      onPointerDown={event => { gutter.touch.current = event.pointerType === "touch"; }} onKeyDown={() => { gutter.touch.current = false; }}
      onClick={() => {
        if (gutter.touch.current && gutter.armed !== key) { gutter.arm(key); return; }
        gutter.arm(null);
        review.note!(cell.anchor!);
      }}><span aria-hidden="true">+</span></button>}
    <span className="marker" aria-label={cell.style === "added" || cell.style === "removed" ? labels[cell.style] : undefined}>{marker}</span>
    <code>{cell.spans.map((span, index) => <span key={index}
      className={`token-${span.token} word-${span.style}`} title={span.tooltip ?? undefined}>{span.text}</span>)}
      {cell.noEofNewline && <span className="eof" title={labels.eof}> ↵ {labels.eof}</span>}
    </code>
    {cell.anchor && review?.marker && <span className="line-actions">{review.marker(cell.anchor)}</span>}
  </div>;
}

/** A change group's rows share one gutter stripe, coloured by its review state and doubling as its reviewed checkbox. */
export function DiffPane({ view, split, showAll, labels, review }: { view: DiffView; split: boolean; showAll: () => void; labels: Labels; review?: ReviewControls }) {
  const [armed, arm] = useState<string | null>(null);
  const touch = useRef(false);
  const gutter = { armed, arm, touch };
  if (view.rows.length === 0) return <div className="empty-diff">{labels.empty}</div>;
  const blocks: ReactNode[] = [];
  let pending: { group: number; rows: ReactNode[] } | null = null;
  const flush = () => {
    if (!pending) return;
    const { group, rows } = pending;
    const state = review?.states[group] ?? "unreviewed";
    blocks.push(<div className={`review-group ${state}`} key={`group:${blocks.length}`}>
      {review && <input type="checkbox" className="group-toggle" checked={state === "reviewed"} disabled={review.disabled}
        aria-label={`${labels.group} ${group + 1} · ${labels[state]}`} title={`${labels.group} ${group + 1} · ${labels[state]}`}
        onChange={() => review.toggle(group)} />}
      {rows}
    </div>);
    pending = null;
  };
  view.rows.forEach((row, index) => {
    if (row.cells[0].style === "separator") {
      flush();
      blocks.push(<button className="context-row" key={index} onClick={showAll}
        title={labels.showAll}>↕ {row.cells[0].spans.map(span => span.text).join("")} · {labels.showAll}</button>);
      return;
    }
    const group = row.cells.find(cell => cell.anchor)?.anchor?.group;
    const cells = row.fullWidth ? row.cells.slice(-1) : row.cells;
    const line = <div className={`diff-row ${row.fullWidth ? "full-width" : ""}`} key={index}>
      {cells.map((cell, i) => <Cell key={i} cell={cell} split={split} labels={labels} review={review} gutter={gutter} />)}
    </div>;
    const annotations = cells.map(cell => cell.anchor && review?.annotation?.(cell.anchor));
    const notes = annotations.some(Boolean) && <div className="note-row" key={`notes:${index}`}>
      {annotations.map((content, i) => content && <div className={`note-slot ${cells[i].anchor!.side}`} key={i}>{content}</div>)}
    </div>;
    if (group === undefined) { flush(); blocks.push(line, notes); return; }
    if (pending?.group !== group) { flush(); pending = { group, rows: [] }; }
    pending!.rows.push(line, notes);
  });
  flush();
  return <div className={`diff-lines ${split ? "split" : "unified"}`}>
    {split && <div className="split-head"><span>{labels.before}</span><span>{labels.after}</span></div>}
    {blocks}
  </div>;
}
