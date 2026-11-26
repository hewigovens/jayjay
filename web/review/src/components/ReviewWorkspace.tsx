/** @jsxRuntime automatic */
import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
import { diffThemeAttributes, type DiffThemeSelection } from "../themes";
import type { DiffAnchor, Labels } from "../model/diff";
import { isImagePath, type ImageSide, type ReviewNote, type ReviewResult } from "../model/review";
import { useCompact } from "../hooks/useCompact";
import { useFileShortcuts } from "../hooks/useFileShortcuts";
import { useFilesWidth } from "../hooks/useFilesWidth";
import { useFolding } from "../hooks/useFolding";
import { useLoadedSections } from "../hooks/useLoadedSections";
import { useNoteJump } from "../hooks/useNoteJump";
import { useOverdue } from "../hooks/useOverdue";
import type { ReviewController } from "../hooks/useReview";
import { useSectionJump } from "../hooks/useSectionJump";
import { DiffPane, lineLabel } from "./diff/DiffPane";
import { ImageDiff } from "./diff/ImageDiff";
import { Dialog } from "./Dialog";
import { ColumnResizer } from "./files/ColumnResizer";
import { FileColumn, noFileFilter } from "./files/FileColumn";
import { ShortcutsDialog } from "./files/ShortcutsDialog";
import { ChangeHeader } from "./header/ChangeHeader";
import { DiffOptionControls } from "./header/DiffOptionControls";
import { FileHeader } from "./header/FileHeader";
import { CollapseAllIcon, ExpandAllIcon, ListIcon, NoteIcon, SidebarIcon } from "./icons";
import { LazySection } from "./LazySection";
import { NoteBubble } from "./notes/NoteBubble";
import { NoteComposer, type NoteTarget } from "./notes/NoteComposer";
import { NotesBanner } from "./notes/NotesBanner";
import { anchorKey, placeNotes } from "./notes/notePlacement";
import { toggled } from "./toggled";
import { ToolButton } from "./ToolButton";

export function ReviewWorkspace({
  review,
  labels,
  savedLabel,
  brand,
  hideChangeId = false,
  titleReference,
  onShowShortcuts,
  headerActions,
  errorActions,
  footer,
  onRetry,
  notes = true,
  avatars = false,
  allLines = true,
  fill = false,
  statusBar = true,
  descriptionExpanded = false,
  reviewMode = true,
  onReviewMode,
  stacked = false,
  onStacked,
  filesSlot,
  filesHeader,
  diffThemes,
}: {
  review: ReviewController;
  labels: Labels;
  savedLabel?: string;
  brand?: ReactNode;
  hideChangeId?: boolean;
  /** Shown muted after the title when the title ends with it, such as a pull request's `#123`. */
  titleReference?: string;
  onShowShortcuts?: () => void;
  headerActions?: ReactNode;
  errorActions?: ReactNode;
  footer?: ReactNode;
  /** Replaces the error's Retry, or hides it when null. */
  onRetry?: (() => void) | null;
  /** Review notes in the diff, their per-line buttons, and the banner of notes the diff cannot place. */
  notes?: boolean;
  /** Looks up the author's avatar on Gravatar, GitHub, or GitLab, which shares their email hash or username with that service. */
  avatars?: boolean;
  /** The All lines option; turn it off when files hold only the lines around their changes. */
  allLines?: boolean;
  /** Fills the host's height edge to edge instead of sitting in a bordered card; the phone-width list still scrolls with the page. */
  fill?: boolean;
  /** The bar under the diff with the saved, loading, or failed state and the render time. */
  statusBar?: boolean;
  /** Starts with the description open; it is collapsed to two lines otherwise. */
  descriptionExpanded?: boolean;
  /** Off hides review marks, Mark reviewed, Space marking, and reviewed-file filtering; notes stay. */
  reviewMode?: boolean;
  /** Adds a review mode toggle to the file list's tools. */
  onReviewMode?: () => void;
  /** Shows every file's diff in one scrolling list beside the file list, as the phone-width layout always does. */
  stacked?: boolean;
  /** Adds a one-list toggle to the file list's tools. */
  onStacked?: () => void;
  /** An element in the host's toolbar that takes the phone list's Files button; the file count stays above the list. */
  filesSlot?: HTMLElement | null;
  /** Shown above the file list: at the top of the file column, or above the phone list's file count. */
  filesHeader?: ReactNode;
  /** Diff-only palettes with an optional appearance override. Omit to use the host CSS colors. */
  diffThemes?: DiffThemeSelection;
}) {
  const {
    result,
    selected,
    options,
    pending,
    error,
    revision,
    select,
    configure,
    mutate,
    renderFile,
    image,
  } = review;
  const compact = useCompact();
  // A file picked from the list shows alone with the one-list setting still on, as in the desktop app.
  const [picked, setPicked] = useState(false);
  const list = compact || (stacked && !picked);
  const overdue = useOverdue(pending);
  const [target, setTarget] = useState<NoteTarget | null>(null);
  // The draft's text, which outlives its composer when switching layouts remounts the diff.
  const draftBody = useRef("");
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const detail = useRef<HTMLElement>(null);
  const sectionList = useRef<HTMLDivElement>(null);
  const columns = useRef<HTMLDivElement>(null);
  const diffScroll = useRef<HTMLDivElement>(null);
  const filesWidth = useFilesWidth();
  const file = result?.overview.files[result.file];
  const path = file?.path;
  const { cacheKey, loaded, failed, load } = useLoadedSections(result, revision, renderFile);
  const jump = useSectionJump(detail, loaded);
  const [filter, setFilter] = useState(noFileFilter);
  const [shortcuts, setShortcuts] = useState(false);
  const [drawer, setDrawer] = useState(false);
  const [drawerFile, setDrawerFile] = useState(0);
  const drawerPick = useRef<number | null>(null);
  const showShortcuts = onShowShortcuts ?? (() => setShortcuts(true));
  function sections() {
    return [...(detail.current?.querySelectorAll<HTMLElement>(".jj-file-section") ?? [])];
  }
  // Below the host's pinned bar, or the top of the wide list's scroller, and above the bottom of the window.
  function onScreen(row: HTMLElement) {
    const { top, bottom } = row.getBoundingClientRect();
    const floor = compact
      ? parseFloat(getComputedStyle(row).scrollMarginTop)
      : (sectionList.current?.getBoundingClientRect().top ?? 0);
    return bottom > floor + 1 && top < innerHeight;
  }
  const topSection = (rows: HTMLElement[]) => rows.findIndex(onScreen);
  function openDrawer() {
    const rows = sections();
    setDrawerFile(Number(rows[Math.max(topSection(rows), 0)]?.dataset.file ?? 0));
    setDrawer(true);
  }

  // Coming back to the one list keeps the reader on the file they had open.
  const listBefore = useRef(list);
  useLayoutEffect(() => {
    if (list === listBefore.current) return;
    listBefore.current = list;
    if (list && !compact) jump(selected);
  }, [list]);

  function toggleStacked() {
    if (picked) return setPicked(false);
    // Leaving the one list opens the file being read.
    const rows = sections();
    const top = list ? Number(rows[topSection(rows)]?.dataset.file ?? selected) : selected;
    if (top !== selected) select(top);
    onStacked?.();
  }

  const [drawerLayout, setDrawerLayout] = useState(compact);
  if (drawerLayout !== compact) {
    setDrawerLayout(compact);
    setDrawer(false);
  }

  // In the one list, keys walk the diff sections unless focus is in the file list, which keeps its own order.
  const walksSections = () =>
    list && !columns.current?.querySelector(".jj-file-rows")?.contains(document.activeElement);
  useFileShortcuts({
    rows: () =>
      walksSections()
        ? sections()
        : [...(columns.current?.querySelectorAll<HTMLElement>(".jj-file-select") ?? [])],
    // Focus runs ahead of the selection while a pick renders, but a phone scroll can leave it off screen.
    current: (rows, focused) =>
      walksSections()
        ? focused >= 0 && onScreen(rows[focused])
          ? focused
          : topSection(rows)
        : focused >= 0
          ? focused
          : rows.findIndex((row) => Number(row.dataset.file) === selected),
    reviewed: (index) => reviewMode && result?.overview.files[index]?.review === "reviewed",
    go: (row) => {
      const file = Number(row.dataset.file);
      if (!row.matches(".jj-file-select")) return jump(file);
      row.focus();
      if (list) jump(file, false);
      else select(file);
    },
    list: () => (compact ? null : columns.current?.querySelector(".jj-file-rows")),
    search: () => {
      setFilter({ ...filter, searching: true });
      if (!compact) return;
      openDrawer();
      // The drawer opens in this event's commit; showModal focuses its first button, not the filter.
      requestAnimationFrame(() =>
        columns.current?.parentElement
          ?.querySelector<HTMLElement>(".jj-file-drawer .jj-file-filter input")
          ?.focus(),
      );
    },
    showShortcuts,
  });

  // Opening another file drops a note draft elsewhere; switching layouts on the same file keeps it.
  const pickedPath = result?.overview.files[selected]?.path;
  useEffect(() => {
    if (list) return;
    setTarget((current) => (current?.path === pickedPath ? current : null));
    setExpanded(new Set());
  }, [list ? undefined : pickedPath]);
  useLayoutEffect(() => {
    diffScroll.current?.scrollTo(0, 0);
  }, [path]);
  useLayoutEffect(() => {
    if (!result) return setTarget(null);
    setTarget((current) => {
      const index = current
        ? result.overview.files.findIndex((entry) => entry.path === current.path)
        : -1;
      return current && index >= 0 ? { ...current, file: index } : null;
    });
  }, [result]);

  const jumpTarget = useNoteJump(detail, result, pending, list, loaded);
  const { folded, unfold, toggleFold, foldAll } = useFolding(
    detail,
    result,
    list,
    reviewMode,
    target?.path,
  );

  async function saveNote(body: string) {
    if (!target) return false;
    const saved = await mutate(
      {
        kind: "addNote",
        file: target.file,
        side: target.anchor.side,
        line: target.anchor.line,
        body,
        ignoreWhitespace: target.ignoreWhitespace,
      },
      target.file,
    );
    if (saved) setTarget(null);
    return saved;
  }

  function locate(item: ReviewNote) {
    if (!result) return;
    const index = result.overview.files.findIndex((entry) => entry.path === item.note.path);
    if (index < 0) return;
    const entry = result.overview.files[index];
    if (folded(entry)) unfold(entry.path);
    if (
      configure(
        {
          ...options,
          ignoreWhitespace: item.note.ignore_whitespace,
          showAll: options.showAll || (allLines && item.status === "current" && !folded(entry)),
        },
        index,
      )
    )
      jumpTarget.current = item;
  }

  const disabled = pending || Boolean(error);
  const filesTotal =
    reviewMode && result?.overview.files.some((entry) => entry.review === "reviewed")
      ? result.overview.counts.reviewed
      : result?.overview.counts.files;
  const update = (id: string, body: string) => mutate({ kind: "updateNote", id, body });
  const resolve = (id: string) => void mutate({ kind: "resolveNote", id });
  // The one list re-renders the marked file's section, which folds; the single-file layout re-renders the open file and moves on.
  async function toggleFiles(indices: number[]) {
    if (!result || !indices.length) return;
    const { files } = result.overview;
    const marking = indices.some((index) => files[index].review !== "reviewed");
    const order = [
      ...(columns.current?.querySelectorAll<HTMLButtonElement>(".jj-file-select") ?? []),
    ].map((row) => Number(row.dataset.file));
    if (
      !(await mutate(
        { kind: "setFiles", files: indices, reviewed: marking },
        list ? indices[0] : undefined,
      )) ||
      list ||
      !marking ||
      !indices.includes(selected) ||
      target
    )
      return;
    const position = order.indexOf(selected);
    const next = [...order.slice(position + 1), ...order.slice(0, Math.max(position, 0))].find(
      (file) => files[file].review !== "reviewed" && !indices.includes(file),
    );
    columns.current
      ?.querySelector<HTMLButtonElement>(`.jj-file-select[data-file="${next}"]`)
      ?.click();
  }

  const diffs = list ? [...loaded.values()] : result ? [{ file: result.file, data: result }] : [];
  const { placed, rendered, unplaced } = placeNotes(
    diffs,
    result?.overview.files ?? [],
    notes && result ? result.overview.notes : [],
    folded,
  );

  const draftKey =
    target && (list ? loaded.get(target.path)?.key : target.path === path ? cacheKey : undefined);
  const draftLost =
    target !== null &&
    draftKey === cacheKey &&
    rendered.get(anchorKey(target.path, target.anchor.side, target.anchor.line)) !== target.text;
  useEffect(() => {
    if (draftLost) setTarget(null);
  }, [draftLost]);

  const diffOptions = (
    <DiffOptionControls
      labels={labels}
      options={options}
      configure={configure}
      allLines={allLines}
    />
  );
  const listControls = (
    <>
      <button className="jj-header-action" onClick={() => foldAll(false)}>
        <ExpandAllIcon />
        <span className="jj-label">{labels.expandAll}</span>
      </button>
      <button className="jj-header-action" onClick={() => foldAll(true)}>
        <CollapseAllIcon />
        <span className="jj-label">{labels.collapseAll}</span>
      </button>
      {diffOptions}
    </>
  );

  function diffPane(index: number, data: ReviewResult, stale = false) {
    const summary = data.overview.files[index];
    const filePath = summary.path;
    const locked = disabled || stale;
    const annotation = (anchor: DiffAnchor) => {
      const key = anchorKey(filePath, anchor.side, anchor.line);
      const shown = (placed.get(key) ?? []).filter(
        (item) => item.status === "current" || expanded.has(key),
      );
      const composing =
        target !== null && anchorKey(target.path, target.anchor.side, target.anchor.line) === key;
      if (!shown.length && !composing) return null;
      return (
        <>
          {shown.map((item) => (
            <NoteBubble
              key={item.note.id}
              item={item}
              labels={labels}
              disabled={locked}
              resolve={resolve}
              update={update}
            />
          ))}
          {composing && (
            <NoteComposer
              target={target}
              initialBody={draftBody.current}
              onBody={(body) => {
                draftBody.current = body;
              }}
              labels={labels}
              disabled={locked}
              save={saveNote}
              cancel={() => setTarget(null)}
            />
          )}
        </>
      );
    };
    const marker = (anchor: DiffAnchor) => {
      const key = anchorKey(filePath, anchor.side, anchor.line);
      const resolved = (placed.get(key) ?? []).filter((item) => item.status === "resolved");
      if (!resolved.length) return null;
      const open = expanded.has(key);
      return (
        <button
          className="jj-note-marker jj-resolved"
          aria-expanded={open}
          aria-label={`${labels.resolved}: ${lineLabel(labels, anchor.side, anchor.line)}`}
          title={labels.resolved}
          onClick={() => setExpanded((previous) => toggled(previous, key))}
        >
          <NoteIcon />
          {resolved.length > 1 && resolved.length}
        </button>
      );
    };
    const note = (anchor: DiffAnchor) => {
      draftBody.current = "";
      setTarget({
        file: index,
        path: filePath,
        anchor,
        ignoreWhitespace: data.options.ignoreWhitespace,
        text: rendered.get(anchorKey(filePath, anchor.side, anchor.line)),
      });
    };
    const diff = data.diff && (
      <DiffPane
        key={filePath}
        labels={labels}
        empty={
          summary.notice ??
          (summary.kind === "added" || summary.kind === "removed"
            ? labels.emptyFile
            : labels.noTextChanges)
        }
        view={data.diff}
        split={data.options.split}
        showAll={allLines ? () => configure({ ...options, showAll: true }) : undefined}
        review={{
          states: reviewMode ? data.groupStates : [],
          disabled: locked,
          toggle: !reviewMode
            ? undefined
            : (group) =>
                void mutate(
                  {
                    kind: "setGroup",
                    file: index,
                    group,
                    reviewed: data.groupStates[group] !== "reviewed",
                    ignoreWhitespace: data.options.ignoreWhitespace,
                  },
                  index,
                ),
          ...(notes && { note, annotation, marker }),
        }}
      />
    );
    // Notes anchor to source lines, so a file with notes keeps its source under the preview.
    if (image && isImagePath(filePath))
      return (
        <ImageDiff
          key={filePath}
          version={data.session}
          kind={summary.kind}
          labels={labels}
          keepSource={
            notes &&
            (target?.path === filePath ||
              Boolean(result?.overview.notes.some((item) => item.note.path === filePath)))
          }
          load={(side: ImageSide, signal: AbortSignal) => image(index, side, signal)}
        >
          {diff}
        </ImageDiff>
      );
    return (
      <>
        {summary.notice && data.diff?.rows.length ? (
          <p className="jj-notice">{summary.notice}</p>
        ) : null}
        {diff ?? <p className="jj-notice jj-pending-notice">{labels.loadingFile}</p>}
      </>
    );
  }

  return (
    <div
      className={`jayjay-review jj-review-workspace ${compact ? "jj-compact" : list ? "jj-stacked" : ""} ${fill ? "jj-fill" : ""} ${!result ? "jj-review-empty" : ""}`}
      aria-busy={pending}
      {...diffThemeAttributes(diffThemes)}
    >
      {error && (
        <div className={`jj-review-error ${!result ? "jj-review-error-empty" : ""}`} role="alert">
          {!result && brand}
          <p>{error === "worker-stopped" ? labels.workerError : error}</p>
          <div className="jj-review-error-actions">
            {errorActions}
            {onRetry !== null && (
              <button
                className="jj-review-retry"
                disabled={pending}
                onClick={onRetry ?? review.retry}
              >
                {labels.retry}
              </button>
            )}
            {!result && headerActions}
          </div>
        </div>
      )}
      {!result && !error && (
        <div className="jj-loading-hud" role="status">
          <span className="jj-loading-mark">{brand}</span>
          <span>{labels.loading}</span>
        </div>
      )}
      {result && (
        <div
          className="jj-review-columns"
          ref={columns}
          style={
            !compact && filesWidth.width
              ? ({ "--jj-files-width": `${filesWidth.width}px` } as CSSProperties)
              : undefined
          }
        >
          <ChangeHeader
            overview={result.overview}
            labels={labels}
            hideChangeId={hideChangeId}
            titleReference={titleReference}
            actions={headerActions}
            controls={compact ? undefined : list ? listControls : diffOptions}
            avatars={avatars}
            descriptionExpanded={descriptionExpanded}
          />
          {!compact && (
            <FileColumn
              onShowShortcuts={onShowShortcuts ? undefined : showShortcuts}
              header={filesHeader}
              overview={result.overview}
              selected={list ? -1 : selected}
              disabled={disabled}
              labels={labels}
              select={(index) => {
                setPicked(stacked);
                select(index);
              }}
              notes={notes}
              marks={reviewMode}
              onReviewMode={onReviewMode}
              stacked={list}
              onStacked={onStacked && toggleStacked}
              toggle={toggleFiles}
              filter={filter}
              setFilter={setFilter}
            />
          )}
          {!compact && (
            <ColumnResizer
              columns={columns}
              width={filesWidth.width}
              setWidth={filesWidth.setWidth}
              commit={filesWidth.commit}
              label={labels.resizeFiles}
            />
          )}
          <section
            className={`jj-review-detail ${overdue ? "jj-overdue" : ""}`}
            aria-label={labels.result}
            aria-busy={pending}
            ref={detail}
          >
            {compact &&
              filesSlot &&
              createPortal(
                <ToolButton
                  className="jj-files-button"
                  aria-haspopup="dialog"
                  aria-keyshortcuts="/"
                  label={labels.files}
                  onClick={openDrawer}
                >
                  <SidebarIcon />
                </ToolButton>,
                filesSlot,
              )}
            {compact && filesHeader}
            {compact && (
              <div className="jj-pane-heading jj-diff-options">
                {filesSlot ? (
                  <span className="jj-files-total">{filesTotal}</span>
                ) : (
                  <button
                    className="jj-header-action jj-files-button"
                    aria-haspopup="dialog"
                    aria-keyshortcuts="/"
                    title={labels.files}
                    onClick={openDrawer}
                  >
                    <ListIcon />
                    {filesTotal}
                  </button>
                )}
                <div className="jj-header-actions">{listControls}</div>
              </div>
            )}
            {!list && file && (
              <FileHeader
                file={file}
                labels={labels}
                marks={reviewMode}
                disabled={disabled}
                toggleReviewed={() => toggleFiles([result.file])}
                open
              />
            )}
            {result.syntaxUnavailable && (
              <p className="jj-notice">
                {labels.syntaxUnavailable}{" "}
                <button onClick={() => select(selected)}>{labels.retry}</button>
              </p>
            )}
            {!list && result.diff?.whitespaceOnlyHidden && (
              <p className="jj-notice">{labels.whitespaceOnly}</p>
            )}
            <NotesBanner
              items={unplaced}
              labels={labels}
              disabled={disabled}
              resolve={resolve}
              update={update}
              locate={locate}
            />
            {list ? (
              <div className="jj-file-sections" ref={sectionList}>
                {result.overview.files.map((entry, index) => {
                  const section = loaded.get(entry.path);
                  const current = section?.key === cacheKey;
                  return (
                    <LazySection
                      key={entry.path}
                      index={index}
                      version={cacheKey}
                      root={compact ? undefined : sectionList}
                      loaded={current || folded(entry)}
                      load={load}
                    >
                      <FileHeader
                        file={entry}
                        labels={labels}
                        marks={reviewMode}
                        disabled={disabled}
                        toggleReviewed={() => toggleFiles([index])}
                        fold={{ folded: folded(entry), toggle: () => toggleFold(entry) }}
                      />
                      <div
                        className="jj-diff-scroll"
                        data-path={entry.path}
                        aria-label={`${labels.lines}: ${entry.path}`}
                        style={
                          section || folded(entry)
                            ? undefined
                            : ({
                                "--jj-pending-lines": entry.added + entry.removed,
                              } as CSSProperties)
                        }
                      >
                        {!folded(entry) && (
                          <>
                            {failed.has(index) && !current && (
                              <p className="jj-notice">
                                {labels.renderError}{" "}
                                <button onClick={() => load(index)}>{labels.retry}</button>
                              </p>
                            )}
                            {section
                              ? diffPane(section.file, section.data, !current)
                              : !failed.has(index) && (
                                  <p className="jj-notice jj-pending-notice">
                                    {labels.loadingFile}
                                  </p>
                                )}
                          </>
                        )}
                      </div>
                    </LazySection>
                  );
                })}
                {!result.overview.files.length && (
                  <p className="jj-empty-diff">{labels.noChangedFiles}</p>
                )}
              </div>
            ) : (
              <div
                ref={diffScroll}
                className="jj-diff-scroll"
                data-path={path}
                tabIndex={0}
                aria-label={labels.lines}
              >
                {file ? (
                  diffPane(result.file, result)
                ) : (
                  <p className="jj-empty-diff">{labels.noChangedFiles}</p>
                )}
              </div>
            )}
            {statusBar && (
              <div className="jj-result-footer" aria-live="polite">
                <span>
                  {overdue ? labels.loadingFile : error ? labels.engineError : savedLabel}
                </span>
                <span>{result.elapsed.toFixed(1)} ms</span>
              </div>
            )}
          </section>
        </div>
      )}
      {result && compact && (
        <Dialog
          className="jj-file-drawer"
          label={labels.files}
          open={drawer}
          onClose={() => {
            setDrawer(false);
            // After the dialog hands focus back to its button, which would scroll the list to it.
            if (drawerPick.current !== null) jump(drawerPick.current);
            drawerPick.current = null;
          }}
        >
          {drawer && (
            <FileColumn
              overview={result.overview}
              selected={drawerFile}
              disabled={disabled}
              labels={labels}
              select={(index) => {
                drawerPick.current = index;
                setDrawer(false);
              }}
              notes={notes}
              marks={reviewMode}
              onReviewMode={onReviewMode}
              toggle={toggleFiles}
              filter={filter}
              setFilter={setFilter}
            />
          )}
        </Dialog>
      )}
      {shortcuts && <ShortcutsDialog labels={labels} onClose={() => setShortcuts(false)} />}
      {footer}
    </div>
  );
}
