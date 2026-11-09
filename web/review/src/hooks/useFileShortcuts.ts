import { useEffect, useRef } from "react";

export interface FileNavigation {
  /** One element per file in display order, each with its index in `data-file`. */
  rows: () => HTMLElement[];
  /** Where J and K start from, given the focused row's position or -1. */
  current: (rows: HTMLElement[], focused: number) => number;
  reviewed: (file: number) => boolean;
  go: (row: HTMLElement) => void;
  /** Rows that also move with the arrow keys while focus is inside. */
  list: () => HTMLElement | null | undefined;
  search: () => void;
  showShortcuts: () => void;
}

export function useFileShortcuts(navigation: FileNavigation) {
  const latest = useRef(navigation);
  latest.current = navigation;
  useEffect(() => {
    function keyDown(event: KeyboardEvent) {
      const { rows, current, reviewed, go, list, search, showShortcuts } = latest.current;
      const target = event.target as HTMLElement;
      if (
        event.defaultPrevented ||
        event.metaKey ||
        event.ctrlKey ||
        event.altKey ||
        target.closest("input, textarea, select, [contenteditable], dialog")
      )
        return;
      const inList = Boolean(list()?.contains(target));
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      if (key === "/") {
        event.preventDefault();
        search();
        return;
      }
      if (event.key === "?") {
        event.preventDefault();
        showShortcuts();
        return;
      }
      const step = { j: 1, k: -1, n: 1, p: -1, ...(inList && { ArrowDown: 1, ArrowUp: -1 }) }[key];
      if (!step) return;
      event.preventDefault();
      const all = rows();
      const unreviewed = key === "n" || key === "p";
      const at = current(all, all.indexOf(document.activeElement as HTMLElement));
      for (let index = at + step; index >= 0 && index < all.length; index += step) {
        if (unreviewed && reviewed(Number(all[index].dataset.file))) continue;
        go(all[index]);
        return;
      }
    }
    addEventListener("keydown", keyDown);
    return () => removeEventListener("keydown", keyDown);
  }, []);
}
