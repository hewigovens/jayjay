import { useEffect, type RefObject } from "react";

export function useFileShortcuts(
  list: RefObject<HTMLElement | null>,
  selected: number,
  select: (index: number) => void,
  search: () => void,
  showShortcuts: () => void,
) {
  useEffect(() => {
    function keyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement;
      if (
        event.defaultPrevented ||
        event.metaKey ||
        event.ctrlKey ||
        event.altKey ||
        target.closest("input, textarea, select, [contenteditable], dialog")
      )
        return;
      const inList = Boolean(list.current?.contains(target));
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
      const rows = [...(list.current?.querySelectorAll<HTMLElement>(".jj-file-select") ?? [])];
      const unreviewed = key === "n" || key === "p";
      // The focused row runs ahead of `selected` while the last pick is still rendering.
      const focused = rows.indexOf(document.activeElement as HTMLElement);
      const at =
        focused >= 0 ? focused : rows.findIndex((row) => Number(row.dataset.file) === selected);
      for (let index = at + step; index >= 0 && index < rows.length; index += step) {
        if (unreviewed && rows[index].parentElement?.classList.contains("jj-reviewed")) continue;
        rows[index].focus();
        select(Number(rows[index].dataset.file));
        return;
      }
    }
    addEventListener("keydown", keyDown);
    return () => removeEventListener("keydown", keyDown);
  }, [selected, select]);
}
