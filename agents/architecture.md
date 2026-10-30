# Architecture Guide

Load this file before changing crate ownership, module boundaries, or cross-shell contracts. For shell file layouts and conventions load [SwiftUI Shell Guide](swiftui.md) or [GPUI Shell Guide](gpui.md); for review marks and notes load [Review State Guide](review-state.md).

## Workspace Crates

| Crate / target | Responsibility |
| --- | --- |
| `jayjay-primitives` | jj-lib-free domain types: `Change`, `Bookmark`, `DiffHunk`, review types, hashing, Seti file icons, and the shared file-tree builder |
| `jayjay-markdown` | Shared Markdown parser/event stream, local Mermaid compiler and SVG normalization, plus safe HTML renderer for rich previews |
| `jj-diff` | Diff engine: Histogram line diff, word diff, tree-sitter syntax, context collapse, wrapping, conflict display, canonical change groups, review group fingerprints |
| `jayjay-review` | Review marks, hunk baselines, notes, and reconciliation with optional native filesystem persistence (uses `jj-diff`) |
| `jayjay-network` | Platform-neutral HTTP request/response policy with an optional blocking adapter |
| `jayjay-i18n` | Fluent message catalog (`locales/<locale>/*.ftl`), locale negotiation, and generated typed accessors for Rust and Swift |
| `jayjay-core` | Portable analysis helpers plus native jj-lib/repository operations and the shared app-owned CLI dispatcher; re-exports `jj-diff` as `jayjay_core::diff` |
| `jayjay-uniffi` | UniFFI bindings for the SwiftUI shell and portable/WASM consumers; no business logic |
| `jayjay-cli` | `jayjay` binary: thin app launcher; app-owned CLI commands are forwarded to the bundled macOS executable, falling back to the GPUI binary (the only option on Linux/Windows) |
| `jj-test` | Shared jj repo fixtures for integration and component tests |
| `shell/gpui` | Cross-platform GPUI shell; links the Rust crates directly (no UniFFI); writes shared headless CLI outcomes before any window init |
| `shell/mac` | SwiftUI shell + the `JayJayDiffUI` Swift package (AppKit diff renderer) |
| `packages/review-web` | Reusable React review UI, typed async backend contract, optional worker/IndexedDB adapter, and thin WASM bindings to `jj-diff`, `jayjay-review`, and the shared file-tree builder |
| `shell/web` | Browser test host for `@jayjay/review-web`: mounts the local backend on a fixture change for Playwright |

Dependency direction (never invert): `primitives` is the leaf → `jj-diff` (may use `primitives`) → `jayjay-review` → `jayjay-core` → `jayjay-uniffi` / `shell/gpui`.
`jayjay-i18n` is a leaf any crate may depend on.

- Mermaid fences compile in `jayjay-markdown`; its SVG adapter resolves native colors, fonts, and CSS expressions for both shells. SwiftUI receives finished SVG through the existing HTML bridge and keeps page JavaScript disabled; GPUI rasterizes the shared diagram with its current theme.
- New shared types go in `jayjay-primitives`, not `jayjay-core`, so review/CLI code stays jj-lib-light.
- Anything two surfaces must agree on (change groups, review identity, group fingerprints, note reconciliation) lives at or below `jayjay-review`/`jj-diff` and is consumed by all surfaces. Do not re-implement a diff, fingerprint, or identity computation per surface; the GUI and `jayjay review notes` must reconcile through the same provider or notes silently report stale.
- App-owned headless commands parse, execute, and format their outcomes once in `jayjay-core`; GPUI calls that dispatcher directly and SwiftUI calls it through one UniFFI entry point. Shell code only writes stdout/stderr and exits.
- Binary file sizes stay locale-independent in materialized content. Shell presentation uses `FileSizeFormatter` after diffing; its span tooltips survive wrapping and never enter canonical review snapshots.
- Comparison endpoints, requests and displays live in `jayjay-core::compare`; a change's selection revision and label are `ChangeInfo` methods; multi-selection rules live in `jayjay-core::dag::OrderedSelection`. Both shells consume these portable types and keep native modifier handling locally.

## Build Profiles

Diffing, Tree-sitter syntax, DAG layout, file trees, themes, projections, and in-memory review state are always portable. `jayjay-core`'s `repository` feature adds jj-lib, Gix, the blocking network adapter, and repository operations; `watchman` adds filesystem monitoring. `jayjay-review`'s `storage` feature adds native filesystem persistence.

`jayjay-uniffi` defaults to its `desktop` profile, which enables repository operations, Watchman, review storage, and the binding generator CLI. A WASM consumer builds with `--no-default-features --features wasm`, supplies persistence and repository access, and implements the async `HttpTransport` foreign trait. Verify that graph and its LLVM-linked Tree-sitter grammars with `just test-wasm`.

`@jayjay/review-web` binds `jj-diff` and `jayjay-review` with wasm-bindgen, not UniFFI. Its React entry point consumes a typed async `ReviewBackend` and imports no worker or persistence, so hosts can supply native server state; the optional local adapter owns the worker and IndexedDB. A grammar-free base WASM owns review sessions and lazily loaded language groups only highlight. File-tree construction lives in primitives (`jayjay_core::file_tree` re-exports it) and labels come from `jayjay-i18n`. The bundle, storage, and native bridge contracts are in `packages/review-web/README.md`; build steps and browser checks are in `shell/web/README.md`.

## MVVM

```text
Rust Core -> UniFFI -> ViewModel -> SwiftUI Views
    |                                      
    +------------- direct link ---------> GPUI shell (view_model + window)
```

- **Model** (`crates/`): all business logic. Pure Rust, no platform code.
- **Bindings** (`crates/jayjay-uniffi/`): convert types and expose core APIs; do not add business logic. Bindings regenerate during `just ffi` (and as part of `just build`).
- **ViewModels** own the repo handle and all jj operations: `Repo/ViewModel/` in SwiftUI, `repo/view_model/` in GPUI.
- **Views**: feature folders in SwiftUI and GPUI. Views render state and call callbacks; they should not know jj internals.

Async conventions (both shells): heavy jj work runs off the UI thread (`Task.detached` → `MainActor.run` in Swift; `cx.background_spawn` → `this.update` in GPUI) and every in-flight result is guarded by a supersession check — a token, generation counter, or commit-id compare against current `@State`/VM state — so a slow stale result can never overwrite a newer one. Guard the result application; do not preemptively discard still-valid presentation state while its replacement loads. If an old-state action would be unsafe, disable that action explicitly instead of replacing the whole pane with an unrelated empty state. Both shells also suppress the FS-watcher echo of their own writes (`lastInternalMutationAt` / `last_internal_mutation_at`).

## Localization

User-facing text lives in `crates/jayjay-i18n/locales/en-US/<area>.ftl`; the file stem prefixes each message id (`settings-page-about` in `settings.ftl`). The build script turns each en-US message into a typed accessor: `jayjay_i18n::settings::page_about()` in Rust, `Localized.Settings.pageAbout` in Swift (generated by `just ffi` into `Sources/JayJayCore/Localized.swift`). A variable that selects a plural variant is numeric, any other is text. Translations go in `locales/<locale>/` with matching file names and a plain-text `-language-name` term naming the language in itself (`-language-name = 日本語`), which the Language picker lists; the build fails on an id or variable that en-US lacks or a missing term, and a missing message falls back to English.

- Add a string to en-US and call its accessor through its area (`settings::page_about()`, `Localized.Settings.pageAbout`), which is what `just lint-i18n` matches; never format English in a shell, and build plurals with Fluent selectors, not `count == 1` branches. Message, file, and variable names are lowercase words, never a Rust or Swift keyword; a selector is numeric only when its variants are plural categories or numbers.
- Element ids, debug selectors, accessibility identifiers, and persisted keys use stable slugs, never a localized label.
- Stay English: jj operation descriptions, jj/git output, revset syntax, internal error contexts, logs, and the `jayjay` CLI (it never calls `set_preferred_languages`).
- Shells call `set_preferred_languages` at launch: SwiftUI with `Locale.preferredLanguages`, GPUI with its `language` setting or else the OS list from `sys-locale`. The Language picker switches without a restart: SwiftUI calls `Localized.setLanguage`, which re-renders every view whose body read `Localized` and mirrors the choice into `AppleLanguages` for AppKit's next launch; GPUI's `config::update` re-applies it and refreshes windows and menus. Text a shell or core formatted earlier keeps its language until reloaded, so Swift date formatters use `Localized.locale` instead of `Locale.current`, and loaded diffs, which carry core-formatted separators, reload on a language change. They also call `set_number_locale` with the region, since numbers follow it rather than the UI language; core's `FileSizeFormatter` shares the same `decimal_formatter`. A language only matches a catalog of the same script.
- The `en-XA` pseudo-locale runs every catalog string through `fluent-pseudo` (lookalike letters, doubled vowels), so unmigrated text stands out and length assumptions show: `-AppleLanguages '(en-XA)'` for SwiftUI, `defaults write com.hewigovens.JayJay AppleLanguages -array en-XA` for the GPUI dev bundle, `LANGUAGE=en_XA` on Linux.
- `just lint-i18n` fails on catalog messages no shell uses and on prose literals anywhere in the shell sources. Product, font, and key names that stay English go in the `names` list in `scripts/lint-i18n.sh`; whole files that stay English (CLI help, generated bindings, model prompts) go in its `english` list. UI test scenes launch with `-AppleLanguages (en)`, except external-tool scenes, whose command line must stay exact.
- A new shipping locale also goes in `CFBundleLocalizations` under the `JayJay` target's `info.properties` in `shell/mac/project.yml` (XcodeGen regenerates `Info.plist` from it), so macOS offers it per app and localizes AppKit's own menus and panels.

## Tracing Cross-Layer Behavior

Before scoping a cross-layer bug, trace the real path end to end: input or persisted state → owning core operation or store → UniFFI or direct Rust boundary → shell view-model → presented output. Fix the lowest shared layer that owns the contract, then verify only the affected adapters and entry points.

## Core Modules

Keep `jayjay-core` logic split by responsibility under `repo/` (one file or folder per operation family: `log`, `diff/`, `mutations`, `bookmarks`, `git/`, `working_copy`, `resolve/`, `conflicts`, `fix/`, `annotate`, `evolog`, `diffedit/`, `stacked_pr/`, `pull_requests`, `review_notes`, `undo`, `workspace`, `overview`). Top-level portable modules (`dag`, `overview` grouping, `file_tree`, `fuzzy`, `palette`, `theme`, `commit_message`) are repo-free helpers. The native-only `repositories` module owns the file-backed pin contract shared by both desktop shells, and the native-only `cli` module owns the shared app CLI dispatcher (`--version`, `config`, `review …`) that both shells adapt.

## SSH Repository Sessions

The public `jayjay_core::Repo` is a local/SSH session; native jj-lib operations remain on `repo::Repo`. `repository/api.rs` declares the allowlisted methods once for client forwarding and helper dispatch. Keep shared records serializable; process-local objects such as cancellation tokens never cross the wire. `jayjay-ssh` embeds the core and runs over OpenSSH stdio. Snapshotting, transactions, and store locks belong on the host owning the working copy. A disconnected write has an uncertain outcome: reconnect and refresh before an explicit retry.

Remote repository locations carry a machine and workspace path (`ssh://…`); host paths cross the protocol `/`-rooted, so a Windows host's `C:\dir` travels as `/C:/dir` and only the helper converts it (`ssh/host_path.rs`); do not canonicalize them on the client, send them to local watchers, or pass them to local file-manager/editor/terminal launchers. In `repository/api.rs`, a `Location` argument is unqualified before it is sent and an `@paths` method has the host paths in its result qualified by the client; relative diff paths stay unchanged. Image bytes are transferred into a disposable local cache. Both shells poll remote working copies through their existing refresh suspension and generation guards. Build a target helper with `just ssh-helper <Rust target> [debug|release]`; Linux helpers are static `*-unknown-linux-musl` builds, made in an Apple `container` on macOS; Windows helpers (`*-pc-windows-msvc`, `jayjay-ssh.exe`) link the C runtime statically and are cross-built with `cargo xwin` on macOS (`rustup target add` the target first); the payload is staged in `build/ssh-helpers/<target>/` and copied into macOS bundles. Linux AppImages carry the native helper beside the app executable. Stage additional supported Linux/macOS/Windows target helpers from the same source revision before bundling for hosts of other architectures. The SSH helpers workflow builds the six supported target payloads and preserves executable modes in tar archives; unpack those archives into `build/ssh-helpers/` before packaging.

## Repository Operation Contracts

- Repository actions run through jj-lib, one transaction per action on the loaded repo (`repo/transaction.rs` helpers). Do not add `run_jj` call sites: a subprocess reloads the repo, re-snapshots the working copy, returns stderr text instead of typed errors, and resolves PATH and config on its own. The subprocess calls that remain stay by design, and a new one needs its reason recorded here: the command palette runs raw `jj` on purpose (`jj_command.rs`); `resolve --tool` uses the CLI's merge-tool plumbing (`repo/conflicts.rs`); the builtin revset alias listing reads jj-cli's `revsets.toml`, which jj-lib does not carry (`repo/resolve/aliases.rs`); fetch and push keep the CLI transport with its progress and cancellation handling (`repo/git/sync.rs`, the fetch in `repo/bookmarks.rs`), since jj-lib's transport spawns `git` either way; and clone runs `jj git clone` for the same transport plus the CLI's init and checkout, under a `SyncToken::standalone()` because no repository is open yet (`repo/clone.rs`).
- `DiffHunk.path` and `old_path` are repo-relative and `/`-separated, built from `RepoPath::as_internal_file_string()` — never `RepoPathUiConverter::format_file_path()`, which emits OS separators. File trees, visibility checks, and basenames split on `/`, and only the Windows CI job catches a regression.
- Every public `Repo` mutation starts with `let _write = self.write_guard();` (`repo/write_lock.rs`): one reentrant lock per repository store, shared by every window and workspace, so mutations run one at a time and resolve their operands after the previous one landed. Take it on the calling thread only; a thread the holder waits for must never take it. Every jj subprocess takes it too, since even a read like `jj diff` snapshots; only `jj git fetch`, `jj git push`, and the `--ignore-working-copy` reads of `run_jj_unlocked_read` run outside it.
- A working copy whose change another workspace rewrote is stale: its files predate the rewrite, so snapshotting it reverts the rewrite, and finishing its lock at a newer operation without a checkout hides the staleness until the next snapshot does. Code that takes the working-copy lock goes through `lock_fresh_working_copy` (or checks `ensure_working_copy_fresh` under a held lock, as `commit_transaction` does) in `repo/working_copy.rs` and returns `JayError::WorkingCopyStale`; `update_stale_workspace` is the recovery, and shells show it as a banner rather than an error.
- Repository-controlled paths and names passed to `jj`, `git`, `gh`, or `glab` are neutralized before they become operands: paths as exact `root-file:"…"` fileset literals, names after a `--` separator with option flags placed before it. Structured argv stops shell injection but not the tool's own option or fileset parser: a tracked file named `--config=…` or `all()` is otherwise code execution or a widened fileset. Every new mutation that takes a path or name extends `crates/jayjay-core/tests/integration/path_injection.rs`.
- External tools the user configures (`[fix.tools]` formatters, editors, AI providers) start through `subprocess_command` (`repo/environment.rs`), which resolves the program and sets `PATH` from the login shell; an app launched from the Dock or a desktop entry inherits only the system `PATH`.
- `jj` catches SIGTERM and exits 1 with empty stderr, so exit status and output cannot tell a cancelled process from a failed one. `repo/command_process.rs` marks a process as signaled only when `try_wait` shows it live under the registry lock; classify cancellation from that flag, never from status or stderr.
- Workspace removal validates the recorded root and live checkout identity in core. Forget disconnects a matching Git worktree without deleting checkout files; Forget & Delete stages the verified checkout beside its original path, forgets it, and deletes only that staged checkout. Git cleanup uses the staged path too, so a replacement at the original path stays untouched. Do not add op-id pinning, retry loops, or path caches around these ownership checks.
