# jayjay

A native GUI for [Jujutsu](https://github.com/jj-vcs/jj) version control.

## Features

- **DAG graph** with lane-based fork/merge visualization, bookmarks, conflict indicators
- **Native diff** with tree-sitter syntax highlighting (15 languages), context collapsing, rename detection
- **Unified + side-by-side** diff modes with copy that strips line numbers
- **All jj operations** — new, describe, squash, abandon, rebase, split (batch + single file)
- **Git integration** — push, fetch, submodule-aware commit
- **Bookmarks** — create, delete, filter by bookmark
- **Review workflow** — mark files as reviewed (space key), tree view, show in Finder
- **AI commit messages** — on-device via Apple Foundation Models (macOS 26+)
- **Multi-window** — open multiple repos, recent repos menu
- **Revset filter** — query any jj revset expression, default includes siblings

## Architecture

```
jj-lib (Rust)
  └── jayjay-core (diff, tree-sitter syntax, repo ops)
       └── uniffi bindings
            └── SwiftUI app (macOS)
```

- **Core**: `jj-lib` direct library access + `jj` CLI for git push/fetch (SSH auth)
- **Diff engine**: jj-lib word-level diff, tree-sitter for syntax tokens, Rust-computed
- **Renderer**: Native `NSTextView` + `NSAttributedString` — no WebView
- **macOS UI**: SwiftUI + `WindowGroup`

## Quick Start

```bash
# Build and launch
just run

# Build and open a specific repo
just run /path/to/jj/repo

# Install CLI launcher
just install-cli
jayjay .
```

## Prerequisites

- Rust 1.85+
- Xcode 16+ with macOS 14+ SDK
- [xcodegen](https://github.com/yonaskolb/XcodeGen), [xcbeautify](https://github.com/cpisciotta/xcbeautify), [just](https://github.com/casey/just)

## Project Structure

```
crates/
  jayjay-core/          Rust core: jj-lib, diff, tree-sitter (15 langs)
  jayjay-uniffi/        uniffi bindings + uniffi.toml config
  jayjay-cli/           Native CLI launcher
shell/mac/              macOS SwiftUI app
  Sources/JayJay/
    App/                Entry point, settings, window manager
    Views/              DAG, detail, file list
      Diff/             Unified + side-by-side renderers
      Components/       Commit box, bookmarks, settings
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
| Space | Toggle file reviewed (working copy) |
| ↑/↓ | Navigate files |

## License

Apache-2.0
