/** @jsxRuntime automatic */
import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { diffThemeAttributes, type DiffThemeSelection } from "../themes";
import type { DiffAnchor, Labels } from "../model/diff";
import { isImagePath, type ImageSide, type ReviewNote, type ReviewResult } from "../model/review";
import { useCompact } from "../hooks/useCompact";
import { useFileShortcuts } from "../hooks/useFileShortcuts";
import { useFilesWidth } from "../hooks/useFilesWidth";
import { useFolding } from "../hooks/useFolding";
import { useLoadedSections } from "../hooks/useLoadedSections";
import { useNoteJump } from "../hooks/useNoteJump";
import type { ReviewController } from "../hooks/useReview";
import { DiffPane, lineLabel } from "./diff/DiffPane";
import { ImageDiff } from "./diff/ImageDiff";
import { Dialog } from "./Dialog";
import { ColumnResizer } from "./files/ColumnResizer";
import { FileColumn, noFileFilter } from "./files/FileColumn";
import { ShortcutsDialog } from "./files/ShortcutsDialog";
import { ChangeHeader } from "./header/ChangeHeader";
import { DiffOptionControls } from "./header/DiffOptionControls";
import { FileHeader } from "./header/FileHeader";
import { ListIcon, NoteIcon } from "./icons";
import { LazySection } from "./LazySection";
import { NoteBubble } from "./notes/NoteBubble";
import { NoteComposer, type NoteTarget } from "./notes/NoteComposer";
import { NotesBanner } from "./notes/NotesBanner";
import { anchorKey, placeNotes } from "./notes/notePlacement";
import { toggled } from "./toggled";

export function ReviewWorkspace({
  review,
  labels,
  savedLabel,
  brand,
  hideChangeId = false,
  onShowShortcuts,
  headerActions,
  errorActions,
  footer,
  onRetry,
  notes = true,
  avatars = false,
  fill = false,
  diffThemes,
}: {
  review: ReviewController;
  labels: Labels;
  savedLabel?: string;
  brand?: ReactNode;
  hideChangeId?: boolean;
  onShowShortcuts?: () => void;
  headerActions?: ReactNode;
  errorActions?: ReactNode;
  footer?: ReactNode;
  onRetry?: () => void;
  /** Review notes in the diff, their per-line buttons, and the banner of notes the diff cannot place. */
  notes?: boolean;
  /** Looks up the author's avatar on Gravatar, GitHub, or GitLab, which shares their email hash or username with that service. */
  avatars?: boolean;
  /** Fills the host's height edge to edge instead of sitting in a bordered card; the phone-width list still scrolls with the page. */
  fill?: boolean;
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
  const [target, setTarget] = useState<NoteTarget | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const detail = useRef<HTMLElement>(null);
  const columns = useRef<HTMLDivElement>(null);
  const diffScroll = useRef<HTMLDivElement>(null);
  const filesWidth = useFilesWidth();
  const file = result?.overview.files[result.file];
  const path = file?.path;
  const { cacheKey, loaded, failed, load } = useLoadedSections(result, revision, renderFile);
  const [filter, setFilter] = useState(noFileFilter);
  const [shortcuts, setShortcuts] = useState(false);
  const [sheet, setSheet] = useState(false);
  const [sheetFile, setSheetFile] = useState(0);
  const sheetPick = useRef<number | null>(null);
  const showShortcuts = onShowShortcuts ?? (() => setShortcuts(true));

  function sections() {
    return [...(detail.current?.querySelectorAll<HTMLElement>(".jj-file-section") ?? [])];
  }
  // Below the host's pinned bar and above the bottom of the window.
  function onScreen(row: HTMLElement) {
    const { top, bottom } = row.getBoundingClientRect();
    return bottom > parseFloat(getComputedStyle(row).scrollMarginTop) + 1 && top < innerHeight;
  }
  const topSection = (rows: HTMLElement[]) => rows.findIndex(onScreen);
  function jump(file: number) {
    const section = sections().find((row) => Number(row.dataset.file) === file);
    section?.focus({ preventScroll: true });
    section?.scrollIntoView({ block: "start" });
  }
  function openSheet() {
    const rows = sections();
    setSheetFile(Number(rows[Math.max(topSection(rows), 0)]?.dataset.file ?? 0));
    setSheet(true);
  }

  const [sheetLayout, setSheetLayout] = useState(compact);
  if (sheetLayout !== compact) {
    setSheetLayout(compact);
    setSheet(false);
  }

  useFileShortcuts({
    rows: () =>
      compact
        ? sections()
        : [...(columns.current?.querySelectorAll<HTMLElement>(".jj-file-select") ?? [])],
    // Focus runs ahead of the selection while a pick renders, but a phone scroll can leave it off screen.
    current: (rows, focused) =>
      compact
        ? focused >= 0 && onScreen(rows[focused])
          ? focused
          : topSection(rows)
        : focused >= 0
          ? focused
          : rows.findIndex((row) => Number(row.dataset.file) === selected),
    reviewed: (index) => result?.overview.files[index]?.review === "reviewed",
    go: (row) => {
      if (compact) return jump(Number(row.dataset.file));
      row.focus();
      select(Number(row.dataset.file));
    },
    list: () => (compact ? null : columns.current?.querySelector(".jj-file-rows")),
    search: () => {
      setFilter({ ...filter, searching: true });
      if (!compact) return;
      openSheet();
      // The sheet opens in this event's commit; showModal focuses its first button, not the filter.
      requestAnimationFrame(() =>
        columns.current?.parentElement
          ?.querySelector<HTMLElement>(".jj-file-sheet .jj-file-filter input")
          ?.focus(),
      );
    },
    showShortcuts,
  });

  useEffect(() => {
    setTarget(null);
    setExpanded(new Set());
  }, [compact ? undefined : path]);
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

  const jumpTarget = useNoteJump(detail, result, pending, compact, loaded);
  const { folded, unfold } = useFolding(detail, result, compact, target?.path);

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
    if (entry.review === "reviewed") unfold(entry.path);
    if (
      configure(
        {
          ...options,
          ignoreWhitespace: item.note.ignore_whitespace,
          showAll: options.showAll || (item.status === "current" && !folded(entry)),
        },
        index,
      )
    )
      jumpTarget.current = item;
  }

  const disabled = pending || Boolean(error);
  const update = (id: string, body: string) => mutate({ kind: "updateNote", id, body });
  const resolve = (id: string) => void mutate({ kind: "resolveNote", id });
  // The compact list re-renders the marked file's section; the wide layout re-renders the open file.
  async function toggleFile(index: number) {
    if (!result) return;
    const { files } = result.overview;
    const marking = files[index].review !== "reviewed";
    // The rendered rows give display order; the compact list has none and stays put.
    const order = [
      ...(columns.current?.querySelectorAll<HTMLButtonElement>(".jj-file-select") ?? []),
    ].map((row) => Number(row.dataset.file));
    if (
      !(await mutate(
        { kind: "setFile", file: index, reviewed: marking },
        compact ? index : undefined,
      )) ||
      !marking ||
      index !== selected ||
      target
    )
      return;
    const position = order.indexOf(index);
    const next = [...order.slice(position + 1), ...order.slice(0, Math.max(position, 0))].find(
      (file) => files[file].review !== "reviewed",
    );
    columns.current
      ?.querySelector<HTMLButtonElement>(`.jj-file-select[data-file="${next}"]`)
      ?.click();
  }

  const diffs = compact
    ? [...loaded.values()]
    : result
      ? [{ file: result.file, data: result }]
      : [];
  const { placed, rendered, unplaced } = placeNotes(
    diffs,
    result?.overview.files ?? [],
    notes && result ? result.overview.notes : [],
    folded,
  );

  const draftKey =
    target &&
    (compact ? loaded.get(target.path)?.key : target.path === path ? cacheKey : undefined);
  const draftLost =
    target !== null &&
    draftKey === cacheKey &&
    rendered.get(anchorKey(target.path, target.anchor.side, target.anchor.line)) !== target.text;
  useEffect(() => {
    if (draftLost) setTarget(null);
  }, [draftLost]);

  function diffPane(index: number, data: ReviewResult, stale = false) {
    const filePath = data.overview.files[index].path;
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
    const note = (anchor: DiffAnchor) =>
      setTarget({
        file: index,
        path: filePath,
        anchor,
        ignoreWhitespace: data.options.ignoreWhitespace,
        text: rendered.get(anchorKey(filePath, anchor.side, anchor.line)),
      });
    const diff = data.diff && (
      <DiffPane
        key={filePath}
        labels={labels}
        view={data.diff}
        split={data.options.split}
        showAll={() => configure({ ...options, showAll: true })}
        review={{
          states: data.groupStates,
          disabled: locked,
          toggle: (group) =>
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
          kind={data.overview.files[index].kind}
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
    return diff ?? <p className="jj-notice">{labels.loadingFile}</p>;
  }

  return (
    <div
      className={`jayjay-review jj-review-workspace ${compact ? "jj-compact" : ""} ${fill ? "jj-fill" : ""} ${!result ? "jj-review-empty" : ""}`}
      aria-busy={pending}
      {...diffThemeAttributes(diffThemes)}
    >
      {error && (
        <div className={`jj-review-error ${!result ? "jj-review-error-empty" : ""}`} role="alert">
          {!result && brand}
          <p>{error === "worker-stopped" ? labels.workerError : error}</p>
          <div className="jj-review-error-actions">
            {errorActions}
            <button
              className="jj-review-retry"
              disabled={pending}
              onClick={onRetry ?? review.retry}
            >
              {labels.retry}
            </button>
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
          {!compact && (
            <FileColumn
              onShowShortcuts={onShowShortcuts ? undefined : showShortcuts}
              overview={result.overview}
              selected={selected}
              disabled={disabled}
              labels={labels}
              select={select}
              notes={notes}
              toggle={toggleFile}
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
            className="jj-review-detail"
            aria-label={labels.result}
            aria-busy={pending}
            ref={detail}
          >
            <ChangeHeader
              overview={result.overview}
              labels={labels}
              hideChangeId={hideChangeId}
              actions={headerActions}
              avatars={avatars}
            />
            {compact && (
              <div className="jj-pane-heading jj-diff-options">
                <button
                  className="jj-header-action jj-files-button"
                  aria-haspopup="dialog"
                  aria-keyshortcuts="/"
                  title={labels.files}
                  onClick={openSheet}
                >
                  <ListIcon />
                  {result.overview.files.some((entry) => entry.review === "reviewed")
                    ? result.overview.counts.reviewed
                    : result.overview.counts.files}
                </button>
                <div className="jj-header-actions">
                  <DiffOptionControls labels={labels} options={options} configure={configure} />
                </div>
              </div>
            )}
            {!compact && file && (
              <FileHeader
                file={file}
                labels={labels}
                disabled={disabled}
                toggleReviewed={() => toggleFile(result.file)}
                controls={
                  <DiffOptionControls labels={labels} options={options} configure={configure} />
                }
              />
            )}
            {result.syntaxUnavailable && (
              <p className="jj-notice">
                {labels.syntaxUnavailable}{" "}
                <button onClick={() => select(selected)}>{labels.retry}</button>
              </p>
            )}
            {!compact && result.diff?.whitespaceOnlyHidden && (
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
            {compact ? (
              <div className="jj-file-sections">
                {result.overview.files.map((entry, index) => {
                  const section = loaded.get(entry.path);
                  const current = section?.key === cacheKey;
                  return (
                    <LazySection
                      key={entry.path}
                      index={index}
                      version={cacheKey}
                      loaded={current || folded(entry)}
                      load={load}
                    >
                      <FileHeader
                        file={entry}
                        labels={labels}
                        disabled={disabled}
                        toggleReviewed={() => toggleFile(index)}
                      />
                      <div
                        className="jj-diff-scroll"
                        data-path={entry.path}
                        aria-label={`${labels.lines}: ${entry.path}`}
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
                                  <p className="jj-notice">{labels.loadingFile}</p>
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
            <div className="jj-result-footer" aria-live="polite">
              <span>{pending ? labels.loadingFile : error ? labels.engineError : savedLabel}</span>
              <span>{result.elapsed.toFixed(1)} ms</span>
            </div>
          </section>
        </div>
      )}
      {result && compact && (
        <Dialog
          className="jj-file-sheet"
          label={labels.files}
          open={sheet}
          onClose={() => {
            setSheet(false);
            // After the dialog hands focus back to its button, which would scroll the list to it.
            if (sheetPick.current !== null) jump(sheetPick.current);
            sheetPick.current = null;
          }}
        >
          {sheet && (
            <FileColumn
              overview={result.overview}
              selected={sheetFile}
              disabled={disabled}
              labels={labels}
              select={(index) => {
                sheetPick.current = index;
                setSheet(false);
              }}
              notes={notes}
              toggle={toggleFile}
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
