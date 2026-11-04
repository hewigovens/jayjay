/** @jsxRuntime automatic */
import type { ComponentPropsWithRef } from "react";

type ToolProps<T extends "button" | "a"> = Omit<ComponentPropsWithRef<T>, "aria-label"> & {
  label: string;
};

const toolClass = (className?: string) => (className ? `jj-tool ${className}` : "jj-tool");

export function ToolButton({ label, title, className, ...props }: ToolProps<"button">) {
  return (
    <button {...props} className={toolClass(className)} aria-label={label} title={title ?? label} />
  );
}

export function ToolLink({ label, title, className, ...props }: ToolProps<"a">) {
  return (
    <a {...props} className={toolClass(className)} aria-label={label} title={title ?? label} />
  );
}
