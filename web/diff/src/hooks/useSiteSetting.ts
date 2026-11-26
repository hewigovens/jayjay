import { useState } from "react";

/** A site-wide on/off setting kept in the browser, off until the reader turns it on. */
export function useSiteSetting(name: string): [boolean, () => void] {
  const key = `jayjay.diff.${name}`;
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
