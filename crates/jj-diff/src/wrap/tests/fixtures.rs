use crate::side_by_side::RowSide;
use crate::syntax::SyntaxToken;
use crate::types::{ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle};

pub(super) fn diff_line(
    text: &str,
    old_line_no: Option<u32>,
    new_line_no: Option<u32>,
    style: DiffSpanStyle,
) -> DiffLine {
    DiffLine {
        old_line_no,
        new_line_no,
        style,
        spans: vec![span(text, style)],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
    }
}

pub(super) fn conflict_line(
    text: &str,
    style: DiffSpanStyle,
    conflict_kind: ConflictLineKind,
) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: Some(1),
        style,
        spans: vec![span(text, style)],
        conflict_kind,
        no_eof_newline: false,
    }
}

pub(super) fn row_side(line_no: &str, text: &str, style: DiffSpanStyle) -> RowSide {
    RowSide {
        line_no: line_no.to_owned(),
        spans: vec![span(text, style)],
        style,
        conflict_kind: ConflictLineKind::None,
    }
}

pub(super) fn span(text: &str, style: DiffSpanStyle) -> DiffSpan {
    DiffSpan {
        text: text.to_owned(),
        style,
        token: SyntaxToken::Plain,
    }
}
