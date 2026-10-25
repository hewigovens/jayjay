import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels } from "@jayjay/review-web/types";
import { LocalReview } from "./LocalReview";
import "@jayjay/review-web/styles.css";
import "./style.css";

function Host() {
  const [client, setClient] = useState<LocalReviewClient>();
  const [labels, setLabels] = useState<Labels>();
  const [error, setError] = useState("");

  useEffect(() => {
    const instance = new LocalReviewClient({ databaseName: "jayjay-review-tests" });
    setClient(instance);
    void instance.labels().then(setLabels).catch(error => setError(error.message));
    return () => instance.dispose();
  }, []);

  if (!labels || !client) return <main aria-busy={!error}>{error && <p role="alert">{error}</p>}</main>;
  return <main className="jayjay-review"><LocalReview client={client} labels={labels} /></main>;
}

createRoot(document.getElementById("root")!).render(<Host />);
