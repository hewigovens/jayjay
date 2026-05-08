# jj-diff

Fast diff engine with histogram line matching, syntax highlighting, and context collapsing. Extracted from [JayJay](https://github.com/hewigovens/jayjay).

**Zero dependency on jj-lib** — usable in any Rust project that needs diff rendering.

## Features

- **Histogram line diff** via `similar` — matches `jj diff` and reads well on code
- **Word-level diff** highlighting within changed lines
- **Syntax highlighting** — tree-sitter with 18 languages
- **Context collapsing** with display-to-full line index mapping and tiny-gap auto-expansion
- **Side-by-side row building** — pairs removed/added lines for two-column rendering
- **Placeholder detection** — Git LFS, submodule, binary file detection
- **Skip highlighting** for `.lock`/`.csv`/`.svg` files

## Usage

```rust
use jj_diff::{compute_file_diff, collapse_context_with_mapping, build_side_by_side_rows};

// Compute diff
let diff = compute_file_diff("main.rs", &old_content, &new_content, false);

// Collapse context for display
let collapsed = collapse_context_with_mapping(&diff);

// Build side-by-side rows
let rows = build_side_by_side_rows(&collapsed.diff.lines);
for row in &rows {
    // row.old_spans, row.new_spans, row.old_style, row.new_style
}
```

## API

| Function | Description |
|---|---|
| `compute_file_diff(path, old, new, ignore_ws)` | Collapsed diff (3 lines context) |
| `compute_file_diff_full(path, old, new, ignore_ws)` | Full diff (all lines, for editing) |
| `collapse_context_with_mapping(diff)` | Collapse with display→full line mapping |
| `build_side_by_side_rows(lines)` | Pair lines into `SideBySideRow` for two-column view |
| `is_editable_text(text)` | Check if content is editable (not binary/LFS) |
| `is_git_lfs(text)` / `is_git_submodule(text)` | Placeholder detection |
| `highlight(source, language)` | tree-sitter syntax highlighting |
| `language_for_path(path)` | Detect language from file extension |

## Supported Languages

Bash, C, C++, CSS, Go, HTML, Java, JavaScript, JSON, Markdown, Python, Ruby, Rust, Swift, TOML, TypeScript, YAML

## Platform Support

### What's in this crate (cross-platform, pure Rust)

Everything needed to compute and structure diffs:

- Diff computation (histogram algorithm)
- Word-level diff within lines
- Syntax highlighting (tree-sitter)
- Context collapsing with index mapping
- Side-by-side row pairing
- Placeholder/binary file detection
- All diff types (`FileDiff`, `DiffLine`, `DiffSpan`, `SideBySideRow`, etc.)

### What platform clients need to provide

The rendering layer is platform-specific:

#### macOS (done — JayJayDiffUI Swift package)

- `NativeDiffView` — NSTextView-based unified diff with gutter, find bar, line selection
- `SideBySideDiffView` — Two NSTextViews with synchronized scrolling
- `DiffGutterTextView` — Line numbers, +/- markers, checkbox support
- `DiffColors` — NSColor theme (dark/light)
- `DiffStore` — SwiftUI @Observable cache + preloading via JayJayCore FFI

#### Linux (planned — GTK via gtk-rs)

Needed:
- `GtkDiffView` — GtkTextView or GtkSourceView-based unified diff renderer
- `GtkSideBySideDiffView` — Two GtkTextViews with scroll sync (GtkScrolledWindow)
- Gutter widget — line numbers + markers (GtkTextView gutter or custom DrawingArea)
- Color theme — map `DiffColors` values to GdkRGBA
- Text tags — map `DiffSpanStyle` → GtkTextTag for background/foreground colors
- Font handling — monospace font configuration

Can reuse directly from Rust (no FFI needed):
- `compute_file_diff` / `compute_file_diff_full`
- `collapse_context_with_mapping`
- `build_side_by_side_rows`
- `highlight` / `language_for_path`
- All placeholder detection
- All diff types

Estimated scope: ~500-800 lines of GTK widget code (vs ~1500 lines macOS AppKit).

#### iOS (future — UIKit/SwiftUI)

Needed:
- `UIKitDiffView` — UITextView-based renderer (or pure SwiftUI Text with AttributedString)
- Side-by-side — horizontal UIScrollView with two UITextViews
- Touch-friendly gutter — larger tap targets, no right-click context menu
- `UIColor` theme mapping

Can reuse via UniFFI (same as macOS):
- All jj-diff functions via JayJayCore FFI bridge
- DiffStore (SwiftUI @Observable — works on iOS)
- DiffPlaceholders, SideBySideRow types

Estimated scope: ~800-1000 lines of UIKit view code. Could share `DiffColors` and `DiffStore` with macOS via the JayJayDiffUI package if made cross-platform with `#if os()` guards.

## License

Apache-2.0
