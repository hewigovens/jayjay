import { useEffect, useLayoutEffect, useRef, useState, type RefObject } from "react";
import type { FileSummary, ReviewResult } from "../model/review";

export function useFolding(
  detail: RefObject<HTMLElement | null>,
  result: ReviewResult | null,
  compact: boolean,
  draftPath: string | undefined,
) {
  const [unfolded, setUnfolded] = useState<Set<string>>(() => new Set());
  const foldedBefore = useRef<{ session?: number; paths: string[] }>({ paths: [] });

  useEffect(() => {
    const files = result?.overview.files ?? [];
    setUnfolded((previous) => {
      const kept = [...previous].filter((path) =>
        files.some((entry) => entry.path === path && entry.review === "reviewed"),
      );
      return kept.length === previous.size ? previous : new Set(kept);
    });
  }, [result]);

  // Only the phone-width list folds reviewed files, to shorten the scroll; a note draft keeps its file open, since folding would discard the text.
  const folded = (entry: FileSummary) =>
    compact && entry.review === "reviewed" && !unfolded.has(entry.path) && draftPath !== entry.path;
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
    if (section && section.getBoundingClientRect().top < 0)
      section.scrollIntoView({ block: "start" });
    if (document.activeElement === document.body)
      (section ?? detail.current)
        ?.querySelector<HTMLElement>(".jj-file-heading input")
        ?.focus({ preventScroll: true });
  }, [result?.session, foldedPaths.join("\0")]);

  const unfold = (path: string) => setUnfolded((previous) => new Set(previous).add(path));

  return { folded, unfold };
}
