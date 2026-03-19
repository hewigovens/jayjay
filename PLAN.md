# jayjay — Implementation Plan

## Phase 1: Rust Core + uniffi Bindings

### 1.1 Scaffold Rust workspace

```
jayjay/
├── crates/
│   ├── jayjay-core/       # Thin wrapper around jj-lib, business logic
│   └── jayjay-uniffi/     # uniffi interface definitions + exported types
├── macos/                  # Xcode project / Swift package
├── Cargo.toml              # Workspace root
└── PLAN.md
```

- `jayjay-core`: depends on `jj-lib`, exposes a clean API surface
- `jayjay-uniffi`: depends on `jayjay-core`, defines the `.udl` or proc-macro uniffi interface, builds the Swift bindings

### 1.2 Define the uniffi interface

Start minimal — only what the GUI needs:

**Repo operations**
- `open_repo(path: String) -> Repo`
- `repo_log(repo, revset: String) -> Vec<ChangeInfo>`
- `repo_diff(repo, rev: String) -> Vec<DiffHunk>`
- `repo_show(repo, rev: String) -> ChangeDetail`

**Mutations**
- `describe(repo, rev: String, message: String)`
- `new_change(repo, parent: String, message: String)`
- `squash(repo, rev: String, into: Option<String>)`
- `abandon(repo, rev: String)`
- `rebase(repo, rev: String, dest: String)`
- `split(repo, rev: String, paths: Vec<String>)` (file-level split; hunk-level is hard without UI)

**Bookmarks**
- `list_bookmarks(repo) -> Vec<BookmarkInfo>`
- `move_bookmark(repo, name: String, to: String)`
- `create_bookmark(repo, name: String, rev: String)`
- `delete_bookmark(repo, name: String)`

**Git**
- `git_push(repo, bookmark: String)`
- `git_fetch(repo, remote: String)`

**Types**
```
ChangeInfo { change_id, commit_id, description, author, timestamp, parents, bookmarks, is_working_copy }
ChangeDetail { info: ChangeInfo, diff: Vec<DiffHunk> }
DiffHunk { path, old_content, new_content, hunk_type: Added|Removed|Modified }
BookmarkInfo { name, change_id, is_tracking_remote }
```

### 1.3 Build and test bindings

- `cargo build` produces `.dylib` + generated Swift files
- Write Rust integration tests against a temp jj repo
- Verify Swift can import and call the generated types

## Phase 2: macOS SwiftUI App (MVP)

### 2.1 Project setup

- Xcode project or Swift Package with macOS app target
- Link the uniffi-generated `.xcframework`
- `WindowGroup` based (NOT document-based)
- Open repo via folder picker or CLI arg

### 2.2 Core views

**RepoWindow** — main window per repo
```
┌─────────────────────────────────────────┐
│ toolbar: [repo path] [revset filter]    │
├──────────────┬──────────────────────────┤
│              │                          │
│  DAG graph   │   Detail panel           │
│  (left)      │   (right)                │
│              │                          │
│  ● change    │   Description            │
│  │           │   Author / timestamp     │
│  ● change    │   Diff view              │
│  │           │                          │
│  ● change    │                          │
│              │                          │
├──────────────┴──────────────────────────┤
│ status bar: working copy info           │
└─────────────────────────────────────────┘
```

**DAGView** — left panel
- Render `repo_log()` as a graph with branch lines
- Each node shows: short change_id, first line of description, bookmarks
- Click to select → loads detail panel
- Revset text field for filtering
- Drag-and-drop for rebase (stretch goal)

**DetailView** — right panel
- Change description (editable → calls `describe()`)
- Author, timestamp, commit_id
- File list with diff hunks

**DiffView** — inside detail panel
- Unified diff (default)
- Side-by-side toggle (like diffs.com)
- Syntax highlighting via tree-sitter (via a Swift wrapper or pre-highlighted from Rust)

### 2.3 Actions

Toolbar or context menu:
- **New** → `new_change()`
- **Squash** → `squash()`
- **Abandon** → `abandon()`
- **Describe** → inline edit in detail panel
- **Push** → `git_push()`
- **Fetch** → `git_fetch()`

Keyboard shortcuts:
- `⌘N` — new change
- `⌘S` — describe (save description)
- `⌘⇧S` — squash
- `⌘⌫` — abandon (with confirmation)
- `⌘⇧P` — push

### 2.4 Multi-window

- `WindowGroup` with `openWindow` environment action
- Each window holds its own `Repo` instance
- Recent repos in File → Open Recent
- `⌘O` opens folder picker

## Phase 3: Semantic Diff

### 3.1 tree-sitter integration

- Option A: Run tree-sitter in Rust (via `tree-sitter` crate), pass structured diff data to Swift via uniffi
- Option B: Use `SwiftTreeSitter` package on the Swift side

Option A is better — keeps diffing logic in Rust, Swift just renders.

### 3.2 Diff types

```
StructuralDiff {
    path: String,
    language: String,
    changes: Vec<StructuralChange>,
}

StructuralChange {
    kind: String,          // "function", "class", "import", "statement", etc.
    name: Option<String>,  // e.g. function name
    change_type: Added | Removed | Modified,
    old_range: Option<Range>,
    new_range: Option<Range>,
    old_text: Option<String>,
    new_text: Option<String>,
}
```

### 3.3 Rendering

- Collapsed by default: "function `foo()` modified"
- Expand to see inline word-level diff
- Side-by-side view with aligned hunks (diffs.com style)
- Color: green added, red removed, yellow modified

## Phase 4: Polish + Platform Expansion

### 4.1 macOS polish
- Conflict visualization (jj materializes conflicts in files)
- Undo/redo via `jj op log`
- File status icons in Finder (optional, via FinderSync extension)
- Sparkle for auto-updates or Homebrew cask via `release-macos` skill

### 4.2 Linux (Qt or GTK)
- Same `jayjay-core` + `jayjay-uniffi` crates
- uniffi generates Kotlin bindings → could use Compose Multiplatform
- Or: generate C bindings → use with Qt (C++) or GTK (C/Vala)
- Or: skip uniffi, use Rust directly with `gtk-rs` or `slint`

### 4.3 Windows
- TBD — depends on Linux approach
- If Qt: already cross-platform
- If `slint`: already cross-platform

## Open Questions

- [ ] jj-lib API stability — pin version or track latest?
- [ ] uniffi vs swift-bridge — uniffi is more mature, swift-bridge is lighter
- [ ] Diff rendering: tree-sitter in Rust or Swift side?
- [ ] Linux toolkit: gtk-rs vs Qt vs slint vs Compose?
- [ ] How to handle jj operations that need user input (e.g. merge conflicts)?
- [ ] Licensing: MIT? Keep consistent with jj (Apache-2.0)?
