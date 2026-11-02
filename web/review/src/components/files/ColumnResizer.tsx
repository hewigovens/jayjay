/** @jsxRuntime automatic */
import { useRef, type KeyboardEvent, type PointerEvent, type RefObject } from "react";
import { minFilesWidth as min } from "../../hooks/useFilesWidth";

const step = 16;

export function ColumnResizer({
  columns,
  width,
  setWidth,
  commit,
  label,
}: {
  columns: RefObject<HTMLElement | null>;
  width: number | null;
  setWidth: (width: number | null) => void;
  commit: (width: number | null) => void;
  label: string;
}) {
  const drag = useRef<{ x: number; width: number } | null>(null);
  const max = () => Math.max(min, Math.round((columns.current?.clientWidth ?? 0) * 0.6));
  const clamp = (value: number) => Math.min(max(), Math.max(min, Math.round(value)));
  const current = () =>
    columns.current?.querySelector(".jj-file-column")?.getBoundingClientRect().width ?? min;

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
    const next = {
      ArrowLeft: current() - step,
      ArrowRight: current() + step,
      Home: min,
      End: max(),
    }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    commit(clamp(next));
  };

  return (
    <div
      className="jj-column-resizer"
      role="separator"
      aria-orientation="vertical"
      aria-label={label}
      title={label}
      tabIndex={0}
      aria-valuemin={min}
      aria-valuemax={max()}
      aria-valuenow={Math.round(width ?? current())}
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onPointerCancel={up}
      onKeyDown={keyDown}
      onDoubleClick={() => commit(null)}
    />
  );
}
