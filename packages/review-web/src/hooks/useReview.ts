import { useEffect, useRef, useState } from "react";
import type { ReviewBackend, ReviewSelection } from "../model/backend";
import { defaultOptions, type DiffOptions, type ImageSide, type ReviewAction, type ReviewResult } from "../model/review";

type Operation = (signal: AbortSignal) => Promise<ReviewResult>;
type RunKind = "render" | "mutate" | "replace";

export function useReview(backend: ReviewBackend) {
  const [result, setResult] = useState<ReviewResult | null>(null);
  const [selected, setSelected] = useState(0);
  const [options, setOptions] = useState(defaultOptions);
  const [pending, setPending] = useState(true);
  const [error, setError] = useState("");
  const generation = useRef(0);
  const current = useRef<ReviewResult | null>(null);
  const [revision, setRevision] = useState(0);
  const lastOperation = useRef<{ operation: Operation; kind: RunKind } | undefined>(undefined);
  const cancellation = useRef<AbortController | undefined>(undefined);
  // Opens, imports, and mutations commit in the backend even when superseded, so only renders may replace an in-flight operation.
  const committing = useRef(false);

  // Renders may be superseded; mutations touch one file; anything else (open, import, undo) can change every file's review state.
  async function run(operation: Operation, kind: RunKind = "replace"): Promise<boolean> {
    const render = kind === "render";
    if (committing.current) return false;
    cancellation.current?.abort();
    const controller = new AbortController();
    cancellation.current = controller;
    const token = ++generation.current;
    // Retry replays only opens and renders; a failed mutation may have committed, so retrying it re-renders instead of writing twice.
    if (render || !current.current) lastOperation.current = { operation, kind };
    committing.current = !render;
    setPending(true); setError("");
    try {
      const next = await operation(controller.signal);
      if (token !== generation.current) return false;
      current.current = next; setResult(next); setSelected(next.file); setOptions(next.options); setPending(false);
      lastOperation.current = { operation: signal => backend.render({ session: next.session, file: next.file, options: next.options }, signal), kind: "render" };
      if (kind === "replace") setRevision(value => value + 1);
      return true;
    } catch (error) {
      if (token === generation.current) { setError(error instanceof Error ? error.message : String(error)); setPending(false); }
      return false;
    } finally {
      if (token === generation.current) committing.current = false;
    }
  }

  useEffect(() => {
    current.current = null; setResult(null); setSelected(0); setOptions(defaultOptions);
    void run(signal => backend.open(signal));
    return () => { generation.current++; committing.current = false; cancellation.current?.abort(); };
  }, [backend]);

  function select(file: number) {
    const active = current.current;
    if (!active || committing.current) return;
    setSelected(file);
    void run(signal => backend.render({ session: active.session, file, options }, signal), "render");
  }

  function configure(next: DiffOptions, file = selected): boolean {
    if (committing.current) return false;
    const active = current.current;
    setSelected(file); setOptions(next);
    if (active) void run(signal => backend.render({ session: active.session, file, options: next }, signal), "render");
    return Boolean(active);
  }

  function mutate(action: ReviewAction, file?: number) {
    const active = current.current;
    if (!active) return Promise.resolve(false);
    return run(signal => backend.mutate({ session: active.session, file: file ?? active.file, options: active.options }, action, signal), "mutate");
  }

  // Renders another file of the current session for the all-files list without changing the selection.
  function renderFile(file: number): Promise<ReviewResult> {
    const active = current.current;
    if (!active) return Promise.reject(new Error("No review is open"));
    return backend.render({ session: active.session, file, options: active.options });
  }

  const selection: ReviewSelection | undefined = result ? { session: result.session, file: result.file, options: result.options } : undefined;
  // Loads one side of an image file of the current session; undefined when the backend has no images.
  const image = backend.image && ((file: number, side: ImageSide, signal: AbortSignal) => {
    const active = current.current;
    if (!active) return Promise.reject(new Error("No review is open"));
    return backend.image!({ session: active.session, file, options: active.options }, side, signal);
  });

  return { result, selected, options, pending, error, selection, revision, select, configure, mutate, renderFile, run, image,
    retry: () => { if (lastOperation.current) void run(lastOperation.current.operation, lastOperation.current.kind); },
  };
}

export type ReviewController = ReturnType<typeof useReview>;
