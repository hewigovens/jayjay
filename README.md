# jayjay

A native GUI for [Jujutsu](https://github.com/jj-vcs/jj) version control.

## Architecture

```
jj-lib (Rust)
  └── uniffi bindings
        ├── SwiftUI app (macOS) ← starting here
        ├── Qt/GTK (Linux)
        └── WinUI (Windows)
```

## Features (planned)

- DAG graph view with revset filtering
- Semantic diff via tree-sitter (difftastic-style)
- Side-by-side diff view
- Change management (describe, new, squash, split, abandon, rebase)
- Bookmark management
- Git push/fetch
- Multi-repo / multi-window support

## Stack

- **Core**: `jj-lib` (Rust crate) — direct library access, no CLI shelling
- **Bindings**: `uniffi` — generates Swift types from Rust
- **Diff**: tree-sitter for AST-level structural diffs
- **macOS UI**: SwiftUI + `WindowGroup` (not document-based)

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

Build and open a specific repo directly:

```bash
just run /path/to/jj/repo
```

Install the `jayjay` launcher into `~/.local/bin`:

```bash
just install-cli
```

Then you can open the current repo directly:

```bash
jayjay .
```

The app icon now uses Xcode's `app.icon` bundle format with a single SVG source at `macos/Resources/app.icon/Assets/jaybird.svg`.
