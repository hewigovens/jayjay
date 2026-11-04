import { useEffect, useRef, type RefObject } from "react";
import type { ReviewNote, ReviewResult } from "../model/review";
import type { LoadedFile } from "./useLoadedSections";

export function useNoteJump(
  detail: RefObject<HTMLElement | null>,
  result: ReviewResult | null,
  pending: boolean,
  compact: boolean,
  loaded: ReadonlyMap<string, LoadedFile>,
) {
  const jumpTarget = useRef<ReviewNote | null>(null);

  useEffect(() => {
    jumpTarget.current = null;
  }, [result?.session]);
  useEffect(() => {
    if (pending || !jumpTarget.current || !result) return;
    const note = jumpTarget.current.note;
    const section = detail.current?.querySelector<HTMLElement>(
      `[data-path="${CSS.escape(note.path)}"]`,
    );
    const node = section?.querySelector<HTMLElement>(
      `.jj-diff-cell[data-side="${note.side}"][data-line="${note.line}"]`,
    );
    // In the compact list the section may still be loading, so keep the jump until its lines render.
    if (!node) {
      section?.scrollIntoView({ block: "start" });
      if (!compact) jumpTarget.current = null;
      return;
    }
    node.scrollIntoView({ block: "center" });
    node.focus({ preventScroll: true });
    jumpTarget.current = null;
  }, [pending, result, loaded, compact]);

  return jumpTarget;
}
