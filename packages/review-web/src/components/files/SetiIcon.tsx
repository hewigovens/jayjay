/** @jsxRuntime automatic */
import type { FileGlyph } from "../../model/review";
import { FileIcon } from "../icons";

const SIZE = 18;

/** Seti glyphs sit unevenly in the em, so the metrics centre each one's ink in a fixed box. */
export function SetiIcon({ icon }: { icon?: FileGlyph }) {
  if (!icon) return <FileIcon />;
  return <span className="jj-icon jj-seti" style={{ fontSize: SIZE * icon.scale }} aria-hidden="true">
    <span style={{ width: `${icon.inkWidth}em`, textIndent: `${-icon.inkLeft}em`, top: `${icon.inkCenter - 0.5}em` }}>{icon.glyph}</span>
  </span>;
}
