/** @jsxRuntime automatic */
import type { ReviewState } from "../../model/review";

export function ReviewStateIcon({ state, size = 16 }: { state: ReviewState; size?: number }) {
  return (
    <svg
      className="jj-icon jj-review-icon"
      data-state={state}
      viewBox="0 0 24 24"
      width={size}
      height={size}
      aria-hidden="true"
    >
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
      {state === "changed_since_review" && <circle cx="12" cy="12" r="3" fill="currentColor" />}
      {state === "partial" && (
        <path d="M8 12h8" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
      )}
      {state === "reviewed" && (
        <path
          d="M20 6 9 17l-5-5"
          transform="translate(6 7) scale(0.5)"
          fill="none"
          stroke="currentColor"
          strokeWidth="4.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
    </svg>
  );
}
