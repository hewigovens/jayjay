import { useEffect, useRef, type FocusEvent, type PointerEvent } from "react";

// Row handlers that run `action` once a mouse rests on the row or the keyboard focuses it, and not for a pass or a tap.
export function useHoverIntent() {
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const cancel = () => clearTimeout(timer.current);
  useEffect(() => cancel, []);
  const wait = (action: () => void) => {
    cancel();
    timer.current = setTimeout(action, 120);
  };
  return (action: (() => void) | undefined) =>
    action && {
      onPointerEnter: (event: PointerEvent) => {
        if (event.pointerType === "mouse") wait(action);
      },
      onFocus: (event: FocusEvent) => {
        if (event.currentTarget.matches(":focus-visible")) wait(action);
      },
      onPointerLeave: cancel,
      onPointerDown: cancel,
      onBlur: cancel,
    };
}
