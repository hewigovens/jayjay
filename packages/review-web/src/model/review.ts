import type { DiffView } from "./diff";

export interface ReviewFile {
  path: string;
  oldPath?: string;
  reviewIdentity: string;
  before: string;
  after: string;
}

export interface ReviewBundle {
  schemaVersion: 1;
  repositoryId: string;
  changeId: string;
  title: string;
  description?: string;
  author?: string;
  authorEmail?: string;
  files: ReviewFile[];
}

export interface AvatarSource { kind: "url" | "gitHubBot" | "gitLabUser"; value: string }

export type ReviewState = "reviewed" | "unreviewed" | "partial" | "changed_since_review";
export type FileKind = "added" | "modified" | "removed" | "renamed" | "copied";
export type ImageSide = "old" | "new";

const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg"]);

/** Whether a path names an image the browser can show, by an extension after a name (`.png` alone is not one). */
export function isImagePath(path: string): boolean {
  const dot = path.lastIndexOf(".");
  return dot > path.lastIndexOf("/") + 1 && IMAGE_EXTENSIONS.has(path.slice(dot + 1).toLowerCase());
}
/** From `jayjay_primitives::seti`; metrics are fractions of the em. */
export interface FileGlyph { glyph: string; scale: number; inkLeft: number; inkWidth: number; inkCenter: number }

export interface FileSummary {
  path: string;
  oldPath: string | null;
  /** Missing from backends that predate it; shown as modified. */
  kind?: FileKind;
  /** Missing from backends that predate it; shown as a plain page icon. */
  icon?: FileGlyph;
  added: number;
  removed: number;
  review: ReviewState;
  noteCount: number;
}

export interface ReviewNote {
  note: { id: string; path: string; side: "old" | "new"; line: number; body: string; resolved: boolean; anchor_excerpt: string; created_at_ms: number; ignore_whitespace: boolean };
  status: "current" | "stale" | "orphaned" | "resolved";
  group_index?: number;
}

export interface Overview {
  repositoryId: string;
  changeId: string;
  title: string;
  description: string;
  author: string;
  /** From `jayjay_primitives::avatar::AvatarSource`; absent or null shows the monogram. */
  authorAvatar?: AvatarSource | null;
  /** How many leading characters of the change id are its shortest unique prefix, as jj highlights it. */
  changeIdPrefix?: number;
  /** Author timestamp in milliseconds since the epoch. */
  timestampMillis?: number;
  files: FileSummary[];
  /** Count labels formatted by the catalog; `filtered` is indexed by the number of files shown. */
  counts: { files: string; reviewed: string; filtered: string[] };
  tree: { name: string; path: string; depth: number; file: number | null }[];
  notes: ReviewNote[];
}

export interface DiffOptions { split: boolean; ignoreWhitespace: boolean; showAll: boolean }
export interface ReviewResult {
  overview: Overview;
  file: number;
  session: number;
  diff: DiffView | null;
  groupStates: ReviewState[];
  options: DiffOptions;
  elapsed: number;
  pack: string;
  syntaxUnavailable: boolean;
  canUndoImport?: boolean;
}

export type ReviewAction =
  | { kind: "setFile"; file: number; reviewed: boolean }
  | { kind: "setGroup"; file: number; group: number; reviewed: boolean; ignoreWhitespace: boolean }
  | { kind: "addNote"; file: number; side: "old" | "new"; line: number; body: string; ignoreWhitespace: boolean }
  | { kind: "resolveNote"; id: string };

export const defaultOptions: DiffOptions = { split: false, ignoreWhitespace: false, showAll: false };
