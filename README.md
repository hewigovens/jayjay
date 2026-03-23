# JayJay

A native macOS GUI for [Jujutsu](https://github.com/jj-vcs/jj) version control.

> Fast, keyboard-driven, built with Rust + SwiftUI.

[![CI](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hewigovens/jayjay?include_prereleases)](https://github.com/hewigovens/jayjay/releases)
![macOS](https://img.shields.io/badge/macOS-26-blue)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-BSL--1.1-green)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/jayjay)

<p align="center">
  <img src="docs/light.png" width="100%" alt="JayJay — light mode">
</p>

<p align="center">
  <img src="docs/dark.png" width="100%" alt="JayJay — dark mode">
</p>

## Features

**History & Graph**
- DAG visualization with lane-based fork/merge rendering
- Bookmark and conflict indicators on every node
- Revset filtering with preset chips (All, Mine, Bookmarks, Trunk, Conflicts, Heads)
- Load more: incrementally load older history
- Auto-refresh via file system watcher

**Diff & Review**
- Unified + side-by-side diff modes (toggle with one click)
- Interdiff: compare any two revisions (shift-click or context menu)
- File annotate (blame) with syntax-highlighted gutter — click to navigate
- File history: list all revisions that modified a file
- tree-sitter syntax highlighting (18 languages)
- Word-level change highlighting
- Context collapsing, rename detection
- Persistent file review state (survives restart, auto-invalidates on content change)

**Conflict Resolution**
- Conflicted files shown with warning indicator and resolution bar
- One-click "Use Ours" / "Use Theirs" resolution
- "Resolve in Editor" via `jj resolve --tool` (VS Code, Zed merge editors)
- Auto-refresh after resolution

**Operations**
- New, edit, describe, squash, abandon, split (with parallel option), graft, duplicate, merge
- Absorb hunks into ancestors, back out (revert) changes
- Git push/fetch with auto-track
- Bookmark management (create, move, delete, rename, track, push)
- Undo via operation log
- Command palette (⌘⇧P) with jj CLI integration

**AI Commit Messages**
- Codex CLI, Claude CLI, Apple Intelligence fallback chain

**Tools & Settings**
- External editor integration (VS Code, Zed, Xcode, Android Studio, Vim + auto-detection)
- Terminal integration (Terminal.app, iTerm2, Ghostty)
- Dock menu with recent repositories
- Font family picker + ⌘+/-/0 zoom
- Multi-window, recent repos, CLI launcher with URL scheme

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ⌘⇧P | Command palette |
| ⌘F | Find in diff |
| ⌘R | Refresh |
| ⌘O | Open repository |
| ⌘+/⌘-/⌘0 | Zoom in/out/reset |
| ⌘⇧U | Undo (operation log) |
| Space | Toggle file reviewed |
| Shift+Click | Compare two revisions (interdiff) |

## Install

**Download**: Grab the latest release from [GitHub Releases](https://github.com/hewigovens/jayjay/releases), unzip, and move to Applications.

**Build from source**:
```bash
just run               # Build and run
just install-cli       # Install CLI launcher to ~/.local/bin
jayjay .               # Open current repo
```

**Auto-update**: JayJay checks for updates automatically via Sparkle. You can also check manually from JayJay → Check for Updates. Auto-update may require App Management permission in System Settings → Privacy & Security.

**Requirements**: macOS 26 (Tahoe) recommended. macOS 15 (Sequoia) minimum.

## Roadmap

See [Roadmap.md](Roadmap.md) for near-term and long-term plans.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

- **Rust crates** (`crates/`): [Apache-2.0](crates/LICENSE)
- **macOS app** (`shell/`, everything else): [BSL 1.1](LICENSE) — free to use, modify, and redistribute; paid app store distribution requires permission. Converts to Apache-2.0 on 2030-03-23.
