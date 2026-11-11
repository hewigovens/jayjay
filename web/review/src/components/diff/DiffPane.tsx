/** @jsxRuntime automatic */
import { useRef, useState, type CSSProperties, type ReactNode } from "react";
import { diffThemeAttributes, type DiffThemeSelection } from "../../themes";
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

const rowLimit = 2000;

export function lineLabel(labels: Labels, side: DiffAnchor["side"], line: number) {
  return `${side === "new" ? labels.after : labels.before} ${line}`;
}

// On touch the first tap on a gutter only reveals its add-note badge; the next tap there opens the composer.
interface GutterTouch {
  armed: string | null;
  arm: (key: string | null) => void;
  touch: { current: boolean };
}

function Cell({
  cell,
  split,
  labels,
  review,
  gutter,
}: {
  cell: DiffCell;
  split: boolean;
  labels: Labels;
  review?: ReviewControls;
  gutter: GutterTouch;
}) {
  const key = cell.anchor ? `${cell.anchor.side}:${cell.anchor.line}` : "";
  const marker = cell.style === "added" ? "+" : cell.style === "removed" ? "−" : " ";
  return (
    <div
      className={`jj-diff-cell jj-${cell.style}`}
      data-side={cell.anchor?.side}
      data-line={cell.anchor?.line}
      tabIndex={-1}
    >
      <span className="jj-line-number" aria-hidden="true">
        {cell.oldLine}
      </span>
      {!split && (
        <span className="jj-line-number" aria-hidden="true">
          {cell.newLine}
        </span>
      )}
      {review?.note && cell.anchor && (
        <button
          className={`jj-gutter-note ${gutter.armed === key ? "jj-armed" : ""}`}
          disabled={review.disabled}
          aria-label={`${labels.addNote}: ${lineLabel(labels, cell.anchor.side, cell.anchor.line)}`}
          title={labels.addNote}
          onPointerDown={(event) => {
            gutter.touch.current = event.pointerType === "touch";
          }}
          onKeyDown={() => {
            gutter.touch.current = false;
          }}
          onClick={() => {
            if (gutter.touch.current && gutter.armed !== key) {
              gutter.arm(key);
              return;
            }
            gutter.arm(null);
            review.note!(cell.anchor!);
          }}
        >
          <span aria-hidden="true">+</span>
        </button>
      )}
      <span
        className="jj-marker"
        aria-label={
          cell.style === "added" || cell.style === "removed" ? labels[cell.style] : undefined
        }
      >
        {marker}
      </span>
      <code>
        {cell.spans.map((span, index) => (
          <span
            key={index}
            className={`jj-token-${span.token} jj-word-${span.style}`}
            title={span.tooltip ?? undefined}
          >
            {span.text}
          </span>
        ))}
        {cell.noEofNewline && (
          <span className="jj-eof" title={labels.eof}>
            {" "}
            ↵ {labels.eof}
          </span>
        )}
      </code>
      {cell.anchor && review?.marker && (
        <span className="jj-line-actions">{review.marker(cell.anchor)}</span>
      )}
    </div>
  );
}

/** A change group's rows share one gutter stripe, coloured by its review state and doubling as its reviewed checkbox. */
export function DiffPane({
  view,
  split,
  showAll,
  labels,
  empty = labels.empty,
  review,
  diffThemes,
}: {
  view: DiffView;
  split: boolean;
  showAll: () => void;
  labels: Labels;
  empty?: string;
  review?: ReviewControls;
  diffThemes?: DiffThemeSelection;
}) {
  const [armed, arm] = useState<string | null>(null);
  const [limit, setLimit] = useState(rowLimit);
  const touch = useRef(false);
  const gutter = { armed, arm, touch };
  if (view.rows.length === 0) return <div className="jj-empty-diff">{empty}</div>;
  const lastLine = view.rows.reduce(
    (last, row) =>
      row.cells.reduce(
        (line, cell) => Math.max(line, Number(cell.oldLine) || 0, cell.newLine ?? 0),
        last,
      ),
    0,
  );
  const groupAt = (index: number) =>
    view.rows[index]?.cells.find((cell) => cell.anchor)?.anchor?.group;
  const splits = (index: number) =>
    index > 0 && groupAt(index) !== undefined && groupAt(index) === groupAt(index - 1);
  // A group's checkbox marks all its rows, so the cut backs off to the group's start.
  let end = Math.min(limit, view.rows.length);
  let start = end;
  while (splits(start)) start--;
  if (start > 0) end = start;
  // A note or draft past the cut keeps its row rendered.
  if (review?.annotation)
    for (let index = view.rows.length - 1; index >= end; index--)
      if (view.rows[index].cells.some((cell) => cell.anchor && review.annotation!(cell.anchor))) {
        end = index + 1;
        while (splits(end)) end++;
        break;
      }
  const cutGroup = splits(end) ? groupAt(end) : undefined;
  const blocks: ReactNode[] = [];
  let pending: { group: number; rows: ReactNode[] } | null = null;
  const flush = () => {
    if (!pending) return;
    const { group, rows } = pending;
    const state = review?.states[group] ?? "unreviewed";
    blocks.push(
      <div className={`jj-review-group jj-${state}`} key={`group:${blocks.length}`}>
        {review && (
          <input
            type="checkbox"
            className="jj-group-toggle"
            checked={state === "reviewed"}
            disabled={review.disabled || group === cutGroup}
            aria-label={`${labels.group} ${group + 1} · ${labels[state]}`}
            title={`${labels.group} ${group + 1} · ${labels[state]}`}
            onChange={() => review.toggle(group)}
          />
        )}
        {rows}
      </div>,
    );
    pending = null;
  };
  view.rows.slice(0, end).forEach((row, index) => {
    if (row.cells[0].style === "separator") {
      flush();
      blocks.push(
        <button className="jj-context-row" key={index} onClick={showAll} title={labels.showAll}>
          ↕ {row.cells[0].spans.map((span) => span.text).join("")} · {labels.showAll}
        </button>,
      );
      return;
    }
    const group = row.cells.find((cell) => cell.anchor)?.anchor?.group;
    const cells = row.fullWidth ? row.cells.slice(-1) : row.cells;
    const line = (
      <div className={`jj-diff-row ${row.fullWidth ? "jj-full-width" : ""}`} key={index}>
        {cells.map((cell, i) => (
          <Cell key={i} cell={cell} split={split} labels={labels} review={review} gutter={gutter} />
        ))}
      </div>
    );
    const annotations = cells.map((cell) => cell.anchor && review?.annotation?.(cell.anchor));
    const notes = annotations.some(Boolean) && (
      <div className="jj-note-row" key={`notes:${index}`}>
        {annotations.map((content, i) => {
          if (!content) return null;
          const whitespace = cells[i].spans
            .map((span) => span.text)
            .join("")
            .match(/^[ \t]*/)![0];
          const indent = [...whitespace].reduce(
            (column, character) => column + (character === "\t" ? 4 - (column % 4) : 1),
            0,
          );
          return (
            <div
              className={`jj-note-slot jj-${cells[i].anchor!.side}`}
              style={{ "--jj-note-indent": `${indent}ch` } as CSSProperties}
              key={i}
            >
              {content}
            </div>
          );
        })}
      </div>
    );
    if (group === undefined) {
      flush();
      blocks.push(line, notes);
      return;
    }
    if (pending?.group !== group) {
      flush();
      pending = { group, rows: [] };
    }
    pending!.rows.push(line, notes);
  });
  flush();
  if (end < view.rows.length)
    blocks.push(
      <button className="jj-context-row" key="rest" onClick={() => setLimit(Infinity)}>
        ↓ {labels.showRest}
      </button>,
    );
  return (
    <div
      className={`jj-diff-lines ${split ? "jj-split" : "jj-unified"}`}
      style={{ "--jj-line-digits": Math.max(String(lastLine).length, 2) } as CSSProperties}
      {...diffThemeAttributes(diffThemes)}
    >
      {split && (
        <div className="jj-split-head">
          <span>{labels.before}</span>
          <span>{labels.after}</span>
        </div>
      )}
      {blocks}
    </div>
  );
}
