# jayjay

A native GUI for [Jujutsu](https://github.com/jj-vcs/jj) version control.

> **Status: Pre-beta** — Actively developed, usable for daily work on macOS.

## Screenshots

<!-- TODO: Add screenshots -->

## Features

- **DAG graph** — lane-based fork/merge visualization with bookmarks and conflict indicators
- **Native diff** — tree-sitter syntax highlighting (15 languages), context collapsing, rename detection
- **Unified + side-by-side** — toggle diff modes, copy strips line numbers automatically
- **All jj operations** — new, describe, squash, abandon, rebase, split (batch + single file)
- **Git integration** — push, fetch, submodule-aware commit, auto-track new bookmarks
- **Bookmarks** — create on any change, push per-bookmark, delete
- **Review workflow** — mark files reviewed (space key), tree view, show in Finder
- **AI commit messages** — Apple Foundation Models (macOS 26+), with planned Codex/Claude fallback
- **Auto-refresh** — file system watcher on jj operations, no manual refresh needed
- **Multi-window** — open multiple repos, recent repos menu, persistent sidebar width

## Quick Start

```bash
# Prerequisites: Rust 1.85+, Xcode 16+, jj, xcodegen, xcbeautify, just

# Build and launch
just run

# Build and open a specific repo
just run /path/to/jj/repo

# Install CLI launcher
just install-cli
jayjay .
```

## Architecture

```
jj-lib (Rust)
  └── jayjay-core (diff, tree-sitter, repo operations)
       └── uniffi bindings
            └── SwiftUI app (macOS) — MVVM
```

| Layer | Tech | Role |
|-------|------|------|
| Model | Rust + jj-lib | All business logic, diff, syntax |
| Bindings | uniffi | Rust → Swift type bridge |
| ViewModel | `@Observable` | Async operations, state |
| View | SwiftUI + AppKit | Rendering only |

## Project Structure

```
crates/
  jayjay-core/          Rust: jj-lib wrapper, diff, tree-sitter (15 langs)
    src/repo/            Modules: log, diff, mutations, bookmarks, git, working_copy
  jayjay-uniffi/         uniffi bindings + config
  jayjay-cli/            Native CLI launcher
shell/mac/               macOS SwiftUI app
  Sources/JayJay/
    App/                 Entry point, settings, window manager, FS watcher
    Views/               DAG, detail, file list, welcome
      Diff/              Unified + side-by-side renderers
      Components/        Commit box, bookmarks, settings, about
      Shared/            DiffColors, SettingsComponents, LabeledRow
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ⌘N | New change |
| ⌘⇧S | Squash into parent |
| ⌘⌫ | Abandon change |
| ⌘⇧P | Git push |
| ⌘⇧F | Git fetch |
| ⌘R | Refresh |
| ⌘O | Open repository |
| ⌘⌥F | Show in Finder |
| Space | Toggle file reviewed |
| ↑/↓ | Navigate files |

## Roadmap

See [PLAN.md](PLAN.md) for the full implementation plan and beta checklist.

## Contributing

This project uses [Jujutsu](https://github.com/jj-vcs/jj) for version control, not git.
See [AGENTS.md](AGENTS.md) for development instructions.

## License

Apache-2.0
