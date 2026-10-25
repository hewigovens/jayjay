/** @jsxRuntime automatic */
import { useEffect, useState } from "react";
import { CheckIcon, CopyIcon } from "../icons";

export function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);
  useEffect(() => { if (!copied) return; const timer = setTimeout(() => setCopied(false), 1500); return () => clearTimeout(timer); }, [copied]);
  return <button className="tool copy-button" aria-label={label} title={label}
    onClick={() => void navigator.clipboard?.writeText(value).then(() => setCopied(true), () => undefined)}>{copied ? <CheckIcon /> : <CopyIcon />}</button>;
}
