import { TransientError } from "./TransientError";

const urls = import.meta.glob<string>(["../wasm/engine/*.wasm", "../wasm/grammars/*.wasm"], {
  query: "?url",
  import: "default",
  eager: true,
});
const queries = import.meta.glob<string>("../wasm/queries/*.scm", {
  query: "?raw",
  import: "default",
});
const manifests = import.meta.glob<unknown>("../wasm/grammars.json", {
  import: "default",
  eager: true,
});

export const engineUrl = urls["../wasm/engine/jj_diff_bg.wasm"];

export function grammarUrl(grammar: string): string | undefined {
  return urls[`../wasm/grammars/${grammar}.wasm`];
}

export interface LanguageGrammar {
  grammar: string;
  requires: string[];
}

export interface GrammarManifest {
  imports: string[];
  languages: Map<string, LanguageGrammar>;
}

const strings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");

// A missing or malformed table leaves every language without syntax, which the review reports.
export function grammarManifest(): GrammarManifest {
  const manifest = manifests["../wasm/grammars.json"];
  const languages = new Map<string, LanguageGrammar>();
  if (typeof manifest !== "object" || manifest === null) return { imports: [], languages };
  const { imports, languages: entries } = manifest as Record<string, unknown>;
  if (typeof entries === "object" && entries !== null)
    for (const [language, entry] of Object.entries(entries)) {
      const { grammar, requires } = (entry ?? {}) as Record<string, unknown>;
      if (typeof grammar === "string" && strings(requires))
        languages.set(language, { grammar, requires });
    }
  return { imports: strings(imports) ? imports : [], languages };
}

const isWasm = (bytes: ArrayBuffer) =>
  String.fromCharCode(...new Uint8Array(bytes.slice(0, 4))) === "\0asm";

export function grammarQuery(language: string): Promise<string> {
  const load = queries[`../wasm/queries/${language}.scm`];
  if (!load) return Promise.reject(new Error(`No highlight query for ${language}`));
  return load().catch((error: unknown) => {
    throw new TransientError(`Highlight query for ${language} did not load: ${String(error)}`);
  });
}

// Fetches a WASM asset, through the Cache API in production; a response that is not WebAssembly, such as a host's fallback page, is neither used nor cached.
export async function cachedWasm(url: string): Promise<ArrayBuffer> {
  let cache: Cache | undefined;
  // Dev asset URLs are not content-hashed, so a cached file would outlive a rebuild.
  if (import.meta.env.PROD)
    try {
      cache = await caches.open("jayjay-syntax-v1");
      const cached = await (await cache.match(url, { ignoreVary: true }))?.arrayBuffer();
      if (cached && isWasm(cached)) return cached;
    } catch {
      /* Cache availability does not prevent online diffing. */
    }
  let response: Response;
  try {
    response = await fetch(url);
  } catch (error) {
    throw new TransientError(`WASM download failed for ${url}: ${String(error)}`);
  }
  if (!response.ok) throw new TransientError(`WASM download failed: ${response.status} for ${url}`);
  const bytes = await response.arrayBuffer();
  if (!isWasm(bytes)) throw new TransientError(`${url} did not return WebAssembly`);
  if (cache) {
    try {
      await cache.put(
        url,
        new Response(bytes, { headers: { "content-type": "application/wasm" } }),
      );
      const current = new Set(Object.values(urls).map((asset) => new Request(asset).url));
      for (const stale of await cache.keys())
        if (!current.has(stale.url)) await cache.delete(stale);
    } catch {
      /* Cache failures leave the current diff usable. */
    }
  }
  return bytes;
}
