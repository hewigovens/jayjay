import { isImagePath, type ImageSide, type ReviewBundle } from "@jayjay/review-web/types";
import type { PullError, PullFile, PullManifest } from "./PullManifest";
import { reviewPath, type ReviewRef } from "./ReviewRef";

export class PullLoadError extends Error {
  constructor(readonly code: PullError) {
    super(code);
  }
}

export interface FileContent {
  before: string;
  after: string;
  images: Partial<Record<ImageSide, Blob>> | null;
}

// Each file is two requests to raw.githubusercontent.com, which the browser multiplexes over HTTP/2.
const concurrency = 24;
const imageTypes: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  bmp: "image/bmp",
  ico: "image/x-icon",
  svg: "image/svg+xml",
};
const errors = new Set<string>([
  "not-found",
  "rate-limited",
  "unauthorized",
  "too-large",
  "unavailable",
] satisfies PullError[]);

export class PullLoader {
  private contents = new Map<number, Promise<FileContent>>();

  private constructor(
    readonly manifest: PullManifest,
    private fetcher: typeof fetch,
  ) {}

  static async open(
    target: ReviewRef,
    token: string | null,
    signal?: AbortSignal,
    fetcher: typeof fetch = fetch.bind(globalThis),
    fresh = false,
  ) {
    const response = await fetcher(`/api${reviewPath(target)}`, {
      signal,
      cache: fresh ? "no-store" : "default",
      headers: {
        ...(token && { authorization: `Bearer ${token}` }),
        ...(fresh && { "cache-control": "no-cache" }),
      },
    });
    if (!response.ok) {
      const code = await response.json().then(
        (body) => body?.error,
        () => undefined,
      );
      throw new PullLoadError(errors.has(code) ? code : "unavailable");
    }
    return new PullLoader((await response.json()) as PullManifest, fetcher);
  }

  bundle(loaded: Map<number, FileContent>): string {
    const { manifest } = this;
    const bundle: ReviewBundle = {
      schemaVersion: 1,
      repositoryId: `github.com/${manifest.repository}`,
      changeId:
        manifest.commit?.slice(0, 12) ??
        (manifest.range ? `compare/${manifest.range}` : `pull/${manifest.number}`),
      title:
        manifest.number && !manifest.commit
          ? `${manifest.title} #${manifest.number}`
          : manifest.title,
      timestampMillis: manifest.timestampMillis,
      description: manifest.body,
      author: manifest.author?.login ?? "",
      authorEmail: manifest.author
        ? `${manifest.author.id}+${manifest.author.login}@users.noreply.github.com`
        : "",
      changeIdPrefix: 0,
      files: manifest.files.map((file, index) => {
        const content = loaded.get(index);
        return {
          path: file.path,
          ...(file.oldPath && { oldPath: file.oldPath }),
          // Both blob IDs name the diff before the content loads; without the old one, the base commit stands in for it.
          reviewIdentity: `${file.kind}|${file.oldPath ?? ""}|${file.oldSha ?? manifest.baseSha}|${file.sha ?? ""}`,
          kind: file.kind,
          added: file.added,
          removed: file.removed,
          ...(content && { before: content.before, after: content.after }),
        };
      }),
    };
    return JSON.stringify(bundle);
  }

  content(index: number): Promise<FileContent> {
    let pending = this.contents.get(index);
    if (!pending) {
      const file = this.manifest.files[index];
      if (!file) return Promise.reject(new Error("Unknown review file"));
      pending = this.fetchContent(file);
      this.contents.set(index, pending);
      pending.catch(() => this.contents.delete(index));
    }
    return pending;
  }

  async prefetch(loaded: (index: number, content: FileContent) => unknown, signal: AbortSignal) {
    let next = 0;
    const { length } = this.manifest.files;
    await Promise.all(
      Array.from({ length: Math.min(concurrency, length) }, async () => {
        while (next < length && !signal.aborted) {
          const index = next++;
          if (this.contents.has(index)) continue;
          try {
            await loaded(index, await this.content(index));
          } catch {
            /* Asking for the file again retries it. */
          }
        }
      }),
    );
  }

  private async fetchContent(file: PullFile): Promise<FileContent> {
    const { repository, baseSha, headSha } = this.manifest;
    const [before, after] = await Promise.all([
      file.kind === "added" ? null : this.raw(repository, baseSha, file.oldPath ?? file.path),
      file.kind === "removed" ? null : this.raw(repository, headSha, file.path),
    ]);
    const type = isImagePath(file.path)
      ? imageTypes[file.path.slice(file.path.lastIndexOf(".") + 1).toLowerCase()]
      : undefined;
    const images = type
      ? {
          ...(before && { old: new Blob([before], { type }) }),
          ...(after && { new: new Blob([after], { type }) }),
        }
      : null;
    return { before: text(before), after: text(after), images };
  }

  // Null where GitHub has no blob: a submodule, or a path the API listed but the commit lacks.
  private async raw(repository: string, sha: string, path: string) {
    const url = `https://raw.githubusercontent.com/${repository}/${sha}/${path.split("/").map(encodeURIComponent).join("/")}`;
    const response = await this.fetcher(url).catch(() => {
      throw new PullLoadError("unavailable");
    });
    if (response.status === 404) return null;
    if (!response.ok)
      throw new PullLoadError(response.status === 429 ? "rate-limited" : "unavailable");
    return new Uint8Array(await response.arrayBuffer());
  }
}

const decoder = new TextDecoder("utf-8", { fatal: true });

// Binary sides review as empty text; images still show through the backend's `image`.
function text(bytes: Uint8Array | null) {
  if (!bytes) return "";
  try {
    const decoded = decoder.decode(bytes);
    return decoded.includes("\0") ? "" : decoded;
  } catch {
    return "";
  }
}
