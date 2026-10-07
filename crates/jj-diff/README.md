# jj-diff

Diff engine behind [JayJay](https://github.com/hewigovens/jayjay), a native GUI for Jujutsu. It turns two versions of a file into display-ready lines: histogram line matching, word-level changes, tree-sitter syntax spans, collapsed context, side-by-side pairing, and soft wrapping. It has no dependency on jj-lib or any UI toolkit.

## Usage

```rust
use jj_diff::{build_side_by_side_rows, collapse_context_with_mapping, compute_file_diff_full};

let old = "fn main() {\n    println!(\"hello\");\n}\n";
let new = "fn main() {\n    println!(\"hello, world\");\n}\n";

// Every line, with word-level and syntax spans.
let diff = compute_file_diff_full("src/main.rs", old, new, false);
assert_eq!(diff.language, "rust");

// Unchanged runs fold into separator lines; `display_to_full` maps back to the full diff.
let collapsed = collapse_context_with_mapping(&diff);

// Removed and added lines paired for a two-column view.
for row in build_side_by_side_rows(&collapsed.diff.lines) {
    let _ = (&row.old.spans, &row.new.spans, row.old.style, row.new.style);
}
```

Each `DiffLine` carries its old and new line numbers, a `DiffSpanStyle` (added, removed, context, separator, …), and `DiffSpan`s that pair text with a word-diff style and a `SyntaxToken`. Rendering is left to the caller.

## API

| Item | Purpose |
|---|---|
| `compute_file_diff(path, old, new, ignore_whitespace)` | Diff collapsed to 3 lines of context |
| `compute_file_diff_full` / `compute_file_diff_full_plain` | Every line, with or without syntax highlighting |
| `collapse_context_with_mapping(&diff)` | Collapse context and keep a display → full line mapping |
| `ExpandableDiff`, `ContextExpansionSession` | Reveal collapsed context incrementally |
| `build_side_by_side_rows(&lines)` | Pair lines into `SideBySideRow`s |
| `wrap_diff_lines`, `wrap_sbs_rows` | Soft-wrap unified or side-by-side lines to a column width |
| `highlight_file`, `highlight_file_against_base` | Syntax spans for a whole file |
| `count_changed_lines(old, new, ignore_whitespace)` | Insertion and deletion counts |
| `change_groups`, `canonical_review_snapshot` | Hunk grouping and stable fingerprints for review state |
| `is_git_lfs`, `is_git_submodule` | Placeholder detection for LFS pointers and submodules |

## Languages

Each grammar sits behind a `lang-*` feature, and all of them are on by default through `all-languages`. Paths without an enabled grammar diff as plain text. For a smaller build, turn off the defaults and pick the grammars you need:

```toml
jj-diff = { version = "0.1", default-features = false, features = ["lang-rust", "lang-markdown"] }
```

| Feature | Files |
|---|---|
| `lang-bash` | `.sh`, `.bash`, `.zsh` |
| `lang-c` | `.c`, `.h` |
| `lang-cpp` | `.cpp`, `.cc`, `.cxx`, `.hpp` |
| `lang-csharp` | `.cs` |
| `lang-css` | `.css`, `.scss` |
| `lang-fish` | `.fish` |
| `lang-go` | `.go` |
| `lang-html` | `.html`, `.htm` |
| `lang-java` | `.java` |
| `lang-javascript` | `.js`, `.cjs`, `.mjs`, `.jsx` |
| `lang-json` | `.json` |
| `lang-kotlin` | `.kt`, `.kts` |
| `lang-make` | `Makefile`, `.mk` |
| `lang-markdown` | `.md`, `.markdown` |
| `lang-nix` | `.nix` |
| `lang-php` | `.php`, `.phtml` |
| `lang-python` | `.py` |
| `lang-ruby` | `.rb` |
| `lang-rust` | `.rs` |
| `lang-solidity` | `.sol` |
| `lang-sql` | `.sql` |
| `lang-swift` | `.swift` |
| `lang-toml` | `.toml` |
| `lang-typescript` | `.ts`, `.tsx` |
| `lang-xml` | `.xml` |
| `lang-yaml` | `.yaml`, `.yml` |
| `lang-zig` | `.zig` |

## License

Apache-2.0
