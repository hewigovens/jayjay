/** @jsxRuntime automatic */
import { useMemo, useState } from "react";
import type { Labels } from "../../model/diff";
import { ChevronIcon, CloseIcon, FolderIcon, HideReviewedIcon, ListIcon, SearchIcon, TreeIcon } from "../icons";
import { MiddlePath } from "./MiddlePath";
import { ReviewToggle } from "./ReviewToggle";
import { SetiIcon } from "./SetiIcon";
import { toggled } from "../toggled";
import type { FileSummary, Overview } from "../../model/review";

function FileRow({ file, index, name, depth, selected, disabled, labels, select, toggle, mark }: {
  file: FileSummary; index: number; name: string; depth: number; selected: number; disabled: boolean;
  labels: Labels; select: (index: number) => void; toggle: (index: number) => void; mark: (index: number, row: HTMLElement) => void;
}) {
  const kind = file.kind ?? "modified";
  const shownPath = file.oldPath ? `${file.oldPath} → ${file.path}` : file.path;
  return <div className={`jj-file-row jj-${file.review} ${selected === index ? "jj-selected" : ""}`} style={{ paddingLeft: 8 + depth * 12 }}>
    <ReviewToggle file={file} labels={labels} disabled={disabled} toggle={() => toggle(index)} />
    {/* Safari leaves a clicked button unfocused, and Space needs it focused. */}
    <button className="jj-file-select" data-file={index} aria-current={selected === index ? "true" : undefined} aria-label={file.path} aria-description={labels[`kind.${kind}`]}
      aria-keyshortcuts="Space" title={`${shownPath} · ${labels[file.review]}`} onClick={event => { event.currentTarget.focus(); select(index); }}
      onKeyDown={event => { if (event.key !== " ") return; event.preventDefault(); if (!event.repeat && !disabled) mark(index, event.currentTarget); }}>
      <span className={`jj-kind-dot jj-${kind}`} title={labels[`kind.${kind}`]} />
      <SetiIcon icon={file.icon} />
      <span className="jj-file-text">
        <span className="jj-file-name">{name}
          {file.noteCount > 0 && <span className="jj-note-count" title={labels.notes}>{file.noteCount}</span>}
        </span>
        <span className="jj-file-path">{file.oldPath && <><MiddlePath path={file.oldPath} className="jj-old-path" /><span aria-hidden="true">→</span></>}<MiddlePath path={file.path} /></span>
      </span>
    </button>
  </div>;
}

export function FileColumn({ overview, selected, disabled, labels, select, toggle, notes = true }: {
  overview: Overview; selected: number; disabled: boolean; labels: Labels;
  select: (index: number) => void; toggle: (index: number) => void;
  notes?: boolean;
}) {
  const [tree, setTree] = useState(true);
  const [searching, setSearching] = useState(false);
  const [filter, setFilter] = useState("");
  const [hideReviewed, setHideReviewed] = useState(false);
  const [onlyNotes, setOnlyNotes] = useState(false);
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const visible = useMemo(() => new Set(overview.files.flatMap((file, index) =>
    file.path.toLocaleLowerCase().includes(filter.toLocaleLowerCase()) && (!hideReviewed || file.review !== "reviewed") && (!onlyNotes || file.noteCount > 0) ? [index] : []
  )), [overview.files, filter, hideReviewed, onlyNotes]);
  const activeNotes = overview.notes.filter(note => !note.note.resolved).length;
  const reviewed = overview.files.some(file => file.review === "reviewed");
  const { counts } = overview;
  const countLabel = filter ? counts.filtered[visible.size] : reviewed ? counts.reviewed : counts.files;

  function folder(path: string) {
    setCollapsed(previous => toggled(previous, path));
  }
  function closeSearch() { setSearching(false); setFilter(""); }
  function mark(index: number, row: HTMLElement) {
    // Hiding reviewed files unmounts this row, which would drop focus to the page.
    if (hideReviewed && overview.files[index].review !== "reviewed") {
      const list = row.closest<HTMLElement>(".jj-file-rows");
      const rows = [...list?.querySelectorAll<HTMLElement>(".jj-file-select") ?? []];
      const at = rows.indexOf(row);
      (rows[at + 1] ?? rows[at - 1] ?? list)?.focus();
    }
    toggle(index);
  }

  return <aside className="jj-file-column" aria-label={labels.files}>
    <div className="jj-pane-heading jj-file-tools">
      {notes && (activeNotes > 0 || onlyNotes) && <button className="jj-tool jj-notes-filter" aria-pressed={onlyNotes} title={labels.onlyNotes} aria-label={labels.onlyNotes}
        onClick={() => setOnlyNotes(!onlyNotes)}>{activeNotes}</button>}
      {(reviewed || hideReviewed) && <button className="jj-tool" aria-pressed={hideReviewed} aria-label={labels.hideReviewedFiles}
        title={hideReviewed ? labels.showingUnreviewed : labels.hideReviewedFiles} onClick={() => setHideReviewed(!hideReviewed)}><HideReviewedIcon /></button>}
      <button className="jj-tool" aria-pressed={tree} aria-label={tree ? labels.showingTree : labels.showingFlat}
        title={tree ? labels.showingTree : labels.showingFlat} onClick={() => setTree(!tree)}>{tree ? <TreeIcon /> : <ListIcon />}</button>
      <button className="jj-tool" aria-pressed={searching} aria-label={labels.filterFiles} title={labels.filterFiles}
        onClick={() => searching ? closeSearch() : setSearching(true)}><SearchIcon /></button>
      <span className="jj-file-count" title={counts.reviewed}>{countLabel}</span>
    </div>
    {searching && <div className="jj-file-filter">
      <input type="search" autoFocus aria-label={labels.filterFiles} placeholder={labels.filterFiles} value={filter}
        onChange={event => setFilter(event.target.value)} onKeyDown={event => { if (event.key === "Escape") closeSearch(); }} />
      <button className="jj-tool" aria-label={labels.cancel} title={labels.cancel} onClick={closeSearch}><CloseIcon /></button>
    </div>}
    <div className="jj-file-rows" tabIndex={-1}>
      {visible.size === 0 && <p className="jj-notice">{filter ? labels.noMatchingFiles : overview.files.length ? labels.allFilesHidden : labels.noChangedFiles}</p>}
      {tree ? overview.tree.map(entry => {
        if ([...collapsed].some(path => entry.path.startsWith(`${path}/`))) return null;
        if (entry.file === null) {
          if (!overview.files.some((file, index) => visible.has(index) && file.path.startsWith(`${entry.path}/`))) return null;
          const open = !collapsed.has(entry.path);
          return <button className="jj-folder-row" key={`dir:${entry.path}`} style={{ paddingLeft: 10 + entry.depth * 12 }}
            aria-expanded={open} onClick={() => folder(entry.path)}><ChevronIcon open={open} /><FolderIcon /><span>{entry.name}</span></button>;
        }
        return visible.has(entry.file) ? <FileRow key={entry.path} file={overview.files[entry.file]} index={entry.file} name={entry.name} depth={entry.depth}
          selected={selected} disabled={disabled} labels={labels} select={select} toggle={toggle} mark={mark} /> : null;
      }) : overview.files.map((file, index) => visible.has(index) ? <FileRow key={file.path} file={file} index={index} name={file.path.split("/").at(-1) ?? file.path} depth={0}
        selected={selected} disabled={disabled} labels={labels} select={select} toggle={toggle} mark={mark} /> : null)}
    </div>
  </aside>;
}
