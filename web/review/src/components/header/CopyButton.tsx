/** @jsxRuntime automatic */
import { useEffect, useState } from "react";
import { CheckIcon, CopyIcon } from "../icons";
import { ToolButton } from "../ToolButton";

export function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) return;
    const timer = setTimeout(() => setCopied(false), 1500);
    return () => clearTimeout(timer);
  }, [copied]);
  return (
    <ToolButton
      className="jj-copy-button"
      label={label}
      onClick={() =>
        void navigator.clipboard?.writeText(value).then(
          () => setCopied(true),
          () => undefined,
        )
      }
    >
      {copied ? <CheckIcon /> : <CopyIcon />}
    </ToolButton>
  );
}
