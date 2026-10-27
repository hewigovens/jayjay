/// <reference lib="webworker" />
import { baseEngine, engineForPath } from "./engine";
import { BrowserReviewStorage } from "./BrowserReviewStorage";
import type { WorkerRequest, WorkerResponse } from "./protocol";
import { defaultOptions, type ReviewBundle, type DiffOptions, type ReviewResult } from "../model/review";

let active: { bundle: ReviewBundle; session: InstanceType<Awaited<typeof baseEngine>["ReviewSession"]>; version: number } | undefined;
let version = 0;
let storage: BrowserReviewStorage;

function current(session: number) {
  if (!active || active.version !== session) throw new Error("Review changed; reload before applying this action");
  return active;
}

async function render(session: number, file: number, options: DiffOptions): Promise<ReviewResult> {
  const selected = current(session);
  const source = selected.bundle.files[file];
  const { module, pack, syntaxUnavailable } = await engineForPath(source?.path ?? "");
  current(session);
  const state = await storage.readReview(selected.bundle.repositoryId);
  current(session);
  const backup = await storage.readStored(`review-backup:${selected.bundle.repositoryId}`);
  current(session);
  const canUndoImport = Boolean(backup && JSON.parse(backup).imported === state);
  const start = performance.now();
  const overview = JSON.parse(selected.session.overview(state));
  const diff = source ? JSON.parse(module.render_diff(source.path, source.before, source.after, options.ignoreWhitespace, options.showAll, options.split)) : null;
  const review = source ? JSON.parse(selected.session.file_review(file, state, options.ignoreWhitespace)) : { groupStates: [] };
  return { overview, diff, ...review, file, options, session, pack, syntaxUnavailable, canUndoImport, elapsed: performance.now() - start };
}

async function handle(request: WorkerRequest): Promise<unknown> {
  const base = await baseEngine;
  switch (request.kind) {
    case "init": {
      storage = new BrowserReviewStorage(request.databaseName);
      return JSON.parse(base.ui_labels(navigator.languages.join(",")));
    }
    case "open": {
      const raw = request.bundle ?? await storage.readStored("active-bundle");
      if (raw === undefined) return null;
      const session = new base.ReviewSession(raw);
      const bundle: ReviewBundle = JSON.parse(raw);
      await storage.writeStored("active-bundle", raw);
      active?.session.free();
      active = { bundle, session, version: ++version };
      return render(version, 0, defaultOptions);
    }
    case "render": return render(request.session, request.file, request.options);
    case "mutate": {
      const selected = current(request.session);
      await storage.updateReview(selected.bundle.repositoryId, state => { current(request.session); return selected.session.apply(JSON.stringify(request.action), state); });
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
      await storage.updateReview(selected.bundle.repositoryId, state => {
        current(request.session);
        if (state !== backup.imported) throw new Error("Review state changed after import; preserve those edits before restoring");
        return backup.previous;
      });
      return render(request.session, request.file, request.options);
    }
    case "importState": {
      const selected = current(request.session);
      selected.session.overview(request.state);
      await storage.updateReview(selected.bundle.repositoryId, () => { current(request.session); return request.state; }, true);
      return render(request.session, request.file, request.options);
    }
  }
}

self.onmessage = async ({ data }: MessageEvent<WorkerRequest & { id: number }>) => {
  let response: WorkerResponse;
  try { response = { id: data.id, result: await handle(data) }; }
  catch (error) { response = { id: data.id, error: error instanceof Error ? error.message : String(error) }; }
  self.postMessage(response);
};
