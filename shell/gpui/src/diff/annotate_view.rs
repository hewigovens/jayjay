use std::sync::Arc;

use gpui::{
    AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, UniformListScrollHandle,
    div, px, rgb, uniform_list,
};
use jayjay_core::AnnotationLine;
use jayjay_core::diff::{DiffSpan, highlight_file};

use crate::app::fonts;
use crate::app::theme::{ANNOTATE_PALETTE, Theme};
use crate::ui::primitives::no_scrollbar_gutter;

fn change_color(change_id: &str) -> u32 {
    let bytes = change_id.as_bytes();
    let mut h: u32 = 0;
    for &b in bytes.iter().take(16) {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    ANNOTATE_PALETTE[(h as usize) % ANNOTATE_PALETTE.len()]
}

/// Per-line syntax-token spans for the whole file via core's `highlight_file`
/// (no diff, no repo) — shared with the SwiftUI shell.
fn highlight_lines(path: &str, lines: &[AnnotationLine]) -> Vec<Vec<DiffSpan>> {
    let full_text = lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    highlight_file(path, &full_text)
}

pub fn annotate_body(
    path: String,
    lines: Arc<Vec<AnnotationLine>>,
    theme: Theme,
    scroll: UniformListScrollHandle,
) -> AnyElement {
    let count = lines.len();
    let theme = Arc::new(theme);
    let highlights = Arc::new(highlight_lines(&path, &lines));
    let list = uniform_list(
        "annotate-lines",
        count,
        move |range: std::ops::Range<usize>, _w, _cx| {
            range
                .map(|ix| annotate_row(&lines[ix], highlights.get(ix).map(Vec::as_slice), &theme))
                .collect()
        },
    )
    .track_scroll(&scroll);
    no_scrollbar_gutter(list).h_full().into_any_element()
}

fn annotate_row(line: &AnnotationLine, spans: Option<&[DiffSpan]>, t: &Theme) -> AnyElement {
    // Highlight the shortest-unique prefix; the stripe still groups by change.
    let short_id = line.change_id.id.chars().take(8).collect::<String>();
    let n = (line.change_id.short_len as usize).min(short_id.len());
    let id_prefix = short_id[..n].to_owned();
    let id_rest = short_id[n..].to_owned();
    let author_initials: String = line
        .author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let stripe_color = change_color(&line.change_id);

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(18.))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(18.))
        .child(stripe(stripe_color))
        .child(line_no_cell(line.line_number, t))
        .child(change_cell(id_prefix, id_rest, t))
        .child(author_cell(&author_initials, &line.author, t))
        .child(date_cell(&line.timestamp, t))
        .child(text_cell(&line.text, spans, t))
        .into_any_element()
}

fn stripe(color: u32) -> Div {
    div().flex_none().w(px(3.)).h(px(18.)).bg(rgb(color))
}

fn line_no_cell(line_no: u32, t: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(36.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.diff_gutter_fg))
        .bg(rgb(t.diff_gutter_bg))
        .child(SharedString::from(line_no.to_string()))
}

fn change_cell(id_prefix: String, id_rest: String, t: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(64.))
        .h(px(18.))
        .px(px(4.))
        .flex()
        .flex_row()
        .child(
            div()
                .text_color(rgb(t.change_id_prefix))
                .child(SharedString::from(id_prefix)),
        )
        .child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(id_rest)),
        )
}

fn author_cell(initials: &str, full_name: &str, t: &Theme) -> Div {
    let _ = full_name; // todo: hover tooltip with full name
    div()
        .flex_none()
        .w(px(28.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(initials.to_owned()))
}

fn date_cell(ts: &str, t: &Theme) -> Div {
    let short = ts.chars().take(10).collect::<String>();
    div()
        .flex_none()
        .w(px(72.))
        .h(px(18.))
        .px(px(4.))
        .text_color(rgb(t.fg_faint))
        .child(SharedString::from(short))
}

fn text_cell(text: &str, spans: Option<&[DiffSpan]>, t: &Theme) -> Div {
    let cell = div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(18.))
        .px(px(8.))
        .text_color(rgb(t.fg));

    match spans {
        // Syntax-highlighted: one child div per token run, colored by token.
        Some(spans) if !spans.is_empty() => spans.iter().fold(cell, |cell, span| {
            let color = t.syntax_token_color(span.token).unwrap_or(t.fg);
            cell.child(
                div()
                    .text_color(rgb(color))
                    .child(SharedString::from(span.text.clone())),
            )
        }),
        // No highlight available (e.g. diff line count drifted); plain text.
        _ => cell.child(SharedString::from(text.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::highlight_lines;
    use jayjay_core::AnnotationLine;
    use jayjay_core::diff::syntax::SyntaxToken;

    fn line(text: &str, n: u32) -> AnnotationLine {
        AnnotationLine {
            change_id: jayjay_core::ShortId::new("zzzzzz".to_owned(), 4),
            author: "Ada".to_owned(),
            timestamp: "2024-01-01".to_owned(),
            line_number: n,
            text: text.to_owned(),
        }
    }

    #[test]
    fn highlights_align_with_lines_and_carry_tokens() {
        let lines = vec![
            line("fn main() {", 1),
            line("    let x = 1;", 2),
            line("}", 3),
        ];
        let spans = highlight_lines("main.rs", &lines);

        // One span vec per source line, in order.
        assert_eq!(spans.len(), lines.len());
        // Tree-sitter should classify at least one non-plain token (e.g. `fn`).
        let has_token = spans
            .iter()
            .flatten()
            .any(|s| s.token != SyntaxToken::Plain);
        assert!(has_token, "expected at least one highlighted token");
    }
}
