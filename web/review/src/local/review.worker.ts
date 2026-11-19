/// <reference lib="webworker" />
import type { ReviewSession } from "../wasm/engine/jj_diff";
import { BrowserReviewStorage } from "./BrowserReviewStorage";
import { engine, engineInstance, isTrap, resetEngine, type Engine } from "./engine";
import { GrammarRegistry } from "./GrammarRegistry";
import type { WorkerRequest, WorkerResponse } from "./protocol";
import {
  defaultOptions,
  type ReviewBundle,
  type DiffOptions,
  type ReviewResult,
} from "../model/review";

let active:
  | {
      bundle: ReviewBundle;
      raw: string;
      loaded: Set<number>;
      session: ReviewSession;
      instance: number;
      version: number;
    }
  | undefined;
let version = 0;
let storage: BrowserReviewStorage;
let bindings: Engine | undefined;
let grammars: GrammarRegistry | undefined;

// A trap can leave the engine inconsistent, so the worker replaces the instance and rebuilds the open session, under the same version, from its bundle and the files loaded since.
function restart() {
  if (!bindings) return;
  resetEngine();
  grammars?.reset();
  if (!active) return;
  const { bundle, raw, loaded } = active;
  try {
    const session = new bindings.ReviewSession(raw);
    for (const file of loaded)
      session.load_file(file, bundle.files[file].before ?? "", bundle.files[file].after ?? "");
    Object.assign(active, { session, instance: engineInstance() });
  } catch {
    active = undefined;
  }
}

// Grammar code runs while a grammar links, so a trap there restarts the engine without that language; a restart while this awaits drops linked grammars, so it checks again.
async function syntax(registry: GrammarRegistry, language: string): Promise<boolean> {
  for (;;) {
    const instance = engineInstance();
    let available: boolean;
    try {
      available = await registry.ensure(language);
    } catch (error) {
      if (!isTrap(error)) throw error;
      restart();
      registry.disable(language);
      available = false;
    }
    if (instance === engineInstance()) return available;
  }
}

function current(session: number) {
  if (!active || active.version !== session)
    throw new Error("Review changed; reload before applying this action");
  return active;
}

// An open may leave its first file unrendered until the host loads it; every other render needs the file.
async function render(
  session: number,
  file: number,
  options: DiffOptions,
  opening = false,
): Promise<ReviewResult> {
  const selected = current(session);
  const source = selected.bundle.files[file];
  const wasm = bindings;
  if (!wasm) throw new Error("The review engine is not loaded");
  grammars ??= new GrammarRegistry(wasm);
  const language = wasm.language_for(source?.path ?? "", source?.after || source?.before || "");
  const state = await storage.readReview(selected.bundle.repositoryId);
  current(session);
  const backup = await storage.readStored(`review-backup:${selected.bundle.repositoryId}`);
  current(session);
  // Nothing below awaits, so the diff renders in the engine instance its grammar check ran in.
  let syntaxUnavailable = !(await syntax(grammars, language));
  current(session);
  const canUndoImport = Boolean(backup && JSON.parse(backup).imported === state);
  const start = performance.now();
  const overview = JSON.parse(selected.session.overview(state));
  const { before, after } = source ?? {};
  const loaded = before !== undefined && after !== undefined;
  if (source && !loaded && !opening) throw new Error("Review file is not loaded");
  const renderDiff = () =>
    source && loaded
      ? JSON.parse(
          wasm.render_diff(
            source.path,
            before,
            after,
            options.ignoreWhitespace,
            options.showAll,
            options.split,
          ),
        )
      : null;
  let diff;
  try {
    diff = renderDiff();
  } catch (error) {
    if (!isTrap(error)) throw error;
    restart();
    const disabledHere = grammars.disable(language);
    try {
      diff = renderDiff();
      syntaxUnavailable = true;
    } catch (plain) {
      // Without the grammar it trapped again, so the grammar was not the cause.
      if (disabledHere) grammars.enable(language);
      throw plain;
    }
  }
  const review =
    source && loaded
      ? JSON.parse(selected.session.file_review(file, state, options.ignoreWhitespace))
      : { groupStates: [] };
  return {
    overview,
    diff,
    ...review,
    file,
    options,
    session,
    syntaxUnavailable,
    canUndoImport,
    elapsed: performance.now() - start,
  };
}

async function handle(request: WorkerRequest): Promise<unknown> {
  bindings ??= await engine;
  switch (request.kind) {
    case "init": {
      storage = new BrowserReviewStorage(request.databaseName);
      return;
    }
    case "open": {
      const raw = request.bundle ?? (await storage.readStored("active-bundle"));
      if (raw === undefined) return null;
      let session = new bindings.ReviewSession(raw);
      const instance = engineInstance();
      const bundle: ReviewBundle = JSON.parse(raw);
      await storage.writeStored("active-bundle", raw);
      if (instance !== engineInstance()) session = new bindings.ReviewSession(raw);
      // wasm-bindgen's free does not check the instance, so a session from a replaced engine is dropped, not freed.
      if (active?.instance === engineInstance()) active.session.free();
      active = {
        bundle,
        raw,
        loaded: new Set(),
        session,
        instance: engineInstance(),
        version: ++version,
      };
      return render(version, 0, defaultOptions, true);
    }
    case "load": {
      const selected = current(request.session);
      selected.session.load_file(request.file, request.before, request.after);
      selected.loaded.add(request.file);
      Object.assign(selected.bundle.files[request.file], {
        before: request.before,
        after: request.after,
      });
      return;
    }
    case "render":
      return render(request.session, request.file, request.options);
    case "mutate": {
      const selected = current(request.session);
      await storage.updateReview(selected.bundle.repositoryId, (state) => {
        current(request.session);
        return selected.session.apply(JSON.stringify(request.action), state);
      });
      return render(request.session, request.file, request.options);
    }
    case "export": {
      const selected = current(request.session);
      return storage.readReview(selected.bundle.repositoryId);
    }
    case "restoreState": {
      const selected = current(request.session);
      const raw = await storage.readStored(`review-backup:${selected.bundle.repositoryId}`);
      if (!raw) throw new Error("No review import to undo");
      const backup: { previous: string; imported: string } = JSON.parse(raw);
      await storage.updateReview(selected.bundle.repositoryId, (state) => {
        current(request.session);
        if (state !== backup.imported)
          throw new Error(
            "Review state changed after import; preserve those edits before restoring",
          );
        return backup.previous;
      });
      return render(request.session, request.file, request.options);
    }
    case "importState": {
      const selected = current(request.session);
      selected.session.overview(request.state);
      await storage.updateReview(
        selected.bundle.repositoryId,
        () => {
          current(request.session);
          return request.state;
        },
        true,
      );
      return render(request.session, request.file, request.options);
    }
  }
}

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

self.onmessage = async ({ data }: MessageEvent<WorkerRequest & { id: number }>) => {
  let response: WorkerResponse = { id: data.id, error: "The review worker failed" };
  try {
    response = { id: data.id, result: await handle(data) };
  } catch (error) {
    response = { id: data.id, error: message(error) };
    if (isTrap(error))
      try {
        restart();
      } catch (failure) {
        response = {
          id: data.id,
          error: `${message(error)}; the engine did not restart: ${message(failure)}`,
        };
      }
  } finally {
    self.postMessage(response);
  }
};
