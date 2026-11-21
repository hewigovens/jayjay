import type { AvatarSource } from "./review";

/** One commit in a host's commit list. */
export interface CommitEntry {
  /** What `onSelect` receives. */
  id: string;
  /** The id as shown, such as a short hash or `@`. */
  ref: string;
  /** How many leading characters of `ref` tell it from the others listed, as jj highlights them. */
  refPrefix: number;
  title: string;
  author: string;
  /** Absent or null shows the monogram. */
  avatar?: AvatarSource | null;
  /** Milliseconds since the epoch; 0 leaves the time out. */
  timestampMillis: number;
  /** Shown after the author and time, such as a branch or that the commit is empty. */
  note?: string;
}
