/** @jsxRuntime automatic */
import { useState, type ReactNode } from "react";
import type { Labels } from "../../model/diff";
import type { Overview } from "../../model/review";
import { Avatar } from "../Avatar";
import { ExpandIcon } from "../icons";
import { DiffStats } from "./DiffStats";

// Joins terminal-wrapped commit message lines into paragraphs, keeping list items, indented lines and blank lines.
function reflow(text: string): string {
  return text.split(/\n\s*\n/).map(paragraph => paragraph.split("\n").reduce((out, line) => {
    const keep = /^(\s|[-*+] |\d+[.)] )/.test(line);
    return out === "" ? line : keep ? `${out}\n${line}` : `${out} ${line.trim()}`;
  }, "")).join("\n\n");
}

export function ChangeHeader({ overview, labels, actions, avatars }: { overview: Overview; labels: Labels; actions?: ReactNode; avatars: boolean }) {
  const [expanded, setExpanded] = useState(false);
  const added = overview.files.reduce((sum, file) => sum + file.added, 0);
  const removed = overview.files.reduce((sum, file) => sum + file.removed, 0);
  const prefix = Math.min(overview.changeIdPrefix ?? 2, overview.changeId.length);
  const date = overview.timestampMillis
    ? new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(overview.timestampMillis)
    : null;
  const toggleLabel = expanded ? labels.collapseDescription : labels.expandDescription;
  return <header className="change-header">
    <div className="change-title">
      <h1>{overview.title}</h1>
      {overview.description && <button className="tool description-toggle" aria-expanded={expanded} aria-label={toggleLabel} title={toggleLabel}
        onClick={() => setExpanded(!expanded)}><ExpandIcon expanded={expanded} /></button>}
      {actions && <div className="change-actions">{actions}</div>}
    </div>
    {overview.description && <p className={`change-description ${expanded ? "expanded" : ""}`}>{reflow(overview.description)}</p>}
    <div className="byline">
      {overview.author && <><Avatar source={avatars ? overview.authorAvatar : null} name={overview.author} />
        <span className="author">{overview.author}</span><span className="separator" aria-hidden="true">·</span></>}
      <code className="change-id" title={overview.changeId}><b>{overview.changeId.slice(0, prefix)}</b>{overview.changeId.slice(prefix)}</code>
      {date && <><span className="separator" aria-hidden="true">·</span><time>{date}</time></>}
      <span className="spacer" />
      <DiffStats added={added} removed={removed} />
    </div>
  </header>;
}
