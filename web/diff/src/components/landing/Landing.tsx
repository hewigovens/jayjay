import { useRef, useState, type ClipboardEvent, type DragEvent } from "react";
import { HideReviewedIcon, TreeIcon } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import { ArrowIcon, GitHubIcon, StackIcon, SyncIcon } from "../icons";
import {
  parseReviewRef,
  reviewName,
  reviewPath,
  threeDotComparison,
  type ReviewRef,
} from "../../github/ReviewRef";
import { samples } from "../../github/samples";
import { parsePatch, PatchError, type Patch } from "../../Patch";

const repository = "https://github.com/hewigovens/jayjay";
const app = "https://jayjay.hewig.dev";
const extension =
  "https://chromewebstore.google.com/detail/open-in-diffshub/pgmcdokikomeilbkobgfmlihmilingpl";

export function Landing({
  labels,
  onOpen,
  onOpenPatch,
  unsupported,
}: {
  labels: Labels;
  onOpen: (pull: ReviewRef) => void;
  onOpenPatch: (patch: Patch) => void;
  /** A forwarded path that names no review. */
  unsupported?: string;
}) {
  const sent = unsupported && `https://github.com${unsupported}`;
  const [rejected, setRejected] = useState<{ input: string; typed: boolean } | null>(
    sent ? { input: sent, typed: false } : null,
  );
  const suggestion = rejected && threeDotComparison(rejected.input);
  const [patchError, setPatchError] = useState<PatchError["code"]>();
  const [dropping, setDropping] = useState(false);
  const picker = useRef<HTMLInputElement>(null);

  async function openPatch(text: string, title: string, name?: string) {
    try {
      onOpenPatch(await parsePatch(text, title, name));
    } catch (error) {
      if (!(error instanceof PatchError)) throw error;
      setRejected(null);
      setPatchError(error.code);
    }
  }

  async function openFile(file: File | undefined) {
    if (file) await openPatch(await file.text(), file.name, file.name);
  }

  function drop(event: DragEvent) {
    event.preventDefault();
    setDropping(false);
    void openFile(event.dataTransfer.files[0]);
  }

  function paste(event: ClipboardEvent) {
    const text = event.clipboardData.getData("text/plain");
    if (!/^diff --git /m.test(text)) return;
    event.preventDefault();
    void openPatch(text, labels.pullPatchPasted);
  }
  const points = [
    [["J", "K"], labels.pullHowMove, labels.pullHowMoveDetail],
    [["Space"], labels.pullHowMark, labels.pullHowMarkDetail],
    [<HideReviewedIcon size={18} />, labels.pullFeatureHide, labels.pullFeatureHideDetail],
    [<TreeIcon size={18} />, labels.pullFeatureTree, labels.pullFeatureTreeDetail],
    [<StackIcon size={18} />, labels.pullFeatureStacks, labels.pullFeatureStacksDetail],
    [<SyncIcon size={18} />, labels.pullFeatureSync, labels.pullFeatureSyncDetail],
  ] as const;

  return (
    <div
      className={dropping ? "landing landing-dropping" : "landing"}
      onDragOver={(event) => {
        if (!event.dataTransfer.types.includes("Files")) return;
        event.preventDefault();
        setDropping(true);
      }}
      onDragLeave={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setDropping(false);
      }}
      onDrop={drop}
      onPaste={paste}
    >
      <div className="landing-wrap landing-hero">
        <h1>
          <img src="/favicon.svg" alt="" width="28" height="28" />
          JayJay Diff
        </h1>
        <p className="landing-tagline">{labels.pullHeading}</p>
        <p className="landing-lede">
          <strong>{labels.pullLedeLead}</strong> {labels.pullLede}
        </p>
        <form
          className="landing-field"
          onSubmit={(event) => {
            event.preventDefault();
            const input = new FormData(event.currentTarget).get("url")?.toString() ?? "";
            const pull = parseReviewRef(input);
            setRejected(pull ? null : { input, typed: true });
            if (pull) onOpen(pull);
          }}
        >
          <input
            name="url"
            type="text"
            autoFocus
            spellCheck={false}
            autoComplete="off"
            aria-label={labels.pullUrl}
            defaultValue={sent || undefined}
            aria-invalid={Boolean(rejected)}
            aria-describedby={rejected ? "landing-error" : undefined}
            placeholder={`https://github.com${reviewPath(samples[1])}`}
            onInput={() => {
              setRejected(null);
              setPatchError(undefined);
            }}
          />
          <button type="submit" aria-label={labels.pullOpen} title={labels.pullOpen}>
            <ArrowIcon size={20} />
          </button>
        </form>
        {rejected && (
          <div id="landing-error" className="landing-error" role="alert">
            {!rejected.typed && (
              <p>
                {labels.pullUnsupported} <code>{rejected.input}</code>
              </p>
            )}
            {suggestion ? (
              <p>
                {labels.pullTwoDot}{" "}
                <a
                  href={reviewPath(suggestion)}
                  onClick={(event) => {
                    event.preventDefault();
                    onOpen(suggestion);
                  }}
                >
                  <code>{reviewName(suggestion)}</code>
                </a>
              </p>
            ) : (
              <p>
                {rejected.typed && `${labels.pullInvalid} `}
                {labels.pullSupported}
              </p>
            )}
          </div>
        )}
        <p className="landing-patch">
          <button type="button" onClick={() => picker.current?.click()}>
            {labels.pullPatchOpen}
          </button>
          {labels.pullPatchDetail}
          <input
            ref={picker}
            type="file"
            accept=".diff,.patch,.txt,text/*"
            hidden
            onChange={(event) => {
              void openFile(event.currentTarget.files?.[0]);
              event.currentTarget.value = "";
            }}
          />
          <span aria-hidden="true"> · </span>
          <a href={extension}>{labels.pullExtension}</a> {labels.pullExtensionDetail}
        </p>
        {patchError && (
          <p className="landing-error" role="alert">
            {labels[`patchError.${patchError}`]}
          </p>
        )}
        <section className="landing-samples" aria-labelledby="landing-samples">
          <h2 id="landing-samples">{labels.pullSamples}</h2>
          <ul>
            {samples.map((sample) => (
              <li key={reviewPath(sample)}>
                <a
                  href={reviewPath(sample)}
                  onClick={(event) => {
                    event.preventDefault();
                    onOpen(sample);
                  }}
                >
                  <span aria-hidden="true">→</span>
                  <code>{`${sample.owner}/${sample.repo}#${sample.number}`}</code>
                  {sample.title}
                </a>
              </li>
            ))}
          </ul>
        </section>
      </div>
      <section className="landing-wrap landing-section" aria-labelledby="landing-how">
        <h2 id="landing-how">{labels.pullHow}</h2>
        <div className="landing-points">
          {points.map(([mark, title, detail]) => (
            <div key={title}>
              <h3>
                {Array.isArray(mark) ? mark.map((key) => <kbd key={key}>{key}</kbd>) : mark}
                {title}
              </h3>
              <p>{detail}</p>
            </div>
          ))}
        </div>
      </section>
      <footer className="landing-wrap landing-footer">
        <p>
          {labels.pullFooterLead} <a href={app}>JayJay</a>
          {labels.pullFooter}
        </p>
        <a href={repository} aria-label={labels.pullSource} title={labels.pullSource}>
          <GitHubIcon size={18} />
        </a>
      </footer>
    </div>
  );
}
