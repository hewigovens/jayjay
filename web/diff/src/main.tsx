import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { useDiffThemes } from "./hooks/useDiffThemes";
import { Landing } from "./components/landing/Landing";
import { PullReview } from "./components/review/PullReview";
import { parseReviewRef, reviewPath, type ReviewRef } from "./github/ReviewRef";
import "@jayjay/review-web/styles.css";
import "./style.css";

function App() {
  const { themes, select } = useDiffThemes();
  const [client, setClient] = useState<LocalReviewClient>();
  const [labels, setLabels] = useState<Labels>();
  const [error, setError] = useState("");
  const [path, setPath] = useState(location.pathname);

  useEffect(() => {
    const instance = new LocalReviewClient({ databaseName: "jayjay-pr" });
    setClient(instance);
    void instance
      .labels()
      .then(setLabels)
      .catch((error) => setError(error.message));
    const update = () => setPath(location.pathname);
    addEventListener("popstate", update);
    return () => {
      instance.dispose();
      removeEventListener("popstate", update);
    };
  }, []);

  const pull = parseReviewRef(path);
  useEffect(() => {
    if (pull && reviewPath(pull) !== location.pathname)
      history.replaceState(null, "", reviewPath(pull));
  }, [path]);

  const open = (next: ReviewRef) => {
    history.pushState(null, "", reviewPath(next));
    setPath(reviewPath(next));
  };

  if (!labels || !client)
    return <main aria-busy={!error}>{error && <p role="alert">{error}</p>}</main>;
  return (
    <main className={pull ? "jayjay-review pull-page" : "jayjay-review"}>
      {pull ? (
        <PullReview
          client={client}
          labels={labels}
          pull={pull}
          onOpen={open}
          themes={themes}
          selectTheme={select}
        />
      ) : (
        <Landing labels={labels} onOpen={open} />
      )}
    </main>
  );
}

if (import.meta.env.PROD && "serviceWorker" in navigator)
  void navigator.serviceWorker.register("/sw.js").catch(() => {});

createRoot(document.getElementById("root")!).render(<App />);
