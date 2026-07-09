use jayjay_core::diff::{
    ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle, WrappedDiffLine, syntax::SyntaxToken,
};
use jayjay_review::{NoteEntry, NoteSide, NoteStatus, ReviewNoteStatus};

pub(super) fn line(
    style: DiffSpanStyle,
    old_line_no: Option<u32>,
    new_line_no: Option<u32>,
    text: &str,
) -> DiffLine {
    DiffLine {
        old_line_no,
        new_line_no,
        style,
        spans: vec![DiffSpan {
            text: text.to_owned(),
            style,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
    }
}

/// A tiny two-line diff: line 1 unchanged context, line 2 added; short enough that wrap never splits the added line.
pub(super) fn display_lines() -> Vec<DiffLine> {
    vec![
        line(DiffSpanStyle::Context, Some(1), Some(1), "unchanged"),
        line(DiffSpanStyle::Added, None, Some(2), "added line"),
    ]
}

pub(super) fn wrapped(lines: &[DiffLine]) -> Vec<WrappedDiffLine> {
    lines
        .iter()
        .enumerate()
        .map(|(ix, line)| WrappedDiffLine {
            line_ix: ix as u32,
            line_len: line.text().chars().count() as u32,
            col_start: 0,
            col_end: line.text().chars().count() as u32,
            line: line.clone(),
        })
        .collect()
}

pub(super) fn note(
    id: &str,
    side: NoteSide,
    line: u32,
    body: &str,
    status: NoteStatus,
    resolved: bool,
) -> ReviewNoteStatus {
    ReviewNoteStatus {
        note: NoteEntry {
            id: id.to_owned(),
            change_id: "c1".to_owned(),
            path: "a.txt".to_owned(),
            identity: "id-1".to_owned(),
            side,
            line,
            anchor_excerpt: String::new(),
            anchor_context: Vec::new(),
            ignore_whitespace: false,
            body: body.to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
            resolved,
            resolved_at_ms: None,
        },
        status,
        group_index: Some(0),
    }
}
