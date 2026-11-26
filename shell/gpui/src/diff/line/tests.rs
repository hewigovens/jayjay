use jayjay_core::diff::{ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle};
use jayjay_core::syntax::SyntaxToken;

use super::content_row::conflict_label;
use super::*;
use crate::app::theme::Theme;

#[test]
fn conflict_header_rows_use_conflict_theme_and_clean_label() {
    let theme = Theme::light();
    let line = diff_line("<<<<<<< conflict 1 of 1", ConflictLineKind::Start);

    assert_eq!(conflict_label(&line).as_deref(), Some("Conflict 1 of 1"));
    assert_eq!(
        line_bg_color(line.style, line.conflict_kind, &theme),
        theme.diff_conflict_header_bg
    );
    assert_eq!(
        line_text_color(line.style, line.conflict_kind, &theme),
        theme.diff_conflict_header_fg
    );
}

#[test]
fn conflict_content_rows_keep_content_text() {
    let line = diff_line("+line2 FEATURE", ConflictLineKind::Added);

    assert_eq!(conflict_label(&line), None);
}

fn diff_line(text: &str, conflict_kind: ConflictLineKind) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: Some(1),
        style: DiffSpanStyle::Added,
        spans: vec![DiffSpan {
            text: text.to_owned(),
            style: DiffSpanStyle::Added,
            token: SyntaxToken::Plain,
            tooltip: None,
        }],
        conflict_kind,
        no_eof_newline: false,
        context_region: None,
        moved: None,
    }
}

#[test]
fn gutter_fits_the_largest_line_number_of_either_side() {
    let numbered = |old: Option<u32>, new: Option<u32>| DiffLine {
        old_line_no: old,
        new_line_no: new,
        ..diff_line("x", ConflictLineKind::None)
    };
    let advance = gpui::px(7.);
    let short = GutterWidths::new(
        GutterWidths::largest_line_number(&[numbered(Some(9), Some(9))]),
        advance,
    );
    let long = GutterWidths::new(
        GutterWidths::largest_line_number(&[numbered(Some(9999), None), numbered(None, Some(12))]),
        advance,
    );

    assert_eq!(short.number, 2. * 7. + 7.);
    assert_eq!(long.number, 4. * 7. + 7.);
    assert!(long.interactive(true) > long.interactive(false));
}
