import type { DiffOptions, ImageSide, ReviewAction, ReviewResult } from "./review";

export interface ReviewSelection {
  session: number;
  file: number;
  options: DiffOptions;
}

// Backends own source loading, canonical review state, and acknowledgement after durable writes.
export interface ReviewBackend {
  open(signal?: AbortSignal): Promise<ReviewResult>;
  render(selection: ReviewSelection, signal?: AbortSignal): Promise<ReviewResult>;
  mutate(selection: ReviewSelection, action: ReviewAction, signal?: AbortSignal): Promise<ReviewResult>;
  /** One side of an image file, or null when that side has none. Without it, image files show no picture. */
  image?(selection: ReviewSelection, side: ImageSide, signal?: AbortSignal): Promise<Blob | null>;
}
