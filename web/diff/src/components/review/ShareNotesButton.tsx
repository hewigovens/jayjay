import { useState } from "react";
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
  const [state, setState] = useState<"idle" | "sharing" | "failed">("idle");

  async function share() {
    const body = notes();
    if (!body) return;
    setState("sharing");
    try {
      const response = await fetch("/api/exports", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!response.ok) throw new Error(String(response.status));
      const { url } = (await response.json()) as { url: string };
      setUrl(url);
      setState("idle");
      await navigator.clipboard?.writeText(url).catch(() => {});
    } catch {
      setState("failed");
    }
  }

  return (
    <span className="share-notes">
      {url && (
        <input
          className="share-url"
          readOnly
          value={url}
          aria-label={labels.pullShareCopied}
          title={labels.pullShareCopied}
          onFocus={(event) => event.currentTarget.select()}
        />
      )}
      {state === "failed" && (
        <span role="alert" className="share-failed">
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
