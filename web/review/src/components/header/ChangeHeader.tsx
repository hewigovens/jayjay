/** @jsxRuntime automatic */
import { useState, type ReactNode } from "react";
import type { Labels } from "../../model/diff";
import type { Overview } from "../../model/review";
import { Avatar } from "../Avatar";
import { ChangeDescription } from "./ChangeDescription";
import { ExpandIcon } from "../icons";
import { ToolButton } from "../ToolButton";
import { DiffStats } from "./DiffStats";

export function ChangeHeader({
  overview,
  labels,
  hideChangeId = false,
  actions,
  avatars,
  inlineDescription = true,
}: {
  overview: Overview;
  labels: Labels;
  hideChangeId?: boolean;
  actions?: ReactNode;
  avatars: boolean;
  /** Off where the description has its own view, so the header is the title and byline alone. */
  inlineDescription?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const description = inlineDescription && overview.description;
  const added = overview.files.reduce((sum, file) => sum + file.added, 0);
  const removed = overview.files.reduce((sum, file) => sum + file.removed, 0);
  const prefix = Math.min(overview.changeIdPrefix ?? 2, overview.changeId.length);
  const date = overview.timestampMillis
    ? new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
        overview.timestampMillis,
      )
    : null;
  const toggleLabel = expanded ? labels.collapseDescription : labels.expandDescription;
  return (
    <header className="jj-change-header">
      <div className="jj-change-title">
        <h1>{overview.title}</h1>
        {description && (
          <ToolButton
            className="jj-description-toggle"
            aria-expanded={expanded}
            label={toggleLabel}
            onClick={() => setExpanded(!expanded)}
          >
            <ExpandIcon expanded={expanded} />
          </ToolButton>
        )}
        {actions && <div className="jj-change-actions">{actions}</div>}
      </div>
      {description && (
        <ChangeDescription
          overview={overview}
          className={`jj-change-description ${expanded ? "jj-expanded" : ""}`}
        />
      )}
      <div className="jj-byline">
        {overview.author && (
          <>
            <Avatar source={avatars ? overview.authorAvatar : null} name={overview.author} />
            <span className="jj-author">{overview.author}</span>
            {(!hideChangeId || date) && (
              <span className="jj-separator" aria-hidden="true">
                ·
              </span>
            )}
          </>
        )}
        {!hideChangeId && (
          <code className="jj-change-id" title={overview.changeId}>
            {prefix > 0 && <b>{overview.changeId.slice(0, prefix)}</b>}
            {overview.changeId.slice(prefix)}
          </code>
        )}
        {date && (
          <>
            {!hideChangeId && (
              <span className="jj-separator" aria-hidden="true">
                ·
              </span>
            )}
            <time>{date}</time>
          </>
        )}
        <span className="jj-spacer" />
        <DiffStats added={added} removed={removed} />
      </div>
    </header>
  );
}
