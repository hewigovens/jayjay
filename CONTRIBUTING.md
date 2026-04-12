# Contributing to JayJay

This project uses [Jujutsu](https://github.com/jj-vcs/jj) for version control, not git.

## Requirements

| Dependency | Version |
|------------|---------|
| macOS | 15+ (Sequoia) |
| Rust | 1.85+ |
| Xcode | 16+ |
| jj | latest |
| just | latest |
| xcodegen | latest |
| xcbeautify | latest |

## Development loop

```bash
just test      # Run Rust tests
just lint      # Clippy + SwiftLint
just format    # cargo fmt + SwiftFormat
just clean     # Remove generated build artifacts
just build     # Build the macOS app
just run       # Build and run macOS app
```

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
- `resolve`, `workspace`, `undo` (`jj op`), `split`, `graft`, `duplicate`, `absorb`, `backout`, parts of Git integration, AI commit-message helpers

When adding a feature:
1. Put business logic in Rust first.
2. Use `jj-lib` if it gives us a clear typed implementation.
3. Fall back to `jj` CLI when the library path is missing, unstable, or significantly more complex.
4. Document the choice here if it introduces a new long-term backend pattern.

## Updating docs

When a feature lands:
- Update [README.md](README.md) if it changes what users can do today.
- Update [Roadmap.md](Roadmap.md) if it changes planned vs shipped status.
- Update this file if it changes architecture, contributor workflow, or the `jj-lib` vs `jj` CLI split.

## Project reference

- [DeepWiki](https://deepwiki.com/hewigovens/jayjay) for indexed codebase docs and architecture browsing
  Useful when you need a quick high-level map before reading the source directly.

See [AGENTS.md](AGENTS.md) for development guidelines.
