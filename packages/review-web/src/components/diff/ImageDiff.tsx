/** @jsxRuntime automatic */
import { useEffect, useRef, useState } from "react";
import type { Labels } from "../../model/diff";
import type { FileKind, ImageSide } from "../../model/review";

type Loaded = { url: string; bytes: number; width: number; height: number } | "missing" | "failed" | null;

/** One side of an image change, fetched from the backend as a blob and shown with its size. */
function Side({ side, label, load, labels }: { side: ImageSide; label: string; load: (side: ImageSide, signal: AbortSignal) => Promise<Blob | null>; labels: Labels }) {
  const [state, setState] = useState<Loaded>(null);
  // The host's loader is a fresh closure each render; the parent's key already remounts this for another file.
  const loader = useRef(load);
  loader.current = load;
  useEffect(() => {
    const controller = new AbortController();
    let url = "";
    setState(null);
    loader.current(side, controller.signal).then(blob => {
      if (controller.signal.aborted) return;
      if (!blob) { setState("missing"); return; }
      url = URL.createObjectURL(blob);
      const image = new Image();
      image.onload = () => !controller.signal.aborted && setState({ url, bytes: blob.size, width: image.naturalWidth, height: image.naturalHeight });
      image.onerror = () => !controller.signal.aborted && setState("failed");
      image.src = url;
    }, () => !controller.signal.aborted && setState("failed"));
    return () => { controller.abort(); if (url) URL.revokeObjectURL(url); };
  }, [side]);
  return <figure className={`jj-image-side jj-${side}`}>
    <figcaption><span className="jj-image-label">{label}</span>
      {state && typeof state === "object" && <span className="jj-image-size">{state.width} × {state.height} · {size(state.bytes)}</span>}</figcaption>
    <div className="jj-image-frame">
      {state === null ? <p className="jj-notice">{labels.loadingFile}</p>
        : state === "missing" ? <p className="jj-notice">{labels.previewUnavailable}</p>
        : state === "failed" ? <p className="jj-notice">{labels.renderError}</p>
        : <img src={state.url} alt={`${label}: ${state.width} × ${state.height}`} />}
    </div>
  </figure>;
}

function size(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** An image's before and after, side by side; an added or removed image shows its one side. */
export function ImageDiff({ kind, load, labels }: { kind: FileKind | undefined; load: (side: ImageSide, signal: AbortSignal) => Promise<Blob | null>; labels: Labels }) {
  return <div className="jj-image-diff">
    {kind !== "added" && <Side side="old" label={labels.before} load={load} labels={labels} />}
    {kind !== "removed" && <Side side="new" label={labels.after} load={load} labels={labels} />}
  </div>;
}
