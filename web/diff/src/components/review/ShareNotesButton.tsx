import { useEffect, useState } from "react";
import { ToolButton } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import { ShareIcon } from "../icons";
import type { NotesExport } from "../../github/NotesExport";

export function ShareNotesButton({
  labels,
  notes,
}: {
  labels: Labels;
  notes: () => NotesExport | null;
}) {
  const [url, setUrl] = useState<string>();
  const [state, setState] = useState<"idle" | "sharing" | "copied" | "manual" | "failed">("idle");

  useEffect(() => {
    if (state !== "copied") return;
    const timeout = setTimeout(() => setState("idle"), 4000);
    return () => clearTimeout(timeout);
  }, [state]);

  async function share() {
    const body = notes();
    if (!body) return;
    setState("sharing");
    setUrl(undefined);
    const contents = fetch("/api/exports", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    }).then(async (response) => {
      if (!response.ok) throw new Error(String(response.status));
      const result: unknown = await response.json();
      if (
        !result ||
        typeof result !== "object" ||
        !("url" in result) ||
        typeof result.url !== "string"
      )
        throw new Error("Missing export URL");
      setUrl(result.url);
      return new Blob([result.url], { type: "text/plain" });
    });
    try {
      await navigator.clipboard.write([new ClipboardItem({ "text/plain": contents })]);
      setState("copied");
    } catch {
      try {
        await contents;
        setState("manual");
      } catch {
        setState("failed");
      }
    }
  }

  return (
    <span className="share-notes">
      {(state === "copied" || state === "manual") && (
        <span role="status" className="share-feedback">
          {state === "copied" ? labels.pullShareCopied : labels.pullShareCopyFailed}
          {state === "manual" && url && (
            <input
              className="share-url"
              readOnly
              value={url}
              aria-label={labels.pullShareLink}
              onFocus={(event) => event.currentTarget.select()}
            />
          )}
        </span>
      )}
      {state === "failed" && (
        <span role="alert" className="share-feedback share-failed">
          {labels.pullShareFailed}
        </span>
      )}
      <ToolButton
        disabled={state === "sharing"}
        label={labels.pullShareNotes}
        onClick={() => void share()}
      >
        <ShareIcon />
      </ToolButton>
    </span>
  );
}
