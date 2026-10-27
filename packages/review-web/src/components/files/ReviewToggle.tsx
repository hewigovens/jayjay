/** @jsxRuntime automatic */
import { useEffect, useRef } from "react";
import type { Labels } from "../../model/diff";
import type { FileSummary } from "../../model/review";

export function ReviewToggle({ file, labels, disabled, toggle }: {
  file: FileSummary; labels: Labels; disabled: boolean; toggle: () => void;
}) {
  const checkbox = useRef<HTMLInputElement>(null);
  useEffect(() => { if (checkbox.current) checkbox.current.indeterminate = file.review === "partial" || file.review === "changed_since_review"; }, [file.review]);
  return <input ref={checkbox} type="checkbox" className="review-toggle" data-state={file.review} checked={file.review === "reviewed"} disabled={disabled}
    aria-label={`${labels.markReviewed}: ${file.path}`} title={labels[file.review]} onChange={toggle} />;
}
