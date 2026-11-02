import { useState } from "react";
import type { Labels } from "@jayjay/review-web/types";
import { parsePullRequest, pullPath, type PullRequestRef } from "../../github/PullRequestRef";
import { samples } from "../../github/samples";

export function Landing({
  labels,
  onOpen,
}: {
  labels: Labels;
  onOpen: (pull: PullRequestRef) => void;
}) {
  const [invalid, setInvalid] = useState(false);

  return (
    <form
      className="landing"
      onSubmit={(event) => {
        event.preventDefault();
        const pull = parsePullRequest(
          new FormData(event.currentTarget).get("url")?.toString() ?? "",
        );
        setInvalid(!pull);
        if (pull) onOpen(pull);
      }}
    >
      <h1>{labels.pullHeading}</h1>
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
          placeholder={`github.com${pullPath({ owner: "jj-vcs", repo: "jj", number: 1 })}`}
          onInput={() => setInvalid(false)}
        />
        <button type="submit">{labels.pullOpen}</button>
      </div>
      <p id="landing-hint" role={invalid ? "alert" : undefined}>
        {invalid ? labels.pullInvalid : labels.pullHint}
      </p>
      <nav className="landing-samples" aria-label={labels.pullSamples}>
        <span>{labels.pullSamples}</span>
        {samples.map((sample) => (
          <a
            key={pullPath(sample)}
            href={pullPath(sample)}
            onClick={(event) => {
              event.preventDefault();
              onOpen(sample);
            }}
          >
            {`${sample.owner}/${sample.repo}#${sample.number}`}
          </a>
        ))}
      </nav>
    </form>
  );
}
