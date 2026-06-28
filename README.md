# JayJay

**A native macOS GUI for [Jujutsu](https://github.com/jj-vcs/jj) (jj)** — fast, keyboard-driven, built with Rust + SwiftUI.

Browse your DAG, review side-by-side diffs, resolve conflicts, and run every jj operation from one window. JayJay is a full-featured **jj GUI** with DAG visualization, interdiff (PR-style revision comparison), diff edit mode, file annotate, one-click conflict resolution, AI-generated commit messages, and a command palette.

> **Two shells share a Rust core:** the **SwiftUI shell** (stable, macOS-only) is what ships in releases. A second **GPUI shell** (alpha, cross-platform target — Linux + Windows + macOS) is in active development with read parity and early write actions. See [UserGuide.md](UserGuide.md) for a feature walkthrough, [Roadmap.md](Roadmap.md) for milestone status, and [Build from source](#install) below to try it.

[![CI](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/jayjay/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hewigovens/jayjay?include_prereleases)](https://github.com/hewigovens/jayjay/releases)
![macOS](https://img.shields.io/badge/macOS-26-blue)
![Rust](https://img.shields.io/badge/rust-1.93%2B-orange)
![License](https://img.shields.io/badge/license-BSL--1.1-green)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/jayjay)

## Features

**History & Graph**
- DAG visualization with lane-based fork/merge rendering
- Bookmark, tag, and conflict indicators on every node (bookmark/tag chips with glyphs)
- Each node shows the author avatar, relative time, and the shortest unique change-id prefix highlighted
- Revset filtering with preset chips (All, Mine, Bookmarks, Trunk, Conflicts, Heads)
- Change evolution viewer (`jj evolog`) — see every prior version of a rewritten change with interdiff against current; right-click to copy commit-id or `jj restore` command for recovery
- Load more: incrementally load older history
- Auto-refresh via file system watcher

**Diff & Review**
- Unified + side-by-side diff modes (toggle with one click)
- Histogram diff algorithm (via [`jj-diff`](#jj-diff) crate) — instant diffs even for large files
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
- Drag a bookmark chip onto a change in the DAG to move it there (backward moves allowed, undoable); drag the working-copy `@` chip to move the working copy (`jj edit`). A one-click **Push** follow-up appears after moving a remote-tracking bookmark
- Pull Request on GitHub/GitLab/Codeberg from bookmark right-click (DAG row + Bookmark Manager) — opens an existing PR/MR when one exists, else a compose URL; PR/MR state and CI status are shown via the GitHub `gh` CLI, GitLab REST API, or Codeberg's Forgejo API
- **Stacked PRs/MRs** — right-click a change → **Create / Update Stacked PRs** to open one PR (GitHub) or MR (GitLab) per change in a linear stack, each targeting the one below it. Branch names are editable in the preview and can be auto-suggested with Apple Intelligence; re-running updates the existing PRs. Needs the `gh` or `glab` CLI
- Divergent commit detection and resolution
- Undo via operation log
- Command palette (⌘⇧P) with ~35 commands; type `jj <args>` (or `! <args>`) for inline raw jj CLI output

**AI Commit Messages**
- Codex CLI, Claude CLI, Apple Intelligence fallback chain

**Tools & Settings**
- External editor integration (VS Code, VSCodium, Cursor, Zed, Xcode, Vim + auto-detection)
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
  <img src="docs/imgs/home.webp" width="100%" alt="JayJay — DAG graph and side-by-side diff">
</p>

<p align="center">
  <img src="docs/imgs/stacked-pr.webp" width="100%" alt="JayJay — stacked pull requests preview">
</p>

<p align="center">
  <img src="docs/imgs/diff-edit.webp" width="100%" alt="JayJay — diff edit and split">
</p>

See the [full feature guide](https://jayjay.hewig.dev/guide.html) for everything else.

## Install

**Homebrew**:
```bash
brew install --cask hewigovens/tap/jayjay
```
Homebrew 6.0 trusts the cask the first time you install it.

**Download**: Grab the latest release from [GitHub Releases](https://github.com/hewigovens/jayjay/releases/latest), unzip, and move to Applications.

**Build from source** (SwiftUI shell, macOS):
```bash
just run               # Build and run
just install-cli       # Install CLI launcher to ~/.local/bin
jayjay .               # Open current repo
```

**Build the GPUI shell (alpha, cross-platform)**:
```bash
just gpui              # Build, ad-hoc sign, launch against current dir
just gpui /path/repo   # Open a specific repo
just gpui-bundle       # Build the .app without launching
```
The GPUI shell uses [Zed's GPUI 1.0](https://github.com/zed-industries/zed). It has read parity for DAG, diffs, annotate, file history, evolog, FS-watcher auto-refresh, persistent file review, bookmark / workspace pickers, plus early write actions for describe and the commit box. More write actions are tracked in [Roadmap.md](Roadmap.md).

**Build the GPUI AppImage (alpha, Linux)**:
```bash
just gpui-appimage
```
The AppImage is built from the GPUI shell and includes Linux desktop metadata plus hicolor icons for launcher integration. In a plain Git checkout, `nix build .#appimage` also works directly.

**Auto-update**: JayJay checks for updates automatically via Sparkle. You can also check manually from JayJay → Check for Updates. Auto-update may require App Management permission in System Settings → Privacy & Security.

**Requirements**: macOS 26 or later.

## Roadmap

See [UserGuide.md](UserGuide.md) for shipped features and [Roadmap.md](Roadmap.md) for current plans.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture notes, development workflow, and the current `jj-lib` vs `jj` CLI backend split.

## jj-diff

The diff engine is extracted as a standalone Rust crate with **zero dependency on jj-lib**:

```toml
[dependencies]
jj-diff = { git = "https://github.com/hewigovens/jayjay" }
```

**Features:**
- Histogram line diff (via `similar` 3.0)
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

The full FAQ lives on the site: **[jayjay.hewig.dev/#faq](https://jayjay.hewig.dev/#faq)** — what JayJay is, install, interdiff, side-by-side diffs, diff edit, how it differs from the CLI, licensing, and platform support.

## License

- **Rust crates** (`crates/`): [Apache-2.0](crates/LICENSE)
- **macOS app** (`shell/`, everything else): [BSL 1.1](LICENSE) — free to use, modify, and redistribute; paid app store distribution requires permission. Converts to Apache-2.0 on 2030-03-23.
