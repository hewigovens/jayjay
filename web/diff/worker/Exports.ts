import {
  exportDays,
  parseNotesExport,
  maxExportBytes,
  notesMarkdown,
} from "../src/github/NotesExport";

interface ExportStore {
  put(key: string, value: string, options: { expirationTtl: number }): Promise<void>;
  get(key: string): Promise<string | null>;
}

export class Exports {
  constructor(private store: ExportStore) {}

  async create(request: Request): Promise<Response> {
    const text = await request.text();
    if (new TextEncoder().encode(text).length > maxExportBytes)
      return Response.json({ error: "too-large" }, { status: 413 });
    let body: unknown;
    try {
      body = JSON.parse(text);
    } catch {
      body = null;
    }
    const normalized = parseNotesExport(body);
    if (!normalized) return Response.json({ error: "invalid" }, { status: 400 });
    const id = crypto.randomUUID().replaceAll("-", "");
    await this.store.put(id, JSON.stringify(normalized), {
      expirationTtl: exportDays * 24 * 60 * 60,
    });
    return Response.json({ url: new URL(`/api/exports/${id}`, request.url).href }, { status: 201 });
  }

  async read(name: string): Promise<Response> {
    const [, id, json] = /^([0-9a-f]{32})(\.json)?$/.exec(name) ?? [];
    const stored = id ? await this.store.get(id) : null;
    if (!stored) return new Response("Not found or expired\n", { status: 404 });
    const body = parseNotesExport(JSON.parse(stored));
    if (!body) return new Response("Invalid export\n", { status: 500 });
    const headers = { "cache-control": "no-store", "x-robots-tag": "noindex" };
    return json
      ? new Response(stored, { headers: { ...headers, "content-type": "application/json" } })
      : new Response(notesMarkdown(body), {
          headers: { ...headers, "content-type": "text/markdown; charset=utf-8" },
        });
  }
}
