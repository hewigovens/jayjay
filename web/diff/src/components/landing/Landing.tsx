import { useState } from "react";
import type { Labels } from "@jayjay/review-web/types";
import { CheckIcon, FileIcon, StackIcon, SyncIcon } from "../icons";
import { parseReviewRef, reviewPath, type ReviewRef } from "../../github/ReviewRef";
import { samples } from "../../github/samples";

const repository = "https://github.com/hewigovens/jayjay";
const app = "https://jayjay.hewig.dev";

export function Landing({ labels, onOpen }: { labels: Labels; onOpen: (pull: ReviewRef) => void }) {
  const [invalid, setInvalid] = useState(false);
  const features = [
    [<CheckIcon size={18} />, labels.pullFeatureMarks, labels.pullFeatureMarksDetail],
    [<StackIcon size={18} />, labels.pullFeatureStacks, labels.pullFeatureStacksDetail],
    [<SyncIcon size={18} />, labels.pullFeatureSync, labels.pullFeatureSyncDetail],
  ] as const;
  const steps = [
    [["J", "K"], labels.pullHowMove, labels.pullHowMoveDetail],
    [["Space"], labels.pullHowMark, labels.pullHowMarkDetail],
    [["?"], labels.pullHowMore, labels.pullHowMoreDetail],
  ] as const;

  const eyebrow = (text: string) => (
    <>
      <span>{text[0]}</span>
      {text.slice(1)}
    </>
  );

  return (
    <div className="landing">
      <nav className="landing-nav">
        <div className="landing-wrap">
          <a className="landing-brand" href="/">
            <img src="/icon.svg" alt="" width="24" height="24" />
            JayJay Diff
          </a>
          <div className="landing-links">
            <a href="#landing-how">{labels.pullHow}</a>
            <a href={repository}>{labels.pullSource}</a>
            <a className="landing-get" href={app}>
              {labels.pullApp}
            </a>
          </div>
        </div>
      </nav>
      <form
        className="landing-wrap landing-hero"
        onSubmit={(event) => {
          event.preventDefault();
          const pull = parseReviewRef(
            new FormData(event.currentTarget).get("url")?.toString() ?? "",
          );
          setInvalid(!pull);
          if (pull) onOpen(pull);
        }}
      >
        <img className="landing-icon" src="/icon.svg" alt="" width="104" height="104" />
        <h1>{labels.pullHeading}</h1>
        <p className="landing-lede">{labels.pullLede}</p>
        <div className="landing-row">
          <input
            name="url"
            type="text"
            autoFocus
            spellCheck={false}
            autoComplete="off"
            aria-label={labels.pullUrl}
            aria-invalid={invalid}
            aria-describedby="landing-hint"
            placeholder={`github.com${reviewPath(samples[1])}`}
            onInput={() => setInvalid(false)}
          />
          <button type="submit">{labels.pullOpen}</button>
        </div>
        <p id="landing-hint" className="landing-hint" role={invalid ? "alert" : undefined}>
          {invalid ? (
            labels.pullInvalid
          ) : (
            <>
              {labels.pullHint}{" "}
              <code>
                <s>github.com</s>/owner/repo/pull/1 → <em>{location.host}</em>/owner/repo/pull/1
              </code>
            </>
          )}
        </p>
        <div className="landing-pillars" aria-label={labels.pullFeatures}>
          {features.map(([icon, title, detail]) => (
            <div key={title}>
              <h3>
                {icon}
                {title}
              </h3>
              <p>{detail}</p>
            </div>
          ))}
        </div>
      </form>
      <section className="landing-wrap landing-section" aria-labelledby="landing-samples">
        <h2 id="landing-samples" className="landing-eyebrow">
          {eyebrow(labels.pullSamples)}
        </h2>
        <div className="landing-samples">
          {samples.map((sample) => (
            <a
              key={reviewPath(sample)}
              className="landing-sample"
              href={reviewPath(sample)}
              onClick={(event) => {
                event.preventDefault();
                onOpen(sample);
              }}
            >
              <code>{`${sample.owner}/${sample.repo} #${sample.number}`}</code>
              <span className="landing-sample-title">{sample.title}</span>
              <span className="landing-sample-stat">
                <FileIcon size={16} />
                {sample.files} <ins>+{sample.added}</ins> <del>−{sample.removed}</del>
              </span>
            </a>
          ))}
        </div>
      </section>
      <section className="landing-wrap landing-section" aria-labelledby="landing-how">
        <h2 id="landing-how" className="landing-eyebrow">
          {eyebrow(labels.pullHow)}
        </h2>
        <div className="landing-pillars">
          {steps.map(([keys, title, detail]) => (
            <div key={title}>
              <h3>
                {keys.map((key) => (
                  <kbd key={key}>{key}</kbd>
                ))}
                {title}
              </h3>
              <p>{detail}</p>
            </div>
          ))}
        </div>
      </section>
      <footer className="landing-footer">
        <div className="landing-wrap">
          <a href={app}>{labels.pullFooter}</a>
          <a href={repository}>{labels.pullSource}</a>
        </div>
      </footer>
    </div>
  );
}
