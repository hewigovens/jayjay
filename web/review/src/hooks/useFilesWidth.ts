import { useState } from "react";

const storageKey = "jayjay.review.filesWidth";
export const minFilesWidth = 180;

function stored() {
  try {
    const width = Number(localStorage.getItem(storageKey));
    return width >= minFilesWidth ? width : null;
  } catch {
    return null;
  }
}

export function useFilesWidth() {
  const [width, setWidth] = useState<number | null>(stored);
  const commit = (value: number | null) => {
    setWidth(value);
    try {
      if (value === null) localStorage.removeItem(storageKey);
      else localStorage.setItem(storageKey, String(value));
    } catch {
      /* The width still applies for this visit. */
    }
  };
  return { width, setWidth, commit };
}
