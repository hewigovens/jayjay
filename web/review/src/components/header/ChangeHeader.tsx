/** @jsxRuntime automatic */
import { useState, type ReactNode } from "react";
import type { Labels } from "../../model/diff";
import type { Overview } from "../../model/review";
import { Avatar } from "../Avatar";
import { ChevronIcon } from "../icons";
import { ToolButton } from "../ToolButton";
import { DiffStats } from "./DiffStats";

// Joins terminal-wrapped commit message lines into paragraphs, keeping list items, indented lines and blank lines.
function reflow(text: string): string {
  return text
    .split(/\n\s*\n/)
    .map((paragraph) =>
      paragraph.split("\n").reduce((out, line) => {
        const keep = /^(\s|[-*+] |\d+[.)] )/.test(line);
        return out === "" ? line : keep ? `${out}\n${line}` : `${out} ${line.trim()}`;
      }, ""),
    )
    .join("\n\n");
}

export function ChangeHeader({
  overview,
  labels,
  hideChangeId = false,
  titleReference,
  actions,
  controls,
  avatars,
  descriptionExpanded,
}: {
  overview: Overview;
  labels: Labels;
  hideChangeId?: boolean;
  titleReference?: string;
  actions?: ReactNode;
  /** Shown in the byline before the line counts. */
  controls?: ReactNode;
  avatars: boolean;
  descriptionExpanded: boolean;
}) {
  const [expanded, setExpanded] = useState(descriptionExpanded);
  const added = overview.files.reduce((sum, file) => sum + file.added, 0);
  const removed = overview.files.reduce((sum, file) => sum + file.removed, 0);
  const prefix = Math.min(overview.changeIdPrefix ?? 2, overview.changeId.length);
  const date = overview.timestampMillis
    ? new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
        overview.timestampMillis,
      )
    : null;
  const referenced = titleReference && overview.title.endsWith(` ${titleReference}`);
  const toggleLabel = expanded ? labels.collapseDescription : labels.expandDescription;
  return (
    <header className="jj-change-header">
      <div className="jj-change-title">
        <h1>
          {referenced ? (
            <>
              {overview.title.slice(0, -titleReference.length - 1)}{" "}
              <span className="jj-title-reference">{titleReference}</span>
            </>
          ) : (
            overview.title
          )}
        </h1>
        {overview.description && (
          <ToolButton
            className="jj-description-toggle"
            aria-expanded={expanded}
            label={toggleLabel}
            onClick={() => setExpanded(!expanded)}
          >
            <ChevronIcon open={expanded} />
          </ToolButton>
        )}
        {actions && <div className="jj-change-actions">{actions}</div>}
      </div>
      {overview.description &&
        (overview.descriptionHtml ? (
          <div
            className={`jj-change-description jj-markdown ${expanded ? "jj-expanded" : ""}`}
            dangerouslySetInnerHTML={{ __html: overview.descriptionHtml }}
          />
        ) : (
          <p className={`jj-change-description ${expanded ? "jj-expanded" : ""}`}>
            {reflow(overview.description)}
          </p>
        ))}
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
        {controls && <div className="jj-header-actions">{controls}</div>}
        <DiffStats added={added} removed={removed} />
      </div>
    </header>
  );
}
