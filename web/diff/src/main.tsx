import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { loadLabels } from "@jayjay/review-web/labels";
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { useDiffThemes } from "./hooks/useDiffThemes";
import { Landing } from "./components/landing/Landing";
import { PullReview } from "./components/review/PullReview";
import { parseReviewRef, reviewPath, type ReviewRef } from "./github/ReviewRef";
import "@jayjay/review-web/styles.css";
import "./style.css";

// Matches the static shell in index.html, so nothing jumps while the labels load.
function Loading({ error }: { error: string }) {
  return (
    <main aria-busy={!error}>
      <div className="landing">
        <div className="landing-wrap landing-hero">
          <h1>
            <img src="/favicon.svg" alt="" width="28" height="28" />
            JayJay Diff
          </h1>
          {error && <p role="alert">{error}</p>}
        </div>
      </div>
    </main>
  );
}

function App() {
  const { themes, select } = useDiffThemes();
  const [client, setClient] = useState<LocalReviewClient>();
  const [labels, setLabels] = useState<Labels>();
  const [error, setError] = useState("");
  const [path, setPath] = useState(location.pathname);

  useEffect(() => {
    void loadLabels()
      .then(setLabels)
      .catch((error) => setError(error.message));
    const update = () => setPath(location.pathname);
    addEventListener("popstate", update);
    return () => removeEventListener("popstate", update);
  }, []);

  const pull = parseReviewRef(path);
  useEffect(() => {
    if (pull && reviewPath(pull) !== location.pathname)
      history.replaceState(null, "", reviewPath(pull));
  }, [path]);
  // The diff engine starts with the first review, so the landing page never downloads it.
  useEffect(() => {
    if (pull && !client) setClient(new LocalReviewClient({ databaseName: "jayjay-pr" }));
  }, [Boolean(pull)]);
  useEffect(() => () => client?.dispose(), [client]);

  const open = (next: ReviewRef) => {
    history.pushState(null, "", reviewPath(next));
    setPath(reviewPath(next));
  };

  if (!labels) return <Loading error={error} />;
  return (
    <main className={pull ? "jayjay-review pull-page" : "jayjay-review"}>
      {pull ? (
        client && (
          <PullReview
            client={client}
            labels={labels}
            pull={pull}
            onOpen={open}
            themes={themes}
            selectTheme={select}
          />
        )
      ) : (
        <Landing labels={labels} onOpen={open} />
      )}
    </main>
  );
}

if (import.meta.env.PROD && "serviceWorker" in navigator)
  void navigator.serviceWorker.register("/sw.js").catch(() => {});

createRoot(document.getElementById("root")!).render(<App />);
