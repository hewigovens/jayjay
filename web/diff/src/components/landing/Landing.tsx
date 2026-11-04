import { useState } from "react";
import type { Labels } from "@jayjay/review-web/types";
import { ArrowIcon, GitHubIcon, StackIcon, SyncIcon } from "../icons";
import { parseReviewRef, reviewPath, type ReviewRef } from "../../github/ReviewRef";
import { samples } from "../../github/samples";

const repository = "https://github.com/hewigovens/jayjay";
const app = "https://jayjay.hewig.dev";
const extension =
  "https://chromewebstore.google.com/detail/open-in-diffshub/pgmcdokikomeilbkobgfmlihmilingpl";

export function Landing({ labels, onOpen }: { labels: Labels; onOpen: (pull: ReviewRef) => void }) {
  const [invalid, setInvalid] = useState(false);
  const points = [
    [["J", "K"], labels.pullHowMove, labels.pullHowMoveDetail],
    [["Space"], labels.pullHowMark, labels.pullHowMarkDetail],
    [<StackIcon size={18} />, labels.pullFeatureStacks, labels.pullFeatureStacksDetail],
    [<SyncIcon size={18} />, labels.pullFeatureSync, labels.pullFeatureSyncDetail],
  ] as const;

  return (
    <div className="landing">
      <div className="landing-wrap landing-hero">
        <h1>
          <img src="/favicon.svg" alt="" width="28" height="28" />
          JayJay Diff
        </h1>
        <p className="landing-tagline">{labels.pullHeading}</p>
        <p className="landing-lede">{labels.pullLede}</p>
        <form
          className="landing-field"
          onSubmit={(event) => {
            event.preventDefault();
            const pull = parseReviewRef(
              new FormData(event.currentTarget).get("url")?.toString() ?? "",
            );
            setInvalid(!pull);
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
            aria-invalid={invalid}
            aria-describedby={invalid ? "landing-error" : undefined}
            placeholder={`https://github.com${reviewPath(samples[1])}`}
            onInput={() => setInvalid(false)}
          />
          <button type="submit" aria-label={labels.pullOpen} title={labels.pullOpen}>
            <ArrowIcon size={20} />
          </button>
        </form>
        {invalid && (
          <p id="landing-error" className="landing-error" role="alert">
            {labels.pullInvalid}
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
        <p className="landing-extension">
          <a href={extension}>{labels.pullExtension}</a> {labels.pullExtensionDetail}
        </p>
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
