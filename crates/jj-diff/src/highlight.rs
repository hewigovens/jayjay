use super::compute::compute_file_diff_full;
use super::highlights::apply_highlights;
use super::types::{DiffLine, DiffSpan, DiffSpanStyle, LineIndex};
use crate::syntax;

/// File extensions that are generated/data — skip syntax highlighting.
const SKIP_HIGHLIGHT_EXTENSIONS: &[&str] = &["lock", "csv", "tsv", "svg"];

pub(crate) fn should_skip_highlight(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|extension| SKIP_HIGHLIGHT_EXTENSIONS.contains(&extension))
}

/// Standalone per-line highlight for blame/annotate — do not fold back into a diff-against-empty; that produced Added spans, collapsed context, and EOF markers in blame views.
pub fn highlight_file(path: &str, content: &str) -> Vec<Vec<DiffSpan>> {
    if content.is_empty() {
        return vec![];
    }
    let language = syntax::language_for_path(path);
    let highlights = if should_skip_highlight(path) {
        vec![]
    } else {
        syntax::highlight(content, language)
    };
    let line_index = LineIndex::from_text(content);
    let mut lines = Vec::new();
    let mut n: u32 = 1;
    while let Some((byte_start, text)) = line_index.get(content, n) {
        lines.push(apply_highlights(
            text,
            byte_start,
            &highlights,
            DiffSpanStyle::Context,
        ));
        n += 1;
    }
    lines
}

/// Syntax-highlight every line in `content` while retaining its Base-relative diff style.
/// Removed Base-only lines have no row in the returned source because `content` is the display basis.
pub fn highlight_file_against_base(path: &str, base: &str, content: &str) -> Vec<DiffLine> {
    compute_file_diff_full(path, base, content, false)
        .lines
        .into_iter()
        .filter(|line| line.new_line_no.is_some())
        .collect()
}
