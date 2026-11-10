import {
  exportDays,
  isNotesExport,
  maxExportBytes,
  notesMarkdown,
  type NotesExport,
} from "../src/github/NotesExport";

interface ExportStore {
  put(key: string, value: string, options: { expirationTtl: number }): Promise<void>;
  get(key: string): Promise<string | null>;
}

interface ExportLimiter {
  limit(options: { key: string }): Promise<{ success: boolean }>;
}

export class Exports {
  constructor(
    private store: ExportStore,
    private limiter: ExportLimiter,
  ) {}

  async create(request: Request): Promise<Response> {
    const client = request.headers.get("cf-connecting-ip") ?? "";
    if (!(await this.limiter.limit({ key: client })).success)
      return Response.json({ error: "rate-limited" }, { status: 429 });
    const text = await Exports.text(request);
    if (text === null) return Response.json({ error: "too-large" }, { status: 413 });
    let body: unknown;
    try {
      body = JSON.parse(text);
    } catch {
      body = null;
    }
    if (!isNotesExport(body)) return Response.json({ error: "invalid" }, { status: 400 });
    const id = crypto.randomUUID().replaceAll("-", "");
    await this.store.put(id, JSON.stringify(body), { expirationTtl: exportDays * 24 * 60 * 60 });
    return Response.json({ url: new URL(`/api/exports/${id}`, request.url).href }, { status: 201 });
  }

  private static async text(request: Request): Promise<string | null> {
    if (Number(request.headers.get("content-length")) > maxExportBytes) return null;
    if (!request.body) return "";
    const reader = request.body.getReader();
    const decoder = new TextDecoder();
    let size = 0;
    let text = "";
    for (;;) {
      const { done, value } = await reader.read();
      if (done) return text + decoder.decode();
      size += value.byteLength;
      if (size > maxExportBytes) {
        await reader.cancel();
        return null;
      }
      text += decoder.decode(value, { stream: true });
    }
  }

  async read(name: string): Promise<Response> {
    const [, id, json] = /^([0-9a-f]{32})(\.json)?$/.exec(name) ?? [];
    const stored = id ? await this.store.get(id) : null;
    if (!stored) return new Response("Not found or expired\n", { status: 404 });
    const headers = { "cache-control": "no-store", "x-robots-tag": "noindex" };
    return json
      ? new Response(stored, { headers: { ...headers, "content-type": "application/json" } })
      : new Response(notesMarkdown(JSON.parse(stored) as NotesExport), {
          headers: { ...headers, "content-type": "text/markdown; charset=utf-8" },
        });
  }
}
