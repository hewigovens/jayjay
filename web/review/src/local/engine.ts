import init, * as bindings from "../wasm/engine/jj_diff";
import { cachedWasm, engineUrl } from "./assets";

export type Engine = typeof bindings;

export const engine: Promise<Engine> = (async () => {
  if (!engineUrl) throw new Error("The review engine is missing; run scripts/build-diff-web.sh");
  await init({ module_or_path: await cachedWasm(engineUrl) });
  bindings.ui_labels(navigator.languages.join(","));
  return bindings;
})();

void engine.catch(() => {});

let instance = 0;

// Increments whenever the engine is replaced, so work that awaited can tell its objects and linked grammars are gone.
export const engineInstance = () => instance;

// V8 reports Wasm stack exhaustion as a RangeError, which leaves the shadow stack pointer as unrecoverable as a trap does.
export const isTrap = (error: unknown) =>
  error instanceof WebAssembly.RuntimeError || error instanceof RangeError;

// A trap can leave the instance's memory, stack pointer, or allocator inconsistent, so it is replaced rather than reused; objects from the old instance stop working.
export function resetEngine() {
  instance++;
  bindings.__wbg_reset_state();
  bindings.ui_labels(navigator.languages.join(","));
}
