/** @jsxRuntime automatic */
import type { ReviewState } from "../../model/review";

export function ReviewStateIcon({ state, size = 16 }: { state: ReviewState; size?: number }) {
  const reviewed = state === "reviewed";
  return (
    <svg
      className="jj-icon jj-review-icon"
      data-state={state}
      viewBox="0 0 24 24"
      width={size}
      height={size}
      aria-hidden="true"
    >
      {reviewed ? (
        <rect x="2" y="2" width="20" height="20" rx="5" fill="currentColor" />
      ) : (
        <rect
          x="3"
          y="3"
          width="18"
          height="18"
          rx="4"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        />
      )}
      {state === "changed_since_review" && (
        <path d="M12 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h5z" fill="currentColor" />
      )}
      {state === "partial" && (
        <path d="M8 12h8" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      )}
      {reviewed && (
        <path
          className="jj-review-check"
          d="M20 6 9 17l-5-5"
          transform="translate(6 7) scale(0.5)"
          fill="none"
          stroke="currentColor"
          strokeWidth="6"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
    </svg>
  );
}
