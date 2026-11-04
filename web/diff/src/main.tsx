import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { Landing } from "./components/landing/Landing";
import { PullReview } from "./components/review/PullReview";
import { parsePullRequest, pullPath, type PullRequestRef } from "./github/PullRequestRef";
import "@jayjay/review-web/styles.css";
import "./style.css";

function App() {
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

  const pull = parsePullRequest(path);
  useEffect(() => {
    if (pull && pullPath(pull) !== location.pathname)
      history.replaceState(null, "", pullPath(pull));
  }, [path]);

  const open = (next: PullRequestRef) => {
    history.pushState(null, "", pullPath(next));
    setPath(pullPath(next));
  };

  if (!labels || !client)
    return <main aria-busy={!error}>{error && <p role="alert">{error}</p>}</main>;
  return (
    <main className={pull ? "jayjay-review pull-page" : "jayjay-review"}>
      {pull ? (
        <PullReview client={client} labels={labels} pull={pull} onOpen={open} />
      ) : (
        <Landing labels={labels} onOpen={open} />
      )}
    </main>
  );
}

if (import.meta.env.PROD && "serviceWorker" in navigator)
  void navigator.serviceWorker.register("/sw.js").catch(() => {});

createRoot(document.getElementById("root")!).render(<App />);
