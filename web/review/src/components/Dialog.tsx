/** @jsxRuntime automatic */
import { useEffect, useRef, type MouseEvent, type ReactNode } from "react";

// A click on the backdrop targets the dialog itself but lands outside its box; keyboard clicks have no position.
function outside(event: MouseEvent<HTMLDialogElement>) {
  if (event.target !== event.currentTarget || event.detail === 0) return false;
  const box = event.currentTarget.getBoundingClientRect();
  return (
    event.clientX < box.left ||
    event.clientX > box.right ||
    event.clientY < box.top ||
    event.clientY > box.bottom
  );
}

export function Dialog({
  className,
  label,
  labelledBy,
  open = true,
  onClose,
  children,
}: {
  className?: string;
  label?: string;
  labelledBy?: string;
  open?: boolean;
  onClose: () => void;
  children: ReactNode;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = dialog.current;
    if (open && !element?.open) element?.showModal();
    else if (!open && element?.open) element.close();
  }, [open]);
  return (
    <dialog
      ref={dialog}
      className={className ? `jj-dialog ${className}` : "jj-dialog"}
      aria-label={label}
      aria-labelledby={labelledBy}
      onClose={onClose}
      onClick={(event) => {
        if (outside(event)) event.currentTarget.close();
      }}
    >
      {children}
    </dialog>
  );
}
