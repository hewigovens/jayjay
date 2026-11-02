import type { FileSummary, ReviewNote, ReviewResult } from "../../model/review";

export const anchorKey = (path: string, side: string, line: number) => `${path}|${side}:${line}`;

// Current notes sit under their line and resolved ones behind a gutter marker; the banner takes stale and orphaned notes, and current ones whose line is folded away.
export function placeNotes(
  diffs: { file: number; data: ReviewResult }[],
  files: FileSummary[],
  notes: ReviewNote[],
  folded: (entry: FileSummary) => boolean,
) {
  const placed = new Map<string, ReviewNote[]>();
  const rendered = new Map<string, string>();
  for (const { file: index, data } of diffs) {
    const filePath = data.overview.files[index]?.path;
    const entry = files.find((candidate) => candidate.path === filePath);
    if (entry && folded(entry)) continue;
    for (const row of data.diff?.rows ?? [])
      for (const cell of row.cells)
        if (cell.anchor && filePath)
          rendered.set(
            anchorKey(filePath, cell.anchor.side, cell.anchor.line),
            cell.spans.map((span) => span.text).join(""),
          );
  }
  const renderedPaths = new Set(
    diffs.map(({ file: index, data }) => data.overview.files[index]?.path),
  );
  // A folded file never loads in the compact list, and the banner is the only place its notes can show.
  for (const entry of files) if (folded(entry)) renderedPaths.add(entry.path);
  const unplaced: ReviewNote[] = [];
  for (const item of notes) {
    const key = anchorKey(item.note.path, item.note.side, item.note.line);
    if (item.status === "stale" || item.status === "orphaned") unplaced.push(item);
    else if (!renderedPaths.has(item.note.path)) continue;
    else if (rendered.has(key)) placed.set(key, [...(placed.get(key) ?? []), item]);
    else if (item.status === "current") unplaced.push(item);
  }
  return { placed, rendered, unplaced };
}
