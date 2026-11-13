/** @jsxRuntime automatic */
import type { ReviewState } from "../../model/review";

// Built from the GPUI shell's circle, check, and circle-half icons (shell/gpui/assets/icons).
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
        <circle cx="12" cy="12" r="11" fill="currentColor" />
      ) : (
        <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" strokeWidth="2" />
      )}
      {state === "changed_since_review" && <path d="M12 2a10 10 0 0 0 0 20z" fill="currentColor" />}
      {(reviewed || state === "partial") && (
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
