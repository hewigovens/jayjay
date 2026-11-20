import type { PullFile } from "../../src/github/PullManifest";

const missing = /^0+$/;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function diffFiles(diff: string): PullFile[] {
  const files = new DiffFiles();
  for (const line of diff.split("\n")) files.read(line);
  return files.list;
}

// Reads the files of a diff as it arrives, so a large one is never held whole, and stops once it lists more than `limit`.
export async function streamDiffFiles(diff: ReadableStream<Uint8Array>, limit: number) {
  const files = new DiffFiles();
  const text = new TextDecoder();
  const source = diff.getReader();
  let rest = "";
  for (;;) {
    const { done, value } = await source.read();
    const chunk = text.decode(value, { stream: !done });
    // Only the chunk is searched for a line end, so a line spanning many chunks is not scanned again for each.
    const end = done ? chunk.length : chunk.lastIndexOf("\n");
    if (end < 0) {
      rest += chunk;
      continue;
    }
    for (const line of (rest + chunk.slice(0, end)).split("\n")) files.read(line);
    rest = chunk.slice(end + 1);
    if (done) break;
    if (files.list.length > limit) {
      await source.cancel();
      break;
    }
  }
  return files.list;
}

class DiffFiles {
  readonly list: PullFile[] = [];
  private file: PullFile | undefined;
  private inHunk = false;

  read(line: string) {
    const { file } = this;
    const header = line.startsWith("diff --git ");
    if (file && this.inHunk && !header) {
      if (line.startsWith("+")) file.added++;
      else if (line.startsWith("-")) file.removed++;
      return;
    }
    // A substring keeps the whole chunk it was cut from alive, so what a file keeps is cut from a copy of its line.
    line = decoder.decode(encoder.encode(line));
    if (header) {
      const [oldPath, path] = headerPaths(line.slice("diff --git ".length));
      this.file = {
        path,
        ...(oldPath !== path && { oldPath }),
        kind: "modified",
        added: 0,
        removed: 0,
        sha: null,
      };
      this.list.push(this.file);
      this.inHunk = false;
    } else if (!file) {
      return;
    } else if (line.startsWith("@@")) {
      this.inHunk = true;
    } else if (line.startsWith("Binary files ") || line === "GIT binary patch") {
      file.binary = true;
    } else if (line.startsWith("old mode ")) {
      file.oldMode = line.slice("old mode ".length);
    } else if (line.startsWith("new mode ")) {
      file.newMode = line.slice("new mode ".length);
    } else if (line.startsWith("new file mode")) {
      file.kind = "added";
    } else if (line.startsWith("deleted file mode")) {
      file.kind = "removed";
    } else if (line.startsWith("rename from ") || line.startsWith("copy from ")) {
      file.kind = line.startsWith("rename") ? "renamed" : "copied";
      file.oldPath = unquote(line.slice(line.indexOf(" from ") + 6));
    } else if (line.startsWith("rename to ") || line.startsWith("copy to ")) {
      file.path = unquote(line.slice(line.indexOf(" to ") + 4));
    } else if (line.startsWith("index ")) {
      const [before, after] = line.slice("index ".length).split(" ")[0].split("..");
      if (before && !missing.test(before)) file.oldSha = before;
      if (after && !missing.test(after)) file.sha = after;
    }
  }
}

// Unquoted paths split in half, which is exact unless the file was renamed, when `rename` lines name it.
function headerPaths(header: string): [string, string] {
  if (header.startsWith('"')) {
    const [old, end] = quoted(header);
    const rest = header.slice(end + 2);
    return [old.slice(2), (rest.startsWith('"') ? quoted(rest)[0] : rest).slice(2)];
  }
  if (header.endsWith('"')) {
    const start = header.lastIndexOf(' "');
    return [header.slice(2, start), quoted(header.slice(start + 1))[0].slice(2)];
  }
  const half = (header.length - 1) / 2;
  return [header.slice(2, half), header.slice(half + 3)];
}

function unquote(path: string) {
  return path.startsWith('"') ? quoted(path)[0] : path;
}

// Returns the leading C-quoted string decoded as UTF-8 and the index of its closing quote.
function quoted(input: string): [string, number] {
  const bytes: number[] = [];
  let index = 1;
  for (; index < input.length && input[index] !== '"'; index++) {
    const ch = input[index];
    if (ch !== "\\") {
      bytes.push(...encoder.encode(ch));
      continue;
    }
    const next = input[++index];
    if (/[0-7]/.test(next)) {
      bytes.push(parseInt(input.slice(index, index + 3), 8));
      index += 2;
    } else {
      bytes.push(
        ({ n: 10, t: 9, r: 13, a: 7, b: 8, f: 12, v: 11 } as Record<string, number>)[next] ??
          next.charCodeAt(0),
      );
    }
  }
  return [decoder.decode(new Uint8Array(bytes)), index];
}
