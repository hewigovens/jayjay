# JayJay

**A native macOS GUI for [Jujutsu](https://github.com/jj-vcs/jj) (jj)** — fast, keyboard-driven, built with Rust + SwiftUI.

Browse your DAG, review side-by-side diffs, resolve conflicts, and run every jj operation from one window. JayJay is a full-featured **jj GUI** with DAG visualization, interdiff (PR-style revision comparison), diff edit mode, file annotate, one-click conflict resolution, AI-generated commit messages, and a command palette.

[![CI](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hewigovens/jayjay?include_prereleases)](https://github.com/hewigovens/jayjay/releases)
![macOS](https://img.shields.io/badge/macOS-26-blue)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-BSL--1.1-green)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/jayjay)

## Features

**History & Graph**
- DAG visualization with lane-based fork/merge rendering
- Bookmark and conflict indicators on every node
- Revset filtering with preset chips (All, Mine, Bookmarks, Trunk, Conflicts, Heads)
- Load more: incrementally load older history
- Auto-refresh via file system watcher

**Diff & Review**
- Unified + side-by-side diff modes (toggle with one click)
- Myers diff algorithm (via [`jj-diff`](#jj-diff) crate) — instant diffs even for large files
- Diff edit mode (`jj diffedit`-style): select files, hunks, or line ranges across a change
- Interdiff: compare any two revisions (shift-click or context menu)
- File annotate (blame) with syntax-highlighted gutter — click to navigate
- File history: list all revisions that modified a file
- tree-sitter syntax highlighting (18 languages)
- Word-level change highlighting
- Context collapsing, rename detection
- Background preloading of all file diffs
- Persistent file review state (survives restart, auto-invalidates on content change)

**Conflict Resolution**
- Conflicted files shown with warning indicator and resolution bar
- One-click "Use Ours" / "Use Theirs" resolution
- "Resolve in Editor" via `jj resolve --tool` (VS Code, Zed merge editors)
- Auto-refresh after resolution

**Operations**
- New, edit, describe, squash, abandon, split (with parallel option), graft, duplicate, merge
- Extract selected diff to child/parallel changes, move selected changes to `@`, discard selected working-copy changes
- Absorb hunks into ancestors, back out (revert) changes
- Git push/fetch with auto-track
- Bookmark Manager (⌘⇧B) with stats, filter, clean up stale branches, resolve conflicts
- Divergent commit detection and resolution
- Undo via operation log
- Command palette (⌘⇧P) with ~35 commands and jj CLI integration

**AI Commit Messages**
- Codex CLI, Claude CLI, Apple Intelligence fallback chain

**Tools & Settings**
- External editor integration (VS Code, Zed, Xcode, Android Studio, Vim + auto-detection)
- Terminal integration (Terminal.app, iTerm2, Ghostty)
- Dock menu with recent repositories
- Font family picker + ⌘+/-/0 zoom
- Commit avatars (GitHub + Gravatar)
- Multi-window, recent repos, CLI launcher with URL scheme

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ⌘⇧P | Command palette |
| ⌘F | Find in diff |
| ⌘R | Refresh |
| ⌘O | Open repository |
| ⌘+/⌘-/⌘0 | Zoom in/out/reset |
| ⌘⇧B | Bookmark Manager |
| ⌘⇧U | Undo (operation log) |
| Space | Toggle file reviewed |
| Shift+Click | Compare two revisions (interdiff) |

## Screenshots

<p align="center">
  <img src="docs/imgs/home.png" width="100%" alt="JayJay — light mode">
</p>

<p align="center">
  <img src="docs/imgs/home-dark.png" width="100%" alt="JayJay — dark mode">
</p>

## Install

**Homebrew**:
```bash
brew install --cask hewigovens/tap/jayjay
```

**Download**: Grab the latest release from [GitHub Releases](https://github.com/hewigovens/jayjay/releases/latest), unzip, and move to Applications.

**Build from source**:
```bash
just run               # Build and run
just install-cli       # Install CLI launcher to ~/.local/bin
jayjay .               # Open current repo
```

**Auto-update**: JayJay checks for updates automatically via Sparkle. You can also check manually from JayJay → Check for Updates. Auto-update may require App Management permission in System Settings → Privacy & Security.

**Requirements**: macOS 26 (Tahoe) recommended.

## Roadmap

See [Roadmap.md](Roadmap.md) for near-term and long-term plans.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture notes, development workflow, and the current `jj-lib` vs `jj` CLI backend split.

## jj-diff

The diff engine is extracted as a standalone Rust crate with **zero dependency on jj-lib**:

```toml
[dependencies]
jj-diff = { git = "https://github.com/hewigovens/jayjay" }
```

**Features:**
- Myers line diff (via `similar`) — O(n*d), same algorithm as libgit2/GitHub Desktop
- Word-level diff highlighting
- tree-sitter syntax highlighting (18 languages)
- Context collapsing with display-to-full line index mapping
- Skip highlighting for `.lock`/`.csv`/`.svg` files

**Usage:**
```rust
use jj_diff::{compute_file_diff, collapse_context_with_mapping};

let diff = compute_file_diff("main.rs", &old_content, &new_content, false);
let collapsed = collapse_context_with_mapping(&diff);
for line in &collapsed.diff.lines {
    // render line with line.style, line.spans
}
```

## FAQ

**What is JayJay?**
JayJay is a native macOS GUI for [Jujutsu (jj)](https://github.com/jj-vcs/jj), the modern version control system that reimagines the git model with mutable history, changes instead of commits, and bookmarks instead of branches. JayJay is built with Rust (wrapping `jj-lib` directly — no CLI scraping) and SwiftUI.

**Is there a GUI for Jujutsu?**
Yes — JayJay is a full-featured native macOS **jj GUI**. It gives you DAG visualization, unified + side-by-side diffs with tree-sitter syntax highlighting, interdiff (PR-style revision comparison), diff edit mode, file annotate, one-click conflict resolution, and every common jj operation (`squash`, `split`, `graft`, `absorb`, `backout`, `merge`, `duplicate`, `describe`, `abandon`) — without memorizing the command flags.

**How do I install JayJay on macOS?**
`brew install --cask hewigovens/tap/jayjay` is the easiest path. Or download the signed, notarized zip from [GitHub Releases](https://github.com/hewigovens/jayjay/releases/latest). Requires macOS 15 Sequoia or later.

**Does JayJay support interdiff (comparing two revisions)?**
Yes. Shift-click two revisions in the DAG to compare them PR-style — same unified + side-by-side diff views with word-level highlighting, but rendering the difference between the two change contents.

**Does JayJay support side-by-side diffs?**
Yes. Toggle between unified and side-by-side diff modes with one click. Both use tree-sitter syntax highlighting (18 languages) and word-level change highlights. Image files (PNG, JPG, SVG, HEIC, …) render as actual images with before/after panes, not as `<binary>` placeholders.

**Can JayJay extract individual lines or hunks into a new change?**
Yes — this is JayJay's **diff edit mode**. Select files, hunks, or line ranges across a change and split them out into a new child or parallel change (`jj diffedit` workflow). You can also right-click → Abandon Selected Lines on the working copy to drop individual edits.

**How is JayJay different from using jj on the command line?**
JayJay doesn't replace the jj CLI — it complements it. The DAG graph, visual diff review, diff-edit mode, and bookmark manager are significantly easier in a GUI than in a terminal. For anything JayJay doesn't expose, the command palette (`⌘⇧P`) has an `!` prefix that drops you into a raw `jj` CLI prompt in-window.

**Is JayJay open source?**
JayJay is **free and source-available**, not OSI open source. The macOS app is [BSL 1.1](LICENSE) — free to use, fork, modify, and redistribute; the only restriction is paid app-store distribution without permission. It converts to Apache-2.0 on 2030-03-23. The Rust crates (`jayjay-core`, `jayjay-uniffi`, `jj-diff`) are Apache-2.0 today.

**Does JayJay work on Windows or Linux?**
Not yet. Today it's macOS-only. The Rust core (`jayjay-core`) is platform-agnostic — a future Windows or Linux frontend is possible if there's demand.

## License

- **Rust crates** (`crates/`): [Apache-2.0](crates/LICENSE)
- **macOS app** (`shell/`, everything else): [BSL 1.1](LICENSE) — free to use, modify, and redistribute; paid app store distribution requires permission. Converts to Apache-2.0 on 2030-03-23.
