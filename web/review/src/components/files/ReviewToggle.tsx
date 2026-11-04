/** @jsxRuntime automatic */
import { useEffect, useRef } from "react";
import type { Labels } from "../../model/diff";
import type { FileSummary } from "../../model/review";
import { ReviewStateIcon } from "./ReviewStateIcon";

export function ReviewToggle({
  file,
  labels,
  disabled,
  toggle,
}: {
  file: FileSummary;
  labels: Labels;
  disabled: boolean;
  toggle: () => void;
}) {
  const checkbox = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (checkbox.current)
      checkbox.current.indeterminate =
        file.review === "partial" || file.review === "changed_since_review";
  }, [file.review]);
  return (
    <span className="jj-review-toggle" data-state={file.review}>
      <input
        ref={checkbox}
        type="checkbox"
        checked={file.review === "reviewed"}
        disabled={disabled}
        aria-label={`${labels.markReviewed}: ${file.path}`}
        title={labels[file.review]}
        onChange={toggle}
      />
      <ReviewStateIcon state={file.review} />
    </span>
  );
}
