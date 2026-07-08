# GPUI Shell Guide

Load this file before changing the GPUI shell's layout, state ownership, globals, caches, or rendering conventions. Crate boundaries live in [Architecture Guide](architecture.md); shell-to-shell user-visible behavior belongs in [Shell Feature Parity Guide](shell-parity.md); review marks and notes in [Review State Guide](review-state.md).

`shell/gpui` is the cross-platform shell (macOS + Linux; packaged as an AppImage via `just gpui-appimage`). It links the Rust crates directly — no UniFFI, no Swift.

## File Layout

```text
shell/gpui/src/
├── app/           process-level: actions, TOML AppConfig store, Theme, native menus,
│                  fs_watcher/ (notify-based repo watcher), telemetry, editor/terminal launchers
├── diff/          pure view components: diff_view/ (unified + side-by-side, find bar, wrap_cache),
│                  file_column/ (flat + tree list, tree_cache), selection, annotate, image diff
├── platform/      cfg-gated macos.rs vs linux.rs seam: MOD_KEY, toolbar insets, menu-bar
│                  strategy (native vs in-window), reveal-in-file-manager
├── repo/          the main repo window feature
│   ├── revset/        change/compare/endpoint helpers over jj revsets
│   ├── toolbar/       top toolbar and buttons
│   ├── view_model/    RepoViewModel: domain state, loaders, mutations, tasks
│   └── window/        RepoWindow view: render, actions, sidebar, DAG, detail, review, status bar
├── ui/            reusable widgets: text_area/, input/, avatar/ (+cache), context_menu, scrollbar
└── windows/       secondary GPUI windows: settings/, bookmark_manager, operation_log/,
                   command_palette/, evolog, file_history — each opened via a static `open(...)`
```

## Conventions

- **State ownership** mirrors the SwiftUI split: `RepoWindow` (`repo/window/view.rs`) owns UI state — focus, pane/layout, find, diff selection, scroll handles, toasts, commit-box `TextArea`s, cache slots, the fs watcher, and the shared review-store handle. `RepoViewModel` (`repo/view_model/`) owns domain state — `Arc<Repo>`, graph data, selection, loaded diffs, compare state, loading flags. Input flows window → `vm.update(cx, …)` → `jayjay_core::Repo`.
- **Async** is centralized in `view_model/tasks.rs` (`background_update`, `repo_write_task`, `core_result_task`): heavy jj work on `cx.background_spawn`, re-entry via `this.update`, with per-section generation counters (`change_gen`, `diff_gen`, `pr_gen`, `refresh_gen`) dropping stale results.
- **Globals** (the only ones): `Theme`, `AppConfigStore` (TOML config), `ReviewStoreHandle` (process-wide review store — mutate only through `window/review.rs::mutate`), and the test-only `WatcherSuppressed`. Tests replace all of them via `tests/support::install_test_globals` (ephemeral config, light theme, in-memory review store, suppressed watcher).
- **Render caches** live in `Rc<RefCell<…>>` slots so re-renders reuse work: `DiffWrapCache` (wrapped lines keyed on `Arc<FileDiff>` identity + columns), `FileTreeCache` (flattened tree keyed on hunks identity + visibility + collapsed dirs), and the VM's `diff_cache`. Canvas prepaint writes panel bounds into `Rc<Cell<…>>` slots that mouse handlers read.
- **FS watcher** (`app/fs_watcher/`): notify events are classified into op-heads vs working-copy changes, debounced (1s/2s), relevance-filtered through `has_unignored_working_copy_paths`, and delivered over a flume channel into `vm.handle_working_copy_change`.

## Rendering Tips

- Row-like controls should usually set `.w_full()` before centering content; `justify_center()` only centers within the element's own width.
