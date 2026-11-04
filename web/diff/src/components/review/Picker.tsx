import { useEffect, useId, useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { ToolButton } from "@jayjay/review-web/react";

export interface PickerItem {
  key: string;
  current: boolean;
  content: ReactNode;
  pick: () => void;
}

export function Picker({
  label,
  button,
  items,
  autoFocus,
}: {
  label: string;
  button: ReactNode;
  items: PickerItem[];
  // The review remounts its header with what it opens, so the picker takes the focus back after a pick.
  autoFocus: boolean;
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const toggle = useRef<HTMLButtonElement>(null);
  const id = useId();

  useEffect(() => {
    if (!open) return;
    root.current?.querySelector<HTMLElement>("[aria-current='true']")?.focus();
    const away = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    addEventListener("pointerdown", away);
    return () => removeEventListener("pointerdown", away);
  }, [open]);

  const close = () => {
    setOpen(false);
    toggle.current?.focus();
  };
  const keyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      return;
    }
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (!step) return;
    event.preventDefault();
    const rows = [...(root.current?.querySelectorAll<HTMLElement>(".picker-row") ?? [])];
    rows[
      Math.max(
        0,
        Math.min(rows.length - 1, rows.indexOf(document.activeElement as HTMLElement) + step),
      )
    ]?.focus();
  };

  return (
    <div className="picker" ref={root} onKeyDown={open ? keyDown : undefined}>
      <ToolButton
        ref={toggle}
        className="picker-toggle"
        autoFocus={autoFocus}
        aria-expanded={open}
        aria-controls={open ? id : undefined}
        label={label}
        onClick={() => setOpen(!open)}
      >
        {button}
      </ToolButton>
      {open && (
        <ol id={id} className="picker-menu" aria-label={label}>
          {items.map((item) => (
            <li key={item.key}>
              <button
                className="picker-row"
                aria-current={item.current}
                onClick={() => {
                  close();
                  if (!item.current) item.pick();
                }}
              >
                {item.content}
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}
