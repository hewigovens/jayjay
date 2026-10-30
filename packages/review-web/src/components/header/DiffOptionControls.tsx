/** @jsxRuntime automatic */
import type { Labels } from "../../model/diff";
import type { DiffOptions } from "../../model/review";
import { AllLinesIcon, SplitIcon, UnifiedIcon, WhitespaceIcon } from "../icons";

export function DiffOptionControls({ labels, options, configure }: { labels: Labels; options: DiffOptions; configure: (next: DiffOptions) => void }) {
  return <>
    {/* Only a note recorded with whitespace ignored turns this on. */}
    {options.ignoreWhitespace && <label className="jj-header-action jj-active"><input type="checkbox" checked
      onChange={() => configure({ ...options, ignoreWhitespace: false })} /><WhitespaceIcon /><span className="jj-label">{labels.ignoreWhitespace}</span></label>}
    <label className={`jj-header-action ${options.showAll ? "jj-active" : ""}`}><input type="checkbox" checked={options.showAll}
      onChange={event => configure({ ...options, showAll: event.target.checked })} /><AllLinesIcon /><span className="jj-label">{labels.allLines}</span></label>
    <button className="jj-header-action" title={options.split ? labels.switchToUnified : labels.switchToSideBySide}
      onClick={() => configure({ ...options, split: !options.split })}>{options.split ? <SplitIcon /> : <UnifiedIcon />}<span className="jj-label">{options.split ? labels.sideBySide : labels.unified}</span></button>
  </>;
}
