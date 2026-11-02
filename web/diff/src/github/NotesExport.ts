import type { Overview } from "@jayjay/review-web/types";
import { parsePullRequest } from "./PullRequestRef";

export interface NotesExport {
  repository: string;
  number: number;
  title: string;
  headSha: string;
  notes: {
    path: string;
    side: "old" | "new";
    line: number;
    status: string;
    excerpt: string;
    body: string;
  }[];
  files: { path: string; review: string }[];
}

export const maxExportBytes = 1024 * 1024;
export const exportDays = 7;

export function notesExport(
  overview: Overview,
  repository: string,
  number: number,
  headSha: string,
): NotesExport {
  return {
    repository,
    number,
    headSha,
    title: overview.title,
    notes: overview.notes
      .filter((item) => item.status !== "resolved")
      .map(({ note, status }) => ({
        path: note.path,
        side: note.side,
        line: note.line,
        status,
        excerpt: note.anchor_excerpt,
        body: note.body,
      })),
    files: overview.files.map((file) => ({ path: file.path, review: file.review })),
  };
}

// The Worker stores nothing that fails this check.
export function isNotesExport(value: unknown): value is NotesExport {
  const body = value as NotesExport;
  return (
    typeof body?.repository === "string" &&
    parsePullRequest(`${body.repository}#${body.number}`) !== null &&
    typeof body.title === "string" &&
    typeof body.headSha === "string" &&
    Array.isArray(body.notes) &&
    Array.isArray(body.files) &&
    body.notes.every(
      (note) =>
        typeof note?.path === "string" &&
        typeof note.body === "string" &&
        Number.isSafeInteger(note.line),
    ) &&
    body.files.every((file) => typeof file?.path === "string" && typeof file.review === "string")
  );
}

// Stays English like other tool output for agents.
export function notesMarkdown(body: NotesExport): string {
  const lines = [
    `# Review notes for ${body.repository}#${body.number}: ${body.title}`,
    "",
    `Pull request: https://github.com/${body.repository}/pull/${body.number} (reviewed at head ${body.headSha})`,
    "",
    "Each note is anchored to a line of the pull request's diff: `new` lines are in the head, `old` lines in the merge base. A `stale` note's line changed after it was written.",
  ];
  const byPath = new Map<string, NotesExport["notes"]>();
  for (const note of body.notes) byPath.set(note.path, [...(byPath.get(note.path) ?? []), note]);
  lines.push("", `## Open notes (${body.notes.length})`);
  for (const [path, notes] of byPath) {
    lines.push("", `### ${path}`);
    for (const note of notes) {
      lines.push(
        "",
        `- ${note.side} line ${note.line}${note.status === "current" ? "" : ` (${note.status})`}: \`${note.excerpt.trim()}\``,
      );
      lines.push(...note.body.split("\n").map((line) => `  > ${line}`));
    }
  }
  // Only files someone has looked at are listed; the rest of the pull request is the unreviewed remainder.
  const touched = body.files.filter((file) => file.review !== "unreviewed");
  const reviewed = touched.filter((file) => file.review === "reviewed").length;
  lines.push("", `## Review state: ${reviewed} of ${body.files.length} files reviewed`);
  if (touched.length)
    lines.push("", ...touched.map((file) => `- ${file.path}: ${file.review.replaceAll("_", " ")}`));
  return lines.join("\n") + "\n";
}
