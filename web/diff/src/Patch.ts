import type { ReviewBundle } from "@jayjay/review-web/types";
import { diffFiles } from "../worker/github/GitHubDiff";
import { maxFiles } from "./github/PullManifest";

export interface PatchContent {
  before: string;
  after: string;
}

export interface Patch {
  title: string;
  /** The files' identities and counts without their text, which `contents` holds in the same order. */
  bundle: string;
  contents: PatchContent[];
}

export class PatchError extends Error {
  constructor(readonly code: "invalid" | "series" | "large") {
    super(code);
  }
}

const hunkHeader = /^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/;
// Stands in for a line the patch leaves out; unlike a blank line, it can't be mistaken for one the patch changes.
const omitted = "⋯";
const maxOmitted = 4_000_000;

interface Budget {
  omitted: number;
}

class Side {
  private lines: string[] = [];
  unterminated = false;

  constructor(private budget: Budget) {}

  // Omitted lines between hunks keep the line numbers of what follows.
  startHunk(start: number, count: number) {
    const gap = (count ? start - 1 : start) - this.lines.length;
    if (gap < 0) throw new PatchError("invalid");
    this.budget.omitted -= gap;
    if (this.budget.omitted < 0) throw new PatchError("large");
    for (let line = 0; line < gap; line++) this.lines.push(omitted);
  }

  push(line: string) {
    this.lines.push(line);
  }

  get text() {
    return this.lines.join("\n") + (this.lines.length && !this.unterminated ? "\n" : "");
  }
}

function sides(section: string, budget: Budget) {
  const before = new Side(budget);
  const after = new Side(budget);
  let added = 0;
  let removed = 0;
  let hunks = 0;
  const lines = section.split("\n");
  for (let index = 0; index < lines.length; index++) {
    const header = hunkHeader.exec(lines[index]);
    if (!header) {
      if (lines[index].startsWith("@@ ")) throw new PatchError("invalid");
      continue;
    }
    hunks++;
    let oldLeft = Number(header[2] ?? 1);
    let newLeft = Number(header[4] ?? 1);
    before.startHunk(Number(header[1]), oldLeft);
    after.startHunk(Number(header[3]), newLeft);
    let last = "";
    for (;;) {
      const line = lines[index + 1];
      if (line?.startsWith("\\")) {
        if (last !== "+") before.unterminated = true;
        if (last !== "-") after.unterminated = true;
      } else if (oldLeft === 0 && newLeft === 0) {
        break;
      } else {
        // Mail clients strip the space from an empty context line.
        const mark = line === "" ? " " : line?.[0];
        if (mark !== " " && mark !== "+" && mark !== "-") throw new PatchError("invalid");
        if (mark !== "+") {
          before.push(line.slice(1));
          oldLeft--;
        }
        if (mark !== "-") {
          after.push(line.slice(1));
          newLeft--;
        }
        if (oldLeft < 0 || newLeft < 0) throw new PatchError("invalid");
        if (mark === "+") added++;
        else if (mark === "-") removed++;
        last = mark;
      }
      index++;
    }
  }
  // Git names a file's two sides only ahead of its hunks, so a section that stops there was cut short.
  if (!hunks && /^\+\+\+ /m.test(section)) throw new PatchError("invalid");
  return { content: { before: before.text, after: after.text }, added, removed };
}

async function digest(text: string) {
  const hash = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return Array.from(new Uint8Array(hash, 0, 8), (byte) => byte.toString(16).padStart(2, "0")).join(
    "",
  );
}

/** Reads `git diff` or `git format-patch` output; `name` identifies the review, so a newer export of the same file keeps its marks. */
export async function parsePatch(text: string, title: string, name?: string): Promise<Patch> {
  // Only a patch saved with CRLF endings ends its headers that way; a carriage return anywhere else belongs to a file's line.
  const diff = /^diff --git [^\n]*\r\n/m.test(text) ? text.replaceAll("\r\n", "\n") : text;
  const start = diff.search(/^diff --git /m);
  if (start < 0) throw new PatchError("invalid");
  const preamble = diff.slice(0, start);
  const sections = diff.slice(start).split(/^(?=diff --git )/m);
  if (sections.length > maxFiles) throw new PatchError("large");
  const budget = { omitted: maxOmitted };
  const paths = new Set<string>();
  const contents: PatchContent[] = [];
  const files = await Promise.all(
    sections.map(async (section) => {
      const [file] = diffFiles(section);
      if (paths.has(file.path)) throw new PatchError("series");
      paths.add(file.path);
      const { content, added, removed } = sides(section, budget);
      contents.push(content);
      return {
        path: file.path,
        ...(file.oldPath && { oldPath: file.oldPath }),
        reviewIdentity: `${file.kind}|${file.oldPath ?? ""}|${await digest(section)}`,
        kind: file.kind,
        added,
        removed,
        ...(file.binary && { binary: true }),
        ...(file.oldMode && { oldMode: file.oldMode, newMode: file.newMode }),
      };
    }),
  );
  const subject = /^Subject: (?:\[PATCH[^\]]*\] )?(.+)$/m.exec(preamble)?.[1];
  const author = /^From: (.+?) <([^>]+)>$/m.exec(preamble);
  const date = Date.parse(/^Date: (.+)$/m.exec(preamble)?.[1] ?? "");
  const bundle: ReviewBundle = {
    schemaVersion: 1,
    repositoryId: "patch",
    changeId: name?.replaceAll("|", "_") || (await digest(text)),
    changeIdPrefix: 0,
    title: subject ?? title,
    ...(!Number.isNaN(date) && { timestampMillis: date }),
    author: author?.[1] ?? "",
    authorEmail: author?.[2] ?? "",
    files,
  };
  return { title: bundle.title, bundle: JSON.stringify(bundle), contents };
}
