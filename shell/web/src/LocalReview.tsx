import { useMemo, useState } from "react";
import { ReviewWorkspace, useReview } from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels, ReviewBackend } from "@jayjay/review-web/types";
import { fixture } from "./fixture";

async function readJson(file: File, limit: number, signal: AbortSignal) {
  if (file.size > limit * 1024 * 1024) throw new Error(`Review import exceeds ${limit} MiB`);
  const text = new TextDecoder("utf-8", { fatal: true }).decode(await file.arrayBuffer());
  signal.throwIfAborted();
  return text;
}

const avatarsKey = "jayjay.review.avatars";

function storedAvatars() {
  try { return localStorage.getItem(avatarsKey) === "on"; } catch { return false; }
}

export function LocalReview({ client, labels }: { client: LocalReviewClient; labels: Labels }) {
  const [avatars, setAvatars] = useState(storedAvatars);
  const backend = useMemo<ReviewBackend>(() => ({
    open: async () => await client.open() ?? await client.open(JSON.stringify(fixture)),
    render: selection => client.render(selection),
    mutate: (selection, action) => client.mutate(selection, action),
  }), [client]);
  const review = useReview(backend);
  const { result, selection, pending, run } = review;

  function toggleAvatars(on: boolean) {
    setAvatars(on);
    try { localStorage.setItem(avatarsKey, on ? "on" : "off"); } catch { /* the choice lasts for this page */ }
  }

  return <ReviewWorkspace review={review} labels={labels} savedLabel={labels.savedLocally} avatars={avatars}
    onRetry={() => review.error === "worker-stopped" ? location.reload() : review.retry()}
    headerActions={<label className="file-button import-button">{labels.importChange}<input type="file" accept="application/json,.json" aria-label={labels.importChange} disabled={pending} onChange={event => {
      const file = event.target.files?.[0];
      if (file) void run(async signal => client.open(await readJson(file, 32, signal)));
      event.target.value = "";
    }} /></label>}
    footer={<footer className="review-footer"><span>{labels.private}</span><details><summary>{labels.localData}</summary>
      <div><label><input type="checkbox" checked={avatars} onChange={event => toggleAvatars(event.target.checked)} />{labels.loadAvatars}</label>
        <button disabled={!result || pending} onClick={() => result && void run(async () => {
        const state = await client.exportState(result.session);
        const url = URL.createObjectURL(new Blob([state], { type: "application/json" }));
        const anchor = document.createElement("a"); anchor.href = url; anchor.download = "review_store.json"; anchor.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        return result;
      })}>{labels.exportReview}</button>
        <button disabled={!result?.canUndoImport || pending} onClick={() => selection && void run(() => client.restoreState(selection))}>{labels.undoImport}</button>
        <p>{labels.importReplaces}</p><label className="file-button import-button">{labels.importReview}<input type="file" accept="application/json,.json" aria-label={labels.importReview} disabled={!result || pending} onChange={event => {
          const file = event.target.files?.[0];
          if (file && selection) void run(async signal => client.importState(selection, await readJson(file, 16, signal)));
          event.target.value = "";
        }} /></label></div>
    </details></footer>} />;
}
