/** @jsxRuntime automatic */
import {
  useMemo,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type MouseEvent,
  type ReactNode,
} from "react";
import type { Labels } from "../../model/diff";
import {
  ChevronIcon,
  CloseIcon,
  FolderIcon,
  HideReviewedIcon,
  KeyboardIcon,
  ListIcon,
  OneListIcon,
  ReviewModeIcon,
  SearchIcon,
  TreeIcon,
} from "../icons";
import { ContextMenu, type ContextMenuItem } from "../ContextMenu";
import { KindBadge } from "./KindBadge";
import { MiddlePath } from "./MiddlePath";
import { ReviewToggle } from "./ReviewToggle";
import { ToolButton } from "../ToolButton";
import { toggled } from "../toggled";
import { folderReview, type FileSummary, type Overview } from "../../model/review";

function folders(path: string): string[] {
  const result: string[] = [];
  for (let slash = path.indexOf("/"); slash !== -1; slash = path.indexOf("/", slash + 1))
    result.push(path.slice(0, slash));
  return result;
}

interface MenuTarget {
  x: number;
  y: number;
  path: string;
  files: number[];
  /** The file's own row, which a mark may unmount. */
  row: HTMLElement | null;
}

// Opens a row's context menu on right click, the Menu key, or ⌃Return, as the desktop app does.
type RowMenu = (
  path: string,
  files: number[],
) => {
  onContextMenu: (event: MouseEvent) => void;
  onKeyDown: (event: KeyboardEvent) => void;
};

function FileRow({
  file,
  index,
  name,
  folder,
  depth,
  selected,
  disabled,
  labels,
  select,
  toggle,
  mark,
  menu,
  marks,
}: {
  file: FileSummary;
  index: number;
  name: string;
  folder: string;
  depth: number;
  selected: number;
  disabled: boolean;
  labels: Labels;
  select: (index: number) => void;
  toggle: (files: number[]) => void;
  mark: (index: number, row: HTMLElement) => void;
  menu: RowMenu;
  marks: boolean;
}) {
  const kind = file.kind ?? "modified";
  const shownPath = file.oldPath ? `${file.oldPath} → ${file.path}` : file.path;
  return (
    <div
      className={`jj-file-row ${marks ? `jj-${file.review}` : ""} ${selected === index ? "jj-selected" : ""}`}
      style={{ "--jj-row-indent": `${8 + depth * 12}px` } as CSSProperties}
      {...menu(file.path, [index])}
    >
      {/* Safari leaves a clicked button unfocused, and Space needs it focused. */}
      <button
        className="jj-file-select"
        data-file={index}
        aria-current={selected === index ? "true" : undefined}
        aria-label={file.path}
        aria-description={labels[`kind.${kind}`]}
        aria-keyshortcuts="Space J K N P"
        title={marks ? `${shownPath} · ${labels[file.review]}` : shownPath}
        onClick={(event) => {
          event.currentTarget.focus();
          select(index);
        }}
        onKeyDown={(event) => {
          if (event.key !== " " || !marks) return;
          event.preventDefault();
          if (!event.repeat && !disabled) mark(index, event.currentTarget);
        }}
      >
        <KindBadge kind={kind} label={labels[`kind.${kind}`]} />
        <span className="jj-file-text">
          <span className="jj-file-name">
            {name}
            {file.noteCount > 0 && (
              <span className="jj-note-count" title={labels.notes}>
                {file.noteCount}
              </span>
            )}
          </span>
          {file.oldPath ? (
            <span className="jj-file-path">
              <MiddlePath path={file.oldPath} className="jj-old-path" />
              <span aria-hidden="true">→</span>
              <MiddlePath path={file.path} />
            </span>
          ) : (
            folder && (
              <span className="jj-file-path">
                <MiddlePath path={folder} />
              </span>
            )
          )}
        </span>
      </button>
      {marks && (
        <ReviewToggle
          review={file.review}
          path={file.path}
          labels={labels}
          disabled={disabled}
          toggle={() => toggle([index])}
        />
      )}
    </div>
  );
}

interface FileFilter {
  searching: boolean;
  text: string;
  hideReviewed: boolean;
  onlyNotes: boolean;
}

export const noFileFilter: FileFilter = {
  searching: false,
  text: "",
  hideReviewed: false,
  onlyNotes: false,
};

export function FileColumn({
  overview,
  selected,
  disabled,
  labels,
  select,
  toggle,
  filter,
  setFilter,
  onShowShortcuts,
  notes = true,
  marks = true,
  onReviewMode,
  stacked = false,
  onStacked,
  header,
}: {
  overview: Overview;
  selected: number;
  disabled: boolean;
  labels: Labels;
  select: (index: number) => void;
  toggle: (files: number[]) => void;
  filter: FileFilter;
  setFilter: (filter: FileFilter) => void;
  onShowShortcuts?: () => void;
  notes?: boolean;
  /** Review mode: off hides the marks and everything that acts on them, and file rows show their kind as a badge. */
  marks?: boolean;
  /** Adds the review mode toggle to the tools. */
  onReviewMode?: () => void;
  stacked?: boolean;
  /** Adds the one-list toggle to the tools. */
  onStacked?: () => void;
  header?: ReactNode;
}) {
  const [tree, setTree] = useState(() => {
    try {
      return localStorage.getItem("jayjay.review.filesView") !== "list";
    } catch {
      return true;
    }
  });
  function toggleTree() {
    const next = !tree;
    setTree(next);
    try {
      localStorage.setItem("jayjay.review.filesView", next ? "tree" : "list");
    } catch {
      return;
    }
  }
  const { searching, text, onlyNotes } = filter;
  const hideReviewed = marks && filter.hideReviewed;
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const [menu, setMenu] = useState<MenuTarget | null>(null);
  const rowMenu: RowMenu = (path, files) => {
    const open = (row: Element, x: number, y: number) =>
      setMenu({ x, y, path, files, row: row.querySelector<HTMLElement>(".jj-file-select") });
    return {
      onContextMenu: (event) => {
        event.preventDefault();
        // The Menu key reports no pointer position.
        const box = event.currentTarget.getBoundingClientRect();
        if (event.clientX === 0 && event.clientY === 0)
          open(event.currentTarget, box.left + 24, box.bottom);
        else open(event.currentTarget, event.clientX, event.clientY);
      },
      onKeyDown: (event) => {
        if (event.key !== "Enter" || !event.ctrlKey) return;
        event.preventDefault();
        const box = event.currentTarget.getBoundingClientRect();
        open(event.currentTarget, box.left + 24, box.bottom);
      },
    };
  };
  // Built on each render, so the open menu follows a mark made or a load finished meanwhile.
  function menuItems({ path, files, row }: MenuTarget): ContextMenuItem[] {
    const copy = {
      label: labels.copyPath,
      run: () => void navigator.clipboard?.writeText(path).catch(() => undefined),
    };
    if (!marks) return [copy];
    const review = folderReview(files.map((index) => overview.files[index].review));
    return [
      {
        label: labels.markReviewed,
        checked: review === "reviewed" ? true : review === "unreviewed" ? false : "mixed",
        disabled,
        run: () => (row ? mark(files[0], row) : toggle(files)),
      },
      copy,
    ];
  }
  const visible = useMemo(
    () =>
      new Set(
        overview.files.flatMap((file, index) =>
          file.path.toLocaleLowerCase().includes(text.toLocaleLowerCase()) &&
          (!hideReviewed || file.review !== "reviewed") &&
          (!onlyNotes || file.noteCount > 0)
            ? [index]
            : [],
        ),
      ),
    [overview.files, text, hideReviewed, onlyNotes],
  );
  const shownFolders = useMemo(
    () => new Set([...visible].flatMap((index) => folders(overview.files[index].path))),
    [overview.files, visible],
  );
  const activeNotes = overview.notes.filter((note) => !note.note.resolved).length;
  const reviewedCount = overview.files.filter((file) => file.review === "reviewed").length;
  const reviewed = reviewedCount > 0;
  const { counts } = overview;
  const countLabel = text
    ? counts.filtered[visible.size]
    : marks && reviewed
      ? counts.reviewed
      : counts.files;

  function folder(path: string) {
    setCollapsed((previous) => toggled(previous, path));
  }
  function closeSearch() {
    setFilter({ ...filter, searching: false, text: "" });
  }
  function mark(index: number, row: HTMLElement) {
    // Hiding reviewed files unmounts this row, which would drop focus to the page.
    if (hideReviewed && overview.files[index].review !== "reviewed") {
      const list = row.closest<HTMLElement>(".jj-file-rows");
      const rows = [...(list?.querySelectorAll<HTMLElement>(".jj-file-select") ?? [])];
      const at = rows.indexOf(row);
      (rows[at + 1] ?? rows[at - 1] ?? list)?.focus();
    }
    toggle([index]);
  }

  return (
    <aside className="jj-file-column" aria-label={labels.files}>
      {header}
      <div className="jj-pane-heading jj-file-tools">
        {notes && (activeNotes > 0 || onlyNotes) && (
          <ToolButton
            className="jj-notes-filter"
            aria-pressed={onlyNotes}
            label={labels.onlyNotes}
            onClick={() => setFilter({ ...filter, onlyNotes: !onlyNotes })}
          >
            {activeNotes}
          </ToolButton>
        )}
        {onReviewMode && (
          <ToolButton
            aria-pressed={marks}
            label={marks ? labels.reviewModeOff : labels.reviewModeOn}
            onClick={onReviewMode}
          >
            <ReviewModeIcon />
          </ToolButton>
        )}
        {marks && (reviewed || hideReviewed) && (
          <ToolButton
            aria-pressed={hideReviewed}
            label={labels.hideReviewedFiles}
            title={hideReviewed ? labels.showingUnreviewed : labels.hideReviewedFiles}
            onClick={() => setFilter({ ...filter, hideReviewed: !hideReviewed })}
          >
            <HideReviewedIcon />
          </ToolButton>
        )}
        <ToolButton
          aria-pressed={tree}
          label={tree ? labels.showingTree : labels.showingFlat}
          onClick={toggleTree}
        >
          {tree ? <TreeIcon /> : <ListIcon />}
        </ToolButton>
        {onStacked && (
          <ToolButton aria-pressed={stacked} label={labels.showAllFileDiffs} onClick={onStacked}>
            <OneListIcon />
          </ToolButton>
        )}
        <ToolButton
          aria-pressed={searching}
          label={labels.filterFiles}
          onClick={() => (searching ? closeSearch() : setFilter({ ...filter, searching: true }))}
        >
          <SearchIcon />
        </ToolButton>
        <span className="jj-file-count" title={marks ? counts.reviewed : undefined}>
          {countLabel}
        </span>
        {onShowShortcuts && (
          <ToolButton label={labels.shortcuts} aria-keyshortcuts="?" onClick={onShowShortcuts}>
            <KeyboardIcon />
          </ToolButton>
        )}
      </div>
      {/* Always 2px tall, so the list does not shift when review mode turns the divider into progress. */}
      <div className="jj-file-progress" aria-hidden="true">
        {marks && overview.files.length > 0 && (
          <div style={{ width: `${(reviewedCount / overview.files.length) * 100}%` }} />
        )}
      </div>
      {searching && (
        <div className="jj-file-filter">
          <input
            type="search"
            autoFocus
            aria-label={labels.filterFiles}
            placeholder={labels.filterFiles}
            value={text}
            onChange={(event) => setFilter({ ...filter, text: event.target.value })}
            onKeyDown={(event) => {
              if (event.key !== "Escape") return;
              // A first Escape only clears the text, so it doesn't also close a dialog around the list.
              if (text) {
                event.preventDefault();
                setFilter({ ...filter, text: "" });
              } else closeSearch();
            }}
          />
          <ToolButton label={labels.cancel} onClick={closeSearch}>
            <CloseIcon />
          </ToolButton>
        </div>
      )}
      <div className="jj-file-rows" tabIndex={-1}>
        {visible.size === 0 && (
          <p className="jj-notice">
            {text
              ? labels.noMatchingFiles
              : overview.files.length
                ? labels.allFilesHidden
                : labels.noChangedFiles}
          </p>
        )}
        {tree
          ? overview.tree.map((entry) => {
              if (folders(entry.path).some((path) => collapsed.has(path))) return null;
              if (entry.file === null) {
                if (!shownFolders.has(entry.path)) return null;
                const open = !collapsed.has(entry.path);
                const inside = [...visible].filter((index) =>
                  overview.files[index].path.startsWith(`${entry.path}/`),
                );
                return (
                  <div
                    className="jj-folder-row"
                    key={`dir:${entry.path}`}
                    style={{ "--jj-row-indent": `${6 + entry.depth * 12}px` } as CSSProperties}
                    {...rowMenu(entry.path, inside)}
                  >
                    <button
                      className="jj-folder-name"
                      aria-expanded={open}
                      onClick={() => folder(entry.path)}
                    >
                      <ChevronIcon open={open} />
                      <FolderIcon />
                      <span>{entry.name}</span>
                    </button>
                    {marks && (
                      <ReviewToggle
                        review={folderReview(inside.map((index) => overview.files[index].review))}
                        path={entry.path}
                        labels={labels}
                        disabled={disabled}
                        toggle={() => toggle(inside)}
                      />
                    )}
                  </div>
                );
              }
              return visible.has(entry.file) ? (
                <FileRow
                  key={entry.path}
                  file={overview.files[entry.file]}
                  index={entry.file}
                  name={entry.name}
                  folder={entry.path.slice(0, Math.max(entry.path.lastIndexOf("/"), 0))}
                  depth={entry.depth}
                  selected={selected}
                  disabled={disabled}
                  labels={labels}
                  select={select}
                  toggle={toggle}
                  mark={mark}
                  menu={rowMenu}
                  marks={marks}
                />
              ) : null;
            })
          : overview.files.map((file, index) =>
              visible.has(index) ? (
                <FileRow
                  key={file.path}
                  file={file}
                  index={index}
                  name={file.path.slice(file.path.lastIndexOf("/") + 1)}
                  folder={file.path.slice(0, Math.max(file.path.lastIndexOf("/"), 0))}
                  depth={0}
                  selected={selected}
                  disabled={disabled}
                  labels={labels}
                  select={select}
                  toggle={toggle}
                  mark={mark}
                  menu={rowMenu}
                  marks={marks}
                />
              ) : null,
            )}
      </div>
      {menu && menu.files.every((index) => index < overview.files.length) && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          label={menu.path}
          items={menuItems(menu)}
          onClose={() => setMenu(null)}
        />
      )}
    </aside>
  );
}
