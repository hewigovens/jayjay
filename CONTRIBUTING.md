# Contributing to JayJay

This project uses [Jujutsu](https://github.com/jj-vcs/jj) for version control, not git.

## Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| macOS | 26+ | |
| Rust | 1.96+ | |
| Xcode | 16+ | |
| jj | latest | |
| just | latest | |
| xcodegen | latest | |
| xcbeautify | latest | |

## Setup

Bootstrap a clean machine with one command:

```bash
./scripts/setup.sh
```

It installs Homebrew (if missing), Rust via rustup, and the Homebrew tools from the repo [Brewfile](Brewfile) — `jj`, `just`, `xcodegen`, `xcbeautify`, `swiftlint`, `swiftformat`, `gh`. It also configures `jj fix` by copying [.jj-config.toml](.jj-config.toml) into this clone (if not already set). It's idempotent and safe to re-run. Xcode is the one piece it can't install (not Homebrew-managed); the script prints instructions if it's missing.

## Development loop

```bash
just test      # Rust unit tests
just test-app  # Swift unit tests
just test-ui   # XCUITest scenes (builds fixtures, sets onboarding default)
just lint      # Clippy + SwiftLint
just format    # cargo fmt + SwiftFormat
just clean     # Remove generated build artifacts
just build     # Build the macOS app
just run       # Build and run macOS app
```

## Pull request workflow

Use [agents/pull-requests.md](agents/pull-requests.md) for GitHub and Codeberg PR workflows.

Optional tools:

| Tool | Needed for |
|------|------------|
| gh CLI | GitHub PR status, checks, and optional PR creation |
| GitHub account | GitHub PR creation from pushed bookmarks |
| Codeberg account | Codeberg PR creation from pushed bookmarks |

Summary:

- GitHub: push a jj bookmark, then use JayJay's **Pull Request on GitHub** action or `gh pr create`.
- Codeberg: push a jj bookmark, then use JayJay's **Pull Request on Codeberg** action to open Codeberg's PR compose page.

## Testing

**Every new feature ships with both unit and UI test coverage.** Bug fixes add the regression test that would have caught them.

- **Rust unit tests** — cover core logic in `crates/jayjay-core/`. Run with `just test`.
- **Swift unit tests** — cover ViewModel-level behavior in `shell/mac/Tests/JayJayTests/`. Run with `just test-app`.
- **XCUITest scenes** — cover user-visible flows in `shell/mac/Tests/JayJayUITests/`. Run with `just test-ui`.

UI tests launch the app against deterministic fixtures under `/tmp/jayjay-test-fixtures` built by `just shell::ui-test-setup`. Canonical fixtures cover simple, complex, structured-format, review-note, bookmark-diff, and conflict workflows. Repository-mutating scenes use workflow-named copies generated from those canonical fixtures so test order cannot leak state. Each scene subclasses `SceneBase` and asserts against accessibility identifiers declared in `shell/mac/Sources/JayJay/Shared/AccessibilityIdentifiers.swift`. When adding a new user-visible view or interaction:

1. Attach a stable `.accessibilityIdentifier(...)` to the view, keyed by the data that makes it unique (change-id prefix, file path, etc.). Add a constant/function to `AID` so tests and views share the same string.
2. Write a scene test under `Tests/JayJayUITests/Scenes/` that exercises the flow end-to-end.
3. If the scene needs fixture state that `simple`/`conflict` don't provide, extend `ui-test-setup` in `shell/justfile`.

## Architecture

```
Rust (crates/)                  Swift (shell/mac/)
├── jayjay-core                 ├── App/  Config, Window, Watcher
│   ├── repo (log, diff,        ├── Repo/ ViewModel, DAG, CommitBox
│   │   mutations, bookmarks,   ├── Detail/ files, tree view
│   │   git, working_copy,      ├── Diff/  unified, side-by-side
│   │   undo)                   ├── Settings/ prefs, jj config, about
│   ├── diff (LCS + word)       ├── Onboarding/ welcome flow
│   └── syntax (18 languages)   └── Shared/ reusable components
├── jayjay-uniffi ──── FFI ────
└── jayjay-cli (launcher)
```

| Layer | Tech | Role |
|-------|------|------|
| Model | Rust + jj-lib | Business logic, diff, syntax |
| Bindings | uniffi | Rust to Swift type bridge |
| ViewModel | `@Observable` | Async operations, state |
| View | SwiftUI + AppKit | Rendering |

## Backend split: `jj-lib` vs `jj` CLI

JayJay uses both.

Prefer `jj-lib` for:
- Structured reads and graph data: log, show, diff, bookmarks, diff stats
- Repo mutations where we need typed state, tree access, or custom composition
- New features that need reusable primitives in Rust, especially anything the UI will build on repeatedly

Prefer the `jj` CLI for:
- Features that are already stable in jj but awkward or unavailable in `jj-lib`
- External-tool flows such as `jj resolve --tool`
- Operations where JayJay is intentionally delegating to jj's own behavior and output

Current `jj-lib`-backed areas:
- Log, revset parsing, show/diff, bookmark data, diffedit application, most core mutations, working-copy refresh

Current `jj` CLI-backed areas:
- `resolve`, `workspace` add/list/forget, `undo` (`jj op`), `split`, `duplicate`, `absorb`, `revert`, parts of Git integration, AI commit-message helpers

Workspace **list** stays on `jj workspace list/root --ignore-working-copy`. Timestamp, description, and `@`-vs-parent file counts are filled from the in-memory jj-lib view (`get_wc_commit_id` + committed trees). Do not open sibling working copies or call `refresh_working_copy` from `workspace_list`.

When adding a feature:
1. Put business logic in Rust first.
2. Use `jj-lib` if it gives us a clear typed implementation.
3. Fall back to `jj` CLI when the library path is missing, unstable, or significantly more complex.
4. Document the choice here if it introduces a new long-term backend pattern.

## Updating docs

When a feature lands:
- Update [README.md](README.md) if it changes what users can do today.
- Update [Roadmap.md](Roadmap.md) if it changes planned vs shipped status.
- Update this file if it changes architecture, contributor workflow, the testing layout, or the `jj-lib` vs `jj` CLI split.

## Project reference

- [DeepWiki](https://deepwiki.com/hewigovens/jayjay) for indexed codebase docs and architecture browsing
  Useful when you need a quick high-level map before reading the source directly.

See [AGENTS.md](AGENTS.md) for development guidelines.
