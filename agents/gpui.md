# GPUI Shell Guide

Load this file before changing the GPUI shell's layout, state ownership, globals, caches, or rendering conventions. Crate boundaries live in [Architecture Guide](architecture.md); review marks and notes in [Review State Guide](review-state.md). Cross-shell user-visible gaps are listed in [Shell Feature Parity Guide](shell-parity.md) and refreshed at release.

`shell/gpui` is the Linux parity shell, packaged as an AppImage via `just gpui-appimage`. It also builds on macOS for development, but the shipped macOS product remains SwiftUI and GPUI macOS integrations are not a parity target. GPUI links the Rust crates directly — no UniFFI, no Swift. Its Cargo package version is synchronized with the SwiftUI release version by `just set-version`.

## File Layout

```text
shell/gpui/src/
├── app/           process-level: actions, TOML AppConfig store, Theme, native menus,
│                  fs_watcher/ (notify-based repo watcher), telemetry, editor/terminal launchers
├── cli/           headless `jayjay review ...`/`--version` dispatch; runs from main() before any
│                  GPUI/window init so it works with no display (Linux agents, jayjay-cli fallback)
├── diff/          pure view components: diff_view/ (unified + side-by-side, find bar, wrap_cache,
│                  rows/ shared row model with inline note rows, gutter_mouse, edit_selection,
│                  note banners), file_column/ (flat + tree list, tree_cache), line/ (gutter,
│                  note_row), selection (text + gutter line-range), annotate, image/svg/markdown
│                  rich previews (media_diff, markdown_diff)
├── platform/      cfg-gated macos.rs vs linux.rs seam: MOD_KEY, toolbar insets, menu-bar
│                  strategy (native vs in-window), URL opening, reveal-in-file-manager
├── repo/          the main repo window feature
│   ├── revset/        change/compare/endpoint helpers over jj revsets
│   ├── toolbar/       top toolbar and grouped-capsule buttons
│   ├── view_model/    RepoViewModel: domain state, loaders (incl. review_notes reconciliation),
│   │                  mutations (incl. abandon_selected_diff_lines), tasks
│   └── window/        RepoWindow view: render, actions, sidebar, DAG, detail, review, status bar,
│                      gutter_menu/note_menu/note_composer (review-note + abandon-lines UI)
├── ui/            reusable widgets: text_area/, input/, avatar/ (+cache), context_menu,
│                  button_group, scrollbar (uniform-list + free-form ScrollHandle variants)
└── windows/       secondary GPUI windows: settings/, bookmark_manager, operation_log/,
                   command_palette/, evolog, file_history — each opened via a static `open(...)`
```

## Conventions

- **State ownership** mirrors the SwiftUI split: `RepoWindow` (`repo/window/view.rs`) owns UI state — focus, pane/layout, find, diff text selection + gutter line selection (mutually exclusive), scroll handles, toasts, commit-box `TextArea`s, the text-modal overlay (edit description / create bookmark / note composer), cache slots, the fs watcher, and the shared review-store handle. `RepoViewModel` (`repo/view_model/`) owns domain state — `Arc<Repo>`, graph data, selection, loaded diffs (with retained old/new content for diff-edit mappings), reconciled review notes, compare state, loading flags. Input flows window → `vm.update(cx, …)` → `jayjay_core::Repo`.
- **Async** is centralized in `view_model/tasks.rs` (`background_update`, `repo_write_task`, `core_result_task`): heavy jj work on `cx.background_spawn`, re-entry via `this.update`, with per-section generation counters (`change_gen`, `diff_gen`, `pr_gen`, `refresh_gen`, `review_notes_gen`) dropping stale results.
- **Globals** (the only ones): `Theme`, `AppConfigStore` (TOML config), `repositories::StoreHandle` (the shared Rust-backed project pins), `ReviewStoreHandle` (process-wide review store — mutate only through `window/review.rs::mutate`), `FeedbackUrlOpener` (injectable platform URL boundary), and the test-only `WatcherSuppressed`. Component tests install ephemeral config, light theme, in-memory pin/review stores, and a suppressed watcher through `tests/support::install_test_globals`; feedback dispatch tests replace `FeedbackUrlOpener` directly.
- **Render caches** live in `Rc<RefCell<…>>` slots so re-renders reuse work: `DiffWrapCache` (wrapped lines keyed on `Arc<FileDiff>` identity + columns, plus the interleaved note-row list keyed additionally on a notes fingerprint that changes on both in-process mutations and external store reloads), `FileTreeCache` (flattened tree keyed on hunks identity + visibility + collapsed dirs), and the VM's `diff_cache`. Canvas prepaint writes panel bounds into `Rc<Cell<…>>` slots that mouse handlers read. Anything mapping a display line to a uniform_list item index must go through `row_index_for_line` — note rows shift row indices past their line indices.
- **FS watcher** (`app/fs_watcher/`): notify events are classified into op-heads vs working-copy changes, debounced (1s/2s), relevance-filtered through `has_unignored_working_copy_paths`, and delivered over a flume channel into `vm.handle_working_copy_change`.

## Rendering Tips

- Row-like controls should usually set `.w_full()` before centering content; `justify_center()` only centers within the element's own width.
