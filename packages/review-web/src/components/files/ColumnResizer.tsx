/** @jsxRuntime automatic */
import { useRef, useState, type KeyboardEvent, type PointerEvent, type RefObject } from "react";

const storageKey = "jayjay.review.filesWidth";
const min = 180;
const step = 16;

function stored() {
  try {
    const width = Number(localStorage.getItem(storageKey));
    return width >= min ? width : null;
  } catch {
    return null;
  }
}

function store(width: number | null) {
  try {
    if (width === null) localStorage.removeItem(storageKey);
    else localStorage.setItem(storageKey, String(width));
  } catch { /* The width still applies for this visit. */ }
}

/** The reader's file column width in pixels, or null for the stylesheet's default. */
export function useFilesWidth() {
  return useState<number | null>(stored);
}

/** The draggable edge between the file column and the diff; double-click restores the default width. */
export function ColumnResizer({ columns, width, setWidth, label }: {
  columns: RefObject<HTMLElement | null>;
  width: number | null;
  setWidth: (width: number | null) => void;
  label: string;
}) {
  const drag = useRef<{ x: number; width: number } | null>(null);
  const max = () => Math.max(min, Math.round((columns.current?.clientWidth ?? 0) * .6));
  const clamp = (value: number) => Math.min(max(), Math.max(min, Math.round(value)));
  const current = () => columns.current?.querySelector(".jj-file-column")?.getBoundingClientRect().width ?? min;
  const commit = (value: number | null) => { setWidth(value); store(value); };

  const down = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = { x: event.clientX, width: current() };
  };
  const move = (event: PointerEvent<HTMLDivElement>) => {
    if (drag.current) setWidth(clamp(drag.current.width + event.clientX - drag.current.x));
  };
  const up = () => {
    if (!drag.current) return;
    drag.current = null;
    commit(clamp(current()));
  };
  const keyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const next = { ArrowLeft: current() - step, ArrowRight: current() + step, Home: min, End: max() }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    commit(clamp(next));
  };

  return <div className="jj-column-resizer" role="separator" aria-orientation="vertical" aria-label={label} title={label} tabIndex={0}
    aria-valuemin={min} aria-valuemax={max()} aria-valuenow={Math.round(width ?? current())}
    onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={up} onKeyDown={keyDown} onDoubleClick={() => commit(null)} />;
}
