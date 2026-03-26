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

See [AGENTS.md](AGENTS.md) for development guidelines.
