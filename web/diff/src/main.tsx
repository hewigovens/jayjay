import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { loadLabels } from "@jayjay/review-web/labels";
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { useDiffThemes } from "./hooks/useDiffThemes";
import { Landing } from "./components/landing/Landing";
import { PatchReview } from "./components/review/PatchReview";
import { PullReview } from "./components/review/PullReview";
import { parseReviewRef, reviewPath, type ReviewRef } from "./github/ReviewRef";
import type { Patch } from "./Patch";
import "@jayjay/review-web/styles.css";
import "./style.css";

const patchPath = "/patch";

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
  const [opened, setOpened] = useState<Patch>();

  useEffect(() => {
    void loadLabels()
      .then(setLabels)
      .catch((error) => setError(error.message));
    const update = () => setPath(location.pathname);
    addEventListener("popstate", update);
    return () => removeEventListener("popstate", update);
  }, []);

  const pull = parseReviewRef(path);
  // A patch lives only in this tab's memory, so a reload of its page has nothing to show.
  const patch = path === patchPath ? opened : undefined;
  useEffect(() => {
    if (pull && reviewPath(pull) !== location.pathname)
      history.replaceState(null, "", reviewPath(pull));
    if (path === patchPath && !opened) history.replaceState(null, "", "/");
  }, [path]);
  // The diff engine starts with the first review, so the landing page never downloads it.
  useEffect(() => {
    if ((pull || patch) && !client) setClient(new LocalReviewClient({ databaseName: "jayjay-pr" }));
  }, [Boolean(pull || patch)]);
  useEffect(() => () => client?.dispose(), [client]);

  const open = (next: ReviewRef) => {
    history.pushState(null, "", reviewPath(next));
    setPath(reviewPath(next));
  };

  if (!labels) return <Loading error={error} />;
  return (
    <main className={pull || patch ? "jayjay-review pull-page" : "jayjay-review"}>
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
      ) : patch ? (
        client && (
          <PatchReview
            client={client}
            labels={labels}
            patch={patch}
            themes={themes}
            selectTheme={select}
          />
        )
      ) : (
        <Landing
          key={path}
          labels={labels}
          onOpen={open}
          onOpenPatch={(next) => {
            setOpened(next);
            history.pushState(null, "", patchPath);
            setPath(patchPath);
          }}
          unsupported={path === "/" || path === patchPath ? undefined : path + location.search}
        />
      )}
    </main>
  );
}

if (import.meta.env.PROD && "serviceWorker" in navigator)
  void navigator.serviceWorker.register("/sw.js").catch(() => {});

createRoot(document.getElementById("root")!).render(<App />);
