/** @jsxRuntime automatic */
import type { ReactNode } from "react";
import type { Labels } from "../../model/diff";
import type { FileSummary } from "../../model/review";
import { MiddlePath } from "../files/MiddlePath";
import { ReviewToggle } from "../files/ReviewToggle";
import { CheckIcon, FileIcon } from "../icons";
import { CopyButton } from "./CopyButton";
import { DiffStats } from "./DiffStats";

export function FileHeader({ file, labels, disabled, toggleReviewed, controls }: {
  file: FileSummary; labels: Labels; disabled: boolean; toggleReviewed: () => void; controls?: ReactNode;
}) {
  const kind = file.kind ?? "modified";
  return <div className="jj-pane-heading jj-file-heading">
    <div className="jj-file-title">
      <FileIcon />
      {file.oldPath && <><MiddlePath path={file.oldPath} className="jj-old-path" /><span aria-hidden="true">→</span></>}
      <h2><MiddlePath path={file.path} /></h2>
      <CopyButton value={file.path} label={labels.copyPath} />
      <span className={`jj-kind-pill jj-${kind}`}>{labels[`kind.${kind}`]}</span>
      <DiffStats added={file.added} removed={file.removed} />
    </div>
    <div className="jj-header-actions">
      {controls
        ? <><label className={`jj-header-action ${file.review === "reviewed" ? "jj-active" : ""}`}><input type="checkbox" checked={file.review === "reviewed"} disabled={disabled}
          aria-label={labels.markReviewed} onChange={toggleReviewed} /><CheckIcon /><span className="jj-label">{labels.markReviewed}</span></label>{controls}</>
        : <ReviewToggle file={file} labels={labels} disabled={disabled} toggle={toggleReviewed} />}
    </div>
  </div>;
}
