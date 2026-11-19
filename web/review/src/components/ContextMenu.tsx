/** @jsxRuntime automatic */
import { useEffect, useLayoutEffect, useRef, type KeyboardEvent } from "react";
import { CheckIcon } from "./icons";

export interface ContextMenuItem {
  label: string;
  /** Makes the item a checkbox; "mixed" when only part of what it covers is checked. */
  checked?: boolean | "mixed";
  disabled?: boolean;
  run: () => void;
}

export function ContextMenu({
  x,
  y,
  label,
  items,
  onClose,
}: {
  x: number;
  y: number;
  label: string;
  items: ContextMenuItem[];
  onClose: () => void;
}) {
  const menu = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const element = menu.current;
    if (!element) return;
    const opener = document.activeElement;
    const { width, height } = element.getBoundingClientRect();
    element.style.left = `${Math.max(4, Math.min(x, innerWidth - width - 4))}px`;
    element.style.top = `${Math.max(4, Math.min(y, innerHeight - height - 4))}px`;
    element.querySelector<HTMLElement>("button:not(:disabled)")?.focus();
    return () => {
      if (element.contains(document.activeElement) && opener instanceof HTMLElement)
        opener.focus({ preventScroll: true });
    };
  }, [x, y]);
  useEffect(() => {
    const away = (event: Event) => {
      if (!menu.current?.contains(event.target as Node)) onClose();
    };
    addEventListener("pointerdown", away, true);
    addEventListener("contextmenu", away, true);
    addEventListener("wheel", away, { capture: true, passive: true });
    addEventListener("resize", onClose);
    addEventListener("blur", onClose);
    return () => {
      removeEventListener("pointerdown", away, true);
      removeEventListener("contextmenu", away, true);
      removeEventListener("wheel", away, true);
      removeEventListener("resize", onClose);
      removeEventListener("blur", onClose);
    };
  }, [onClose]);

  function keyDown(event: KeyboardEvent) {
    if (event.key === "Escape" || event.key === "Tab") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (!step) return;
    event.preventDefault();
    const buttons = [
      ...(menu.current?.querySelectorAll<HTMLElement>("button:not(:disabled)") ?? []),
    ];
    const at = buttons.indexOf(document.activeElement as HTMLElement);
    buttons[(at + step + buttons.length) % buttons.length]?.focus();
  }

  return (
    <div
      ref={menu}
      className="jj-context-menu"
      role="menu"
      aria-label={label}
      onKeyDown={keyDown}
      // Focus that moves elsewhere, as J and K do, closes the menu; Safari's unfocused click reports no target and must not.
      onBlur={(event) => {
        if (event.relatedTarget && !event.currentTarget.contains(event.relatedTarget)) onClose();
      }}
      onContextMenu={(event) => event.preventDefault()}
    >
      {items.map((item) => (
        <button
          key={item.label}
          role={item.checked === undefined ? "menuitem" : "menuitemcheckbox"}
          aria-checked={item.checked}
          disabled={item.disabled}
          onClick={() => {
            onClose();
            item.run();
          }}
        >
          <span className="jj-menu-check">{item.checked === true && <CheckIcon />}</span>
          {item.label}
        </button>
      ))}
    </div>
  );
}
