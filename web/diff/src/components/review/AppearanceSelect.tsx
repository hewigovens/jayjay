import { useId, useRef } from "react";
import { Icon } from "@jayjay/review-web/react";

export function AppearanceSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: readonly { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  const id = useId();
  const button = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const selected = options.find((option) => option.value === value);
  const close = () => {
    menu.current?.hidePopover();
    button.current?.focus();
  };
  return (
    <div className="diff-theme-row">
      <span id={`${id}-label`}>{label}</span>
      <button
        ref={button}
        className="appearance-select"
        popoverTarget={id}
        onClick={(event) => {
          event.preventDefault();
          const element = menu.current;
          if (!element) return;
          if (element.matches(":popover-open")) {
            close();
            return;
          }
          element.showPopover();
          const box = event.currentTarget.getBoundingClientRect();
          const below = innerHeight - box.bottom - 16;
          const above = box.top - 16;
          const down = below >= Math.min(element.scrollHeight, 280) || below >= above;
          element.style.width = `${Math.min(Math.max(box.width, 180), innerWidth - 16)}px`;
          element.style.maxHeight = `${Math.min(280, Math.max(0, down ? below : above))}px`;
          element.style.left = `${Math.max(8, Math.min(box.right - element.offsetWidth, innerWidth - element.offsetWidth - 8))}px`;
          element.style.top = `${down ? box.bottom + 6 : box.top - element.offsetHeight - 6}px`;
          element.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
        }}
        aria-haspopup="menu"
        aria-labelledby={`${id}-label ${id}-value`}
      >
        <span id={`${id}-value`}>{selected?.label}</span>
        <Icon d="m4 6 4 4 4-4" />
      </button>
      <div
        id={id}
        ref={menu}
        popover="auto"
        role="menu"
        aria-label={label}
        className="appearance-menu"
        onKeyDown={(event) => {
          if (event.key === "Escape" || event.key === "Tab") {
            if (event.key === "Escape") {
              event.preventDefault();
              event.stopPropagation();
            }
            close();
            return;
          }
          const keys = ["ArrowDown", "ArrowUp", "Home", "End"];
          if (!keys.includes(event.key)) return;
          event.preventDefault();
          const items = [...event.currentTarget.querySelectorAll<HTMLButtonElement>("button")];
          const current = items.indexOf(document.activeElement as HTMLButtonElement);
          const next =
            event.key === "Home"
              ? 0
              : event.key === "End"
                ? items.length - 1
                : (current + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
          items[next]?.focus();
        }}
      >
        {options.map((option) => (
          <button
            key={option.value}
            role="menuitemradio"
            aria-checked={value === option.value}
            onClick={() => {
              onChange(option.value);
              close();
            }}
          >
            {option.label}
          </button>
        ))}
      </div>
    </div>
  );
}
