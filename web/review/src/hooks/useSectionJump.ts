import { useEffect, useLayoutEffect, useRef, type RefObject } from "react";
import type { LoadedFile } from "./useLoadedSections";

const takeovers = ["wheel", "touchstart", "pointerdown", "keydown"] as const;

// Lazy sections above the file grow as they render, so the jump holds the file at the top until the reader takes over.
export function useSectionJump(
  detail: RefObject<HTMLElement | null>,
  loaded: ReadonlyMap<string, LoadedFile>,
) {
  const pinned = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const release = () => {
      pinned.current = null;
    };
    for (const type of takeovers) addEventListener(type, release, { capture: true, passive: true });
    return () => {
      for (const type of takeovers) removeEventListener(type, release, { capture: true });
    };
  }, []);
  useLayoutEffect(() => {
    if (pinned.current?.isConnected) pinned.current.scrollIntoView({ block: "start" });
  }, [loaded]);

  return (file: number) => {
    const section = detail.current?.querySelector<HTMLElement>(
      `.jj-file-section[data-file="${file}"]`,
    );
    if (!section) return;
    section.focus({ preventScroll: true });
    section.scrollIntoView({ block: "start" });
    pinned.current = section;
  };
}
