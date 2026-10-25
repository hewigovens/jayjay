import type * as Engine from "../wasm/base/jj_diff_base";

type EngineModule = typeof Engine;
const imports = import.meta.glob<EngineModule>("../wasm/*/jj_diff_*.js");
const wasmUrls = import.meta.glob<string>("../wasm/*/*_bg.wasm", { query: "?url", import: "default", eager: true });
const modules = new Map<string, Promise<EngineModule>>();

export function engine(pack = "base"): Promise<EngineModule> {
  let pending = modules.get(pack);
  if (!pending) {
    const load = imports[`../wasm/${pack}/jj_diff_${pack}.js`];
    if (!load) return Promise.reject(new Error(`Unknown language pack: ${pack}`));
    pending = Promise.all([load(), wasmResponse(wasmUrls[`../wasm/${pack}/jj_diff_${pack}_bg.wasm`])])
      .then(async ([module, response]) => {
        await module.default({ module_or_path: response });
        module.ui_labels(navigator.languages.join(","));
        return module;
      });
    modules.set(pack, pending);
    void pending.catch(() => modules.delete(pack));
  }
  return pending;
}

export const baseEngine = engine();

void baseEngine.catch(() => {});

async function wasmResponse(url: string): Promise<Response> {
  let cache: Cache | undefined;
  // Dev asset URLs are not content-hashed, so a cached pack would outlive a rebuilt wasm-bindgen glue module.
  if (import.meta.env.PROD) try {
    cache = await caches.open("jayjay-syntax-v1");
    const response = await cache.match(url, { ignoreVary: true });
    if (response) return response;
  } catch { /* Cache availability does not prevent online diffing. */ }
  const response = await fetch(url);
  if (!response.ok) throw new Error(`WASM download failed: ${response.status}`);
  if (cache) { try { await cache.put(url, response.clone()); } catch { /* Quota failures leave the current diff usable. */ } }
  return response;
}

export async function engineForPath(path: string) {
  const base = await baseEngine;
  const pack = base.language_pack(path);
  try { return { module: await engine(pack), pack, syntaxUnavailable: false }; }
  catch (error) {
    if (pack === "base") throw error;
    return { module: base, pack: "base", syntaxUnavailable: true };
  }
}
