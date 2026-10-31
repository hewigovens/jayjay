/** @jsxRuntime automatic */
import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import type { DiffAnchor, Labels } from "../model/diff";
import { isImagePath, type FileSummary, type ImageSide, type ReviewNote, type ReviewResult } from "../model/review";
import { useCompact } from "../hooks/useCompact";
import type { ReviewController } from "../hooks/useReview";
import { DiffPane, lineLabel } from "./diff/DiffPane";
import { ImageDiff } from "./diff/ImageDiff";
import { FileColumn } from "./files/FileColumn";
import { ChangeHeader } from "./header/ChangeHeader";
import { DiffOptionControls } from "./header/DiffOptionControls";
import { FileHeader } from "./header/FileHeader";
import { NoteIcon } from "./icons";
import { LazySection } from "./LazySection";
import { anchorKey, NoteBubble, NoteComposer, NotesBanner, type NoteTarget } from "./notes/InlineNotes";
import { toggled } from "./toggled";

export function ReviewWorkspace({ review, labels, savedLabel, headerActions, footer, onRetry, notes = true, avatars = false }: {
  review: ReviewController;
  labels: Labels;
  savedLabel: string;
  headerActions?: ReactNode;
  footer?: ReactNode;
  onRetry?: () => void;
  /** Review notes in the diff, their per-line buttons, and the banner of notes the diff cannot place. */
  notes?: boolean;
  /** Looks up the author's avatar on Gravatar, GitHub, or GitLab, which shares their email hash or username with that service. */
  avatars?: boolean;
}) {
  const { result, selected, options, pending, error, revision, select, configure, mutate, renderFile, image } = review;
  const compact = useCompact();
  const [target, setTarget] = useState<NoteTarget | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const [unfolded, setUnfolded] = useState<Set<string>>(() => new Set());
  const [loaded, setLoaded] = useState<Map<number, ReviewResult>>(() => new Map());
  const loading = useRef(new Set<number>());
  const [failed, setFailed] = useState<Set<number>>(() => new Set());
  const jumpTarget = useRef<ReviewNote | null>(null);
  const foldedBefore = useRef<{ session?: number; paths: string[] }>({ paths: [] });
  const detail = useRef<HTMLElement>(null);
  const columns = useRef<HTMLDivElement>(null);
  const file = result?.overview.files[result.file];
  const path = file?.path;
  const cacheKey = result ? `${result.session}|${revision}|${JSON.stringify(result.options)}` : "";
  const liveKey = useRef(cacheKey);
  // The newest result the review itself produced for each file; a lazy render that started before it is stale.
  const reviewed = useRef(new Map<number, ReviewResult>());

  useEffect(() => { jumpTarget.current = null; setUnfolded(new Set()); }, [result?.session]);
  useEffect(() => { setTarget(null); setExpanded(new Set()); }, [result?.session, compact ? undefined : path]);
  useEffect(() => { liveKey.current = cacheKey; reviewed.current.clear(); loading.current.clear(); setLoaded(new Map()); setFailed(new Set()); }, [cacheKey]);
  useEffect(() => {
    if (!result) return;
    reviewed.current.set(result.file, result);
    setLoaded(previous => new Map(previous).set(result.file, result));
  }, [result]);

  // Every rendered file's diff, keyed by file index: the selected one on wide screens, all loaded sections in the compact list.
  const diffs = compact ? loaded : new Map(result ? [[result.file, result]] : []);

  useEffect(() => {
    if (pending || !jumpTarget.current || !result) return;
    const note = jumpTarget.current.note;
    const section = detail.current?.querySelector<HTMLElement>(`[data-path="${CSS.escape(note.path)}"]`);
    const node = section?.querySelector<HTMLElement>(`.jj-diff-cell[data-side="${note.side}"][data-line="${note.line}"]`);
    // In the compact list the section may still be loading, so keep the jump until its lines render.
    if (!node) { section?.scrollIntoView({ block: "start" }); if (!compact) jumpTarget.current = null; return; }
    node.scrollIntoView({ block: "center" }); node.focus({ preventScroll: true });
    jumpTarget.current = null;
  }, [pending, result, loaded, compact]);

  useEffect(() => {
    const files = result?.overview.files ?? [];
    setUnfolded(previous => {
      const kept = [...previous].filter(path => files.some(entry => entry.path === path && entry.review === "reviewed"));
      return kept.length === previous.size ? previous : new Set(kept);
    });
  }, [result]);

  function load(index: number) {
    if (loading.current.has(index) || !result) return;
    loading.current.add(index);
    setFailed(previous => previous.has(index) ? toggled(previous, index) : previous);
    const key = cacheKey;
    const latest = reviewed.current.get(index);
    renderFile(index).then(next => {
      if (key !== liveKey.current) return;
      loading.current.delete(index);
      if (reviewed.current.get(index) !== latest) return;
      if (`${next.session}|${revision}|${JSON.stringify(next.options)}` === key) setLoaded(previous => new Map(previous).set(index, next));
    }, () => {
      if (key !== liveKey.current) return;
      loading.current.delete(index);
      setFailed(previous => toggled(previous, index));
    });
  }

  async function saveNote(body: string) {
    if (!target) return false;
    const saved = await mutate({ kind: "addNote", file: target.file, side: target.anchor.side, line: target.anchor.line, body, ignoreWhitespace: target.ignoreWhitespace }, target.file);
    if (saved) setTarget(null);
    return saved;
  }

  function locate(item: ReviewNote) {
    if (!result) return;
    const index = result.overview.files.findIndex(entry => entry.path === item.note.path);
    if (index < 0) return;
    const entry = result.overview.files[index];
    if (entry.review === "reviewed") setUnfolded(previous => new Set(previous).add(entry.path));
    if (configure({ ...options, ignoreWhitespace: item.note.ignore_whitespace, showAll: options.showAll || (item.status === "current" && !folded(entry)) }, index)) jumpTarget.current = item;
  }

  const disabled = pending || Boolean(error);
  // A note draft keeps its file open, since folding would discard the text.
  const folded = (entry: FileSummary) => entry.review === "reviewed" && !unfolded.has(entry.path) && target?.path !== entry.path;
  const foldedPaths = result?.overview.files.filter(folded).map(entry => entry.path) ?? [];

  // Folding unmounts the diff, which may hold focus, and leaves a long file's reader past its end.
  useLayoutEffect(() => {
    const { session, paths } = foldedBefore.current;
    foldedBefore.current = { session: result?.session, paths: foldedPaths };
    const fresh = foldedPaths.filter(path => !paths.includes(path));
    if (session !== result?.session || fresh.length !== 1) return;
    const diff = detail.current?.querySelector(`[data-path="${CSS.escape(fresh[0])}"]`);
    if (!diff) return;
    const section = diff.closest(".jj-file-section");
    if (section && section.getBoundingClientRect().top < 0) section.scrollIntoView({ block: "start" });
    if (document.activeElement === document.body) (section ?? detail.current)?.querySelector<HTMLElement>(".jj-file-heading input")?.focus({ preventScroll: true });
  }, [result?.session, foldedPaths.join("\0")]);
  const resolve = (id: string) => void mutate({ kind: "resolveNote", id });
  // The compact list re-renders the marked file's section; the wide layout re-renders the open file.
  async function toggleFile(index: number) {
    if (!result) return;
    const { files } = result.overview;
    const marking = files[index].review !== "reviewed";
    // The rendered rows give display order; the compact list has none and stays put.
    const order = [...columns.current?.querySelectorAll<HTMLButtonElement>(".jj-file-select") ?? []].map(row => Number(row.dataset.file));
    if (!await mutate({ kind: "setFile", file: index, reviewed: marking }, compact ? index : undefined) || !marking || index !== selected || target) return;
    const position = order.indexOf(index);
    const next = [...order.slice(position + 1), ...order.slice(0, Math.max(position, 0))].find(file => files[file].review !== "reviewed");
    columns.current?.querySelector<HTMLButtonElement>(`.jj-file-select[data-file="${next}"]`)?.click();
  }

  // Current notes sit under their line and resolved ones behind a gutter marker; the banner takes stale and orphaned notes, and current ones whose line is folded away.
  const placed = new Map<string, ReviewNote[]>();
  const rendered = new Set<string>();
  for (const [index, data] of diffs) {
    const filePath = data.overview.files[index]?.path;
    const entry = result?.overview.files[index];
    if (entry && folded(entry)) continue;
    for (const row of data.diff?.rows ?? []) for (const cell of row.cells) if (cell.anchor && filePath) rendered.add(anchorKey(filePath, cell.anchor.side, cell.anchor.line));
  }
  const renderedPaths = new Set([...diffs.keys()].map(index => result?.overview.files[index]?.path));
  // A folded file never loads in the compact list, and the banner is the only place its notes can show.
  if (compact) for (const path of foldedPaths) renderedPaths.add(path);
  const unplaced: ReviewNote[] = [];
  for (const item of notes && result ? result.overview.notes : []) {
    const key = anchorKey(item.note.path, item.note.side, item.note.line);
    if (item.status === "stale" || item.status === "orphaned") unplaced.push(item);
    else if (!renderedPaths.has(item.note.path)) continue;
    else if (rendered.has(key)) placed.set(key, [...placed.get(key) ?? [], item]);
    else if (item.status === "current") unplaced.push(item);
  }

  function diffPane(index: number, data: ReviewResult) {
    const filePath = data.overview.files[index].path;
    const annotation = (anchor: DiffAnchor) => {
      const key = anchorKey(filePath, anchor.side, anchor.line);
      const shown = (placed.get(key) ?? []).filter(item => item.status === "current" || expanded.has(key));
      const composing = target !== null && anchorKey(target.path, target.anchor.side, target.anchor.line) === key;
      if (!shown.length && !composing) return null;
      return <>
        {shown.map(item => <NoteBubble key={item.note.id} item={item} labels={labels} disabled={disabled} resolve={resolve} />)}
        {composing && <NoteComposer target={target} labels={labels} disabled={disabled} save={saveNote} cancel={() => setTarget(null)} />}
      </>;
    };
    const marker = (anchor: DiffAnchor) => {
      const key = anchorKey(filePath, anchor.side, anchor.line);
      const resolved = (placed.get(key) ?? []).filter(item => item.status === "resolved");
      if (!resolved.length) return null;
      const open = expanded.has(key);
      return <button className="jj-note-marker jj-resolved" aria-expanded={open} aria-label={`${labels.resolved}: ${lineLabel(labels, anchor.side, anchor.line)}`}
        title={labels.resolved} onClick={() => setExpanded(previous => toggled(previous, key))}>
        <NoteIcon />{resolved.length > 1 && resolved.length}</button>;
    };
    const note = (anchor: DiffAnchor) => setTarget({ file: index, path: filePath, anchor, ignoreWhitespace: data.options.ignoreWhitespace });
    const diff = data.diff && <DiffPane labels={labels} view={data.diff} split={data.options.split} showAll={() => configure({ ...options, showAll: true })}
      review={{ states: data.groupStates, disabled, toggle: group => void mutate({ kind: "setGroup", file: index, group, reviewed: data.groupStates[group] !== "reviewed", ignoreWhitespace: data.options.ignoreWhitespace }, index),
        ...notes && { note, annotation, marker } }} />;
    // Notes anchor to source lines, so a file with notes keeps its source under the preview.
    if (image && isImagePath(filePath)) return <ImageDiff key={`${data.session}|${filePath}`} kind={data.overview.files[index].kind} labels={labels}
      keepSource={notes && (target?.path === filePath || Boolean(result?.overview.notes.some(item => item.note.path === filePath)))} load={(side: ImageSide, signal: AbortSignal) => image(index, side, signal)}>{diff}</ImageDiff>;
    return diff ?? <p className="jj-empty-diff">{labels.noFiles}</p>;
  }

  return <div className={`jayjay-review jj-review-workspace ${compact ? "jj-compact" : ""}`} aria-busy={pending}>
    {error && <div className="jj-review-error" role="alert">{labels.renderError} {error === "worker-stopped" ? labels.workerError : error}
      <button onClick={onRetry ?? review.retry}>{labels.retry}</button>{!result && headerActions}</div>}
    {!result && !error && <p className="jj-notice">{labels.loading}</p>}
    {result && <div className="jj-review-columns" ref={columns}>
      {!compact && <FileColumn overview={result.overview} selected={selected} disabled={disabled} labels={labels} select={select} notes={notes} toggle={toggleFile} />}
      <section className="jj-review-detail" aria-label={labels.result} aria-busy={pending} ref={detail}>
        <ChangeHeader overview={result.overview} labels={labels} actions={headerActions} avatars={avatars} />
        {compact && <div className="jj-pane-heading jj-diff-options"><div className="jj-header-actions"><DiffOptionControls labels={labels} options={options} configure={configure} /></div></div>}
        {!compact && file && <FileHeader file={file} labels={labels} disabled={disabled} toggleReviewed={() => toggleFile(result.file)}
          controls={<DiffOptionControls labels={labels} options={options} configure={configure} />} />}
        {result.syntaxUnavailable && <p className="jj-notice">{labels.syntaxUnavailable} <button onClick={() => select(selected)}>{labels.retry}</button></p>}
        {!compact && result.diff?.whitespaceOnlyHidden && <p className="jj-notice">{labels.whitespaceOnly}</p>}
        <NotesBanner items={unplaced} labels={labels} disabled={disabled} resolve={resolve} locate={locate} />
        {compact
          ? <div className="jj-file-sections">
            {result.overview.files.map((entry, index) => <LazySection key={entry.path} index={index} version={cacheKey} loaded={loaded.has(index) || folded(entry)} load={load}>
              <FileHeader file={entry} labels={labels} disabled={disabled} toggleReviewed={() => toggleFile(index)} />
              <div className="jj-diff-scroll" data-path={entry.path} aria-label={`${labels.lines}: ${entry.path}`}>
                {folded(entry) ? null
                  : loaded.get(index) ? diffPane(index, loaded.get(index)!)
                  : failed.has(index) ? <p className="jj-notice">{labels.renderError} <button onClick={() => load(index)}>{labels.retry}</button></p>
                  : <p className="jj-notice">{labels.loadingFile}</p>}
              </div>
            </LazySection>)}
            {!result.overview.files.length && <p className="jj-empty-diff">{labels.noChangedFiles}</p>}
          </div>
          : <div className="jj-diff-scroll" data-path={path} tabIndex={0} aria-label={labels.lines}>
            {!file ? <p className="jj-empty-diff">{labels.noChangedFiles}</p>
              : folded(file) ? <p className="jj-empty-diff">{labels.reviewed}</p>
              : diffPane(result.file, result)}
          </div>}
        <div className="jj-result-footer" aria-live="polite"><span>{pending ? labels.loadingFile : error ? labels.engineError : savedLabel}</span><span>{result.pack} · {result.elapsed.toFixed(1)} ms</span></div>
      </section>
    </div>}
    {footer}
  </div>;
}
