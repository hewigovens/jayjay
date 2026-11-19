/** @jsxRuntime automatic */
import type { FileKind } from "../../model/review";

// The desktop app's file-status badge: the kind's glyph on a tinted square.
const glyphs: Record<FileKind, string> = {
  added: "M8 4.5v7M4.5 8h7",
  removed: "M4.5 8h7",
  modified: "M8 3.5v5M5.5 6h5M5.5 11.5h5",
  renamed: "M4.5 8h7M8.5 5l3 3-3 3",
  copied: "M6.5 6.5h5v5h-5zM4.5 9.5v-5h5",
};

export function KindBadge({ kind, label }: { kind: FileKind; label: string }) {
  return (
    <span className={`jj-kind-badge jj-${kind}`} title={label}>
      <svg
        viewBox="0 0 16 16"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d={glyphs[kind]} />
      </svg>
    </span>
  );
}
