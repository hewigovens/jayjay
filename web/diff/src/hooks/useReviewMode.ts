import { useState } from "react";

const key = "jayjay.diff.reviewMode";

/** The site-wide review mode, off until the reader turns it on. */
export function useReviewMode(): [boolean, () => void] {
  const [on, setOn] = useState(() => {
    try {
      return localStorage.getItem(key) === "on";
    } catch {
      return false;
    }
  });
  return [
    on,
    () => {
      setOn(!on);
      try {
        localStorage.setItem(key, on ? "off" : "on");
      } catch {
        return;
      }
    },
  ];
}
