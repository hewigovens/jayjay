import { useEffect, useLayoutEffect, useRef, useState, type RefObject } from "react";
import type { FileSummary, ReviewResult } from "../model/review";

export function useFolding(
  detail: RefObject<HTMLElement | null>,
  result: ReviewResult | null,
  list: boolean,
  foldReviewed: boolean,
  draftPath: string | undefined,
) {
  // A reader's fold or unfold of a file, over the default of folding reviewed files.
  const [chosen, setChosen] = useState<Map<string, boolean>>(() => new Map());
  const foldedBefore = useRef<{ session?: number; paths: string[] }>({ paths: [] });

  useEffect(() => {
    const files = result?.overview.files ?? [];
    // An unfold lapses once its file is no longer reviewed, so marking it again folds it.
    setChosen((previous) => {
      const kept = [...previous].filter(([path, fold]) =>
        files.some((entry) => entry.path === path && (fold || entry.review === "reviewed")),
      );
      return kept.length === previous.size ? previous : new Map(kept);
    });
  }, [result]);

  // Only the one-list layout folds; a note draft keeps its file open, since folding would discard the text.
  const folded = (entry: FileSummary) =>
    list &&
    draftPath !== entry.path &&
    (chosen.get(entry.path) ?? (foldReviewed && entry.review === "reviewed"));
  const foldedPaths = result?.overview.files.filter(folded).map((entry) => entry.path) ?? [];

  // Folding unmounts the diff, which may hold focus, and leaves a long file's reader past its end.
  useLayoutEffect(() => {
    const { session, paths } = foldedBefore.current;
    foldedBefore.current = { session: result?.session, paths: foldedPaths };
    const fresh = foldedPaths.filter((path) => !paths.includes(path));
    if (session !== result?.session || fresh.length !== 1) return;
    const diff = detail.current?.querySelector(`[data-path="${CSS.escape(fresh[0])}"]`);
    if (!diff) return;
    const section = diff.closest(".jj-file-section");
    const floor = Math.max(0, section?.parentElement?.getBoundingClientRect().top ?? 0);
    if (section && section.getBoundingClientRect().top < floor)
      section.scrollIntoView({ block: "start" });
    if (document.activeElement === document.body)
      (section ?? detail.current)
        ?.querySelector<HTMLElement>(".jj-file-heading input")
        ?.focus({ preventScroll: true });
  }, [result?.session, foldedPaths.join("\0")]);

  const choose = (paths: string[], fold: boolean) =>
    setChosen((previous) => new Map([...previous, ...paths.map((path) => [path, fold] as const)]));
  const unfold = (path: string) => choose([path], false);
  const toggleFold = (entry: FileSummary) => choose([entry.path], !folded(entry));
  const foldAll = (fold: boolean) =>
    choose(result?.overview.files.map((entry) => entry.path) ?? [], fold);

  return { folded, unfold, toggleFold, foldAll };
}
