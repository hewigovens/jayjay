# jayjay

A native GUI for [Jujutsu](https://github.com/jj-vcs/jj) version control.

## Features

- **DAG graph** — visual change history with branch lines, bookmarks, conflict indicators
- **Native diff** — tree-sitter syntax highlighting for 15 languages, context collapsing, rename detection
- **Unified + side-by-side** — toggle between diff modes, auto-fallback for added/deleted files
- **All jj operations** — new, describe, squash, abandon, rebase, split, restore
- **Git integration** — push, fetch, submodule-aware commit
- **Bookmarks** — create, delete, filter by bookmark
- **Review workflow** — mark files as reviewed (space key), tree view, show in Finder
- **AI commit messages** — on-device via Apple Foundation Models (macOS 26+)
- **Multi-window** — open multiple repos, recent repos menu

## Architecture

```
jj-lib (Rust)
  └── jayjay-core (diff, syntax, repo operations)
       └── uniffi bindings
            └── SwiftUI app (macOS)
```

- **Core**: `jj-lib` — direct library access, no CLI shelling (except git push/fetch for auth)
- **Diff**: tree-sitter for AST-level syntax highlighting, jj-lib for word-level diff
- **Renderer**: Native `NSTextView` + `NSAttributedString` — no WebView
- **macOS UI**: SwiftUI + `WindowGroup`

## Run macOS App

List commands:

```bash
just list
```

Build the app:

```bash
just build
```

Build and launch:

```bash
just run
```

Build and open a specific repo:

```bash
just run /path/to/jj/repo
```

## Install CLI

Build and install the `jayjay` launcher:

```bash
just install-cli
```

Then open any jj repo:

```bash
jayjay .
jayjay /path/to/repo
```

The CLI finds `JayJay.app` in `/Applications` or next to the binary.

## Prerequisites

- Rust 1.85+
- Xcode 16+ with macOS 14+ SDK
- [xcodegen](https://github.com/yonaskolb/XcodeGen)
- [xcbeautify](https://github.com/cpisciotta/xcbeautify)
- [just](https://github.com/casey/just)

## Project Structure

```
crates/
  jayjay-core/     Rust core: jj-lib wrapper, diff engine, tree-sitter
  jayjay-uniffi/   uniffi bindings → Swift
  jayjay-cli/      Native CLI launcher
shell/
  mac/             macOS SwiftUI app
    Sources/JayJay/
      App/           Entry point, settings, window manager
      Views/         Main views, DAG, detail panel
        Diff/        Unified + side-by-side renderers
        Components/  Commit box, bookmarks, settings
```

## License

Apache-2.0
