import { useEffect, useMemo, useState } from "react";
import {
  ReviewWorkspace,
  ToolButton,
  ShortcutsDialog,
  KeyboardIcon,
  useReview,
} from "@jayjay/review-web/react";
import type { LocalReviewClient } from "@jayjay/review-web/local";
import type { Labels, ReviewBackend } from "@jayjay/review-web/types";
import type { AppearanceSettings } from "../../AppearanceSettings";
import type { Patch } from "../../Patch";
import { DiffThemePicker } from "./DiffThemePicker";

export function PatchReview({
  client,
  labels,
  patch,
  themes,
  selectTheme,
}: {
  themes: AppearanceSettings;
  selectTheme: (themes: AppearanceSettings) => void;
  client: LocalReviewClient;
  labels: Labels;
  patch: Patch;
}) {
  const backend = useMemo(
    (): ReviewBackend => ({
      // The worker keeps an opened bundle in IndexedDB, so the patch's text is handed over afterwards instead.
      open: async () => {
        const result = await client.open(patch.bundle);
        await Promise.all(
          patch.contents.map(({ before, after }, index) =>
            client.load(result.session, index, before, after),
          ),
        );
        return result;
      },
      render: (selection) => client.render(selection),
      mutate: (selection, action) => client.mutate(selection, action),
    }),
    [client, patch],
  );
  const review = useReview(backend);
  const [shortcuts, setShortcuts] = useState(false);
  useEffect(() => {
    document.title = `${patch.title} · JayJay Diff`;
  }, [patch]);

  return (
    <>
      {review.result && (
        <nav className="pull-app-bar">
          <a href="/" className="pull-home">
            <img src="/favicon.svg" alt="" width="20" height="20" />
            JayJay Diff
          </a>
          <div className="pull-app-actions">
            <DiffThemePicker labels={labels} themes={themes} select={selectTheme} />
            <ToolButton
              label={labels.shortcuts}
              aria-keyshortcuts="?"
              onClick={() => setShortcuts(true)}
            >
              <KeyboardIcon />
            </ToolButton>
          </div>
        </nav>
      )}
      {shortcuts && <ShortcutsDialog labels={labels} onClose={() => setShortcuts(false)} />}
      <ReviewWorkspace
        review={review}
        diffThemes={themes}
        hideChangeId
        onShowShortcuts={() => setShortcuts(true)}
        labels={labels}
        brand={
          <a className="pull-brand" href="/" aria-label="JayJay Diff" title="JayJay Diff">
            <img src="/favicon.svg" alt="" width="28" height="28" />
          </a>
        }
        allLines={false}
        fill
        onRetry={null}
      />
    </>
  );
}
