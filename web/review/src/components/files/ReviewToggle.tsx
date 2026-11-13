/** @jsxRuntime automatic */
import { useEffect, useRef } from "react";
import type { Labels } from "../../model/diff";
import type { ReviewState } from "../../model/review";
import { ReviewStateIcon } from "./ReviewStateIcon";

export function ReviewToggle({
  review,
  path,
  labels,
  disabled,
  toggle,
}: {
  review: ReviewState;
  path: string;
  labels: Labels;
  disabled: boolean;
  toggle: () => void;
}) {
  const checkbox = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (checkbox.current)
      checkbox.current.indeterminate = review === "partial" || review === "changed_since_review";
  }, [review]);
  return (
    <span className="jj-review-toggle" data-state={review}>
      <input
        ref={checkbox}
        type="checkbox"
        checked={review === "reviewed"}
        disabled={disabled}
        aria-label={`${labels.markReviewed}: ${path}`}
        title={labels[review]}
        onChange={toggle}
      />
      <ReviewStateIcon state={review} />
    </span>
  );
}
