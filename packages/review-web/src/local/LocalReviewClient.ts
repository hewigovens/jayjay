import type { ReviewSelection } from "../model/backend";
import type { Labels } from "../model/diff";
import type { ReviewAction, ReviewResult } from "../model/review";
import type { WorkerRequest, WorkerResponse } from "./protocol";

export class LocalReviewClient {
  private worker = new Worker(new URL("./review.worker.ts", import.meta.url), { type: "module" });
  private sequence = 0;
  private failure?: Error;
  private pending = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void }>();
  private ready: Promise<Labels>;

  constructor({ databaseName }: { databaseName: string }) {
    this.worker.onmessage = ({ data }: MessageEvent<WorkerResponse>) => {
      const pending = this.pending.get(data.id);
      if (!pending) return;
      this.pending.delete(data.id);
      if ("error" in data) pending.reject(new Error(data.error));
      else pending.resolve(data.result);
    };
    this.worker.onerror = () => this.fail(new Error("worker-stopped"));
    this.ready = this.send({ kind: "init", databaseName });
    void this.ready.catch(() => {});
  }

  private send<T>(request: WorkerRequest): Promise<T> {
    if (this.failure) return Promise.reject(this.failure);
    const id = ++this.sequence;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve: value => resolve(value as T), reject });
      try { this.worker.postMessage({ ...request, id }); }
      catch (error) { this.pending.delete(id); reject(error); }
    });
  }

  private async request<T>(request: WorkerRequest): Promise<T> {
    await this.ready;
    return this.send(request);
  }

  labels() { return this.ready; }
  open(bundle: string): Promise<ReviewResult>;
  open(): Promise<ReviewResult | null>;
  open(bundle?: string) { return this.request<ReviewResult | null>({ kind: "open", bundle }); }
  render(selection: ReviewSelection) { return this.request<ReviewResult>({ kind: "render", ...selection }); }
  mutate(selection: ReviewSelection, action: ReviewAction) { return this.request<ReviewResult>({ kind: "mutate", ...selection, action }); }
  exportState(session: number) { return this.request<string>({ kind: "export", session }); }
  importState(selection: ReviewSelection, state: string) { return this.request<ReviewResult>({ kind: "importState", ...selection, state }); }
  restoreState(selection: ReviewSelection) { return this.request<ReviewResult>({ kind: "restoreState", ...selection }); }

  dispose() { this.worker.terminate(); this.fail(new Error("worker-stopped")); }

  private fail(error: Error) {
    this.failure = error;
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }
}
