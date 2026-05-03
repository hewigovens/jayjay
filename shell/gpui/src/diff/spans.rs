use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::diff::syntax::SyntaxToken;
use jayjay_core::diff::{DiffSpan, DiffSpanStyle};

use crate::app::theme::Theme;

/// Render one DiffSpan, optionally splitting on a case-insensitive `query`
/// match so we can highlight find results inline.
pub fn span_element(
    span: &DiffSpan,
    base_fg: u32,
    line_style: DiffSpanStyle,
    theme: &Theme,
    find_query: Option<&str>,
) -> AnyElement {
    let color = token_color(span.token, theme).unwrap_or(base_fg);
    let bg = word_bg(line_style, span.style, theme);

    let parts = match find_query.filter(|q| !q.is_empty()) {
        Some(q) => split_query(&span.text, q),
        None => vec![(span.text.clone(), false)],
    };

    if parts.len() == 1 && !parts[0].1 {
        // Common case: no matches in this span, render as a single div.
        let mut el = div()
            .h(px(18.))
            .line_height(px(18.))
            .text_color(rgb(color))
            .child(SharedString::from(parts.into_iter().next().unwrap().0));
        if let Some(word_bg) = bg {
            el = el.bg(rgb(word_bg));
        }
        return el.into_any_element();
    }

    let mut row = div().flex().flex_row().h(px(18.)).line_height(px(18.));
    if let Some(word_bg) = bg {
        row = row.bg(rgb(word_bg));
    }
    for (chunk, is_match) in parts {
        let mut child = div()
            .text_color(rgb(color))
            .child(SharedString::from(chunk));
        if is_match {
            child = child
                .bg(rgb(theme.find_match_bg))
                .text_color(rgb(theme.find_match_fg));
        }
        row = row.child(child);
    }
    row.into_any_element()
}

/// Splits `text` into runs alternating between non-match and match parts
/// (case-insensitive). Unicode-safe: `text.to_lowercase()` may change byte
/// length per character (e.g. Turkish `İ` → `i\u{307}`), so we maintain a
/// parallel mapping from each byte in the lowered string back to a char
/// boundary in the original `text` and slice using those.
fn split_query(text: &str, query: &str) -> Vec<(String, bool)> {
    let lower_q = query.to_lowercase();
    if lower_q.is_empty() {
        return vec![(text.to_owned(), false)];
    }

    let mut lower_text = String::with_capacity(text.len());
    // For each byte in `lower_text`, record the byte offset in `text` of
    // the source character. The trailing sentinel maps the end of
    // `lower_text` to `text.len()`.
    let mut lower_to_orig: Vec<usize> = Vec::with_capacity(text.len() + 1);
    for (orig_off, ch) in text.char_indices() {
        for lower_ch in ch.to_lowercase() {
            for _ in 0..lower_ch.len_utf8() {
                lower_to_orig.push(orig_off);
            }
            lower_text.push(lower_ch);
        }
    }
    lower_to_orig.push(text.len());

    let mut parts = Vec::new();
    let mut last_orig_end = 0usize;
    let mut lower_cursor = 0usize;
    while lower_cursor < lower_text.len() {
        match lower_text[lower_cursor..].find(&lower_q) {
            Some(rel) => {
                let lower_match_start = lower_cursor + rel;
                let lower_match_end = lower_match_start + lower_q.len();
                let orig_match_start = lower_to_orig[lower_match_start];
                let orig_match_end = lower_to_orig[lower_match_end];
                if orig_match_start > last_orig_end {
                    parts.push((text[last_orig_end..orig_match_start].to_owned(), false));
                }
                parts.push((text[orig_match_start..orig_match_end].to_owned(), true));
                last_orig_end = orig_match_end;
                lower_cursor = lower_match_end;
            }
            None => break,
        }
    }
    if last_orig_end < text.len() {
        parts.push((text[last_orig_end..].to_owned(), false));
    }
    if parts.is_empty() {
        parts.push((text.to_owned(), false));
    }
    parts
}

pub fn token_color(token: SyntaxToken, theme: &Theme) -> Option<u32> {
    match token {
        SyntaxToken::Keyword | SyntaxToken::Operator => Some(theme.tok_keyword),
        SyntaxToken::StringLit => Some(theme.tok_string),
        SyntaxToken::Comment => Some(theme.tok_comment),
        SyntaxToken::Number => Some(theme.tok_number),
        SyntaxToken::Type | SyntaxToken::Function | SyntaxToken::Attribute => Some(theme.tok_type),
        SyntaxToken::Plain | SyntaxToken::Variable | SyntaxToken::Punctuation => None,
    }
}

pub fn word_bg(line_style: DiffSpanStyle, span_style: DiffSpanStyle, theme: &Theme) -> Option<u32> {
    match (line_style, span_style) {
        (DiffSpanStyle::Added, DiffSpanStyle::Added) => Some(theme.diff_added_word_bg),
        (DiffSpanStyle::Removed, DiffSpanStyle::Removed) => Some(theme.diff_removed_word_bg),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::split_query;

    #[test]
    fn ascii_match() {
        let parts = split_query("Hello World", "world");
        assert_eq!(
            parts,
            vec![("Hello ".to_owned(), false), ("World".to_owned(), true)]
        );
    }

    #[test]
    fn no_match_returns_whole_text() {
        let parts = split_query("Hello", "xyz");
        assert_eq!(parts, vec![("Hello".to_owned(), false)]);
    }

    #[test]
    fn turkish_capital_i_does_not_panic() {
        // İ (U+0130) lowercases to "i\u{307}" — different byte length than
        // the original. Older code panicked on the byte mismatch; this
        // should now just not crash and slice on char boundaries.
        let _ = split_query("İstanbul", "i");
        let _ = split_query("İstanbul", "is");
    }

    #[test]
    fn multibyte_mid_string() {
        let parts = split_query("café Café", "café");
        assert_eq!(
            parts,
            vec![
                ("café".to_owned(), true),
                (" ".to_owned(), false),
                ("Café".to_owned(), true),
            ]
        );
    }
}
