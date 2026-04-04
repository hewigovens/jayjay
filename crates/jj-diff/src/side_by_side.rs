use crate::types::{DiffLine, DiffSpan, DiffSpanStyle};

/// A row in a side-by-side diff view, pairing old and new sides.
#[derive(Debug, Clone)]
pub struct SideBySideRow {
    pub old_line_no: String,
    pub old_marker: String,
    pub old_spans: Vec<DiffSpan>,
    pub old_style: DiffSpanStyle,
    pub new_line_no: String,
    pub new_marker: String,
    pub new_spans: Vec<DiffSpan>,
    pub new_style: DiffSpanStyle,
}

/// Build side-by-side rows from a unified diff's lines.
/// Pairs consecutive removed+added lines together.
pub fn build_side_by_side_rows(lines: &[DiffLine]) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        match line.style {
            DiffSpanStyle::Context => {
                let line_no = line
                    .old_line_no
                    .or(line.new_line_no)
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                rows.push(SideBySideRow {
                    old_line_no: line_no.clone(),
                    old_marker: " ".to_owned(),
                    old_spans: line.spans.clone(),
                    old_style: DiffSpanStyle::Context,
                    new_line_no: line
                        .new_line_no
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                    new_marker: " ".to_owned(),
                    new_spans: line.spans.clone(),
                    new_style: DiffSpanStyle::Context,
                });
                i += 1;
            }
            DiffSpanStyle::Separator => {
                rows.push(SideBySideRow {
                    old_line_no: String::new(),
                    old_marker: String::new(),
                    old_spans: line.spans.clone(),
                    old_style: DiffSpanStyle::Separator,
                    new_line_no: String::new(),
                    new_marker: String::new(),
                    new_spans: line.spans.clone(),
                    new_style: DiffSpanStyle::Separator,
                });
                i += 1;
            }
            DiffSpanStyle::Removed => {
                let mut removed = Vec::new();
                while i < lines.len() && lines[i].style == DiffSpanStyle::Removed {
                    removed.push(&lines[i]);
                    i += 1;
                }
                let mut added = Vec::new();
                while i < lines.len() && lines[i].style == DiffSpanStyle::Added {
                    added.push(&lines[i]);
                    i += 1;
                }
                let max_len = removed.len().max(added.len());
                for j in 0..max_len {
                    let rem = removed.get(j);
                    let add = added.get(j);
                    rows.push(SideBySideRow {
                        old_line_no: rem
                            .and_then(|l| l.old_line_no)
                            .map(|n| n.to_string())
                            .unwrap_or_default(),
                        old_marker: if rem.is_some() { "-" } else { " " }.to_owned(),
                        old_spans: rem.map(|l| l.spans.clone()).unwrap_or_default(),
                        old_style: if rem.is_some() {
                            DiffSpanStyle::Removed
                        } else {
                            DiffSpanStyle::Context
                        },
                        new_line_no: add
                            .and_then(|l| l.new_line_no)
                            .map(|n| n.to_string())
                            .unwrap_or_default(),
                        new_marker: if add.is_some() { "+" } else { " " }.to_owned(),
                        new_spans: add.map(|l| l.spans.clone()).unwrap_or_default(),
                        new_style: if add.is_some() {
                            DiffSpanStyle::Added
                        } else {
                            DiffSpanStyle::Context
                        },
                    });
                }
            }
            DiffSpanStyle::Added => {
                rows.push(SideBySideRow {
                    old_line_no: String::new(),
                    old_marker: " ".to_owned(),
                    old_spans: Vec::new(),
                    old_style: DiffSpanStyle::Context,
                    new_line_no: line
                        .new_line_no
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                    new_marker: "+".to_owned(),
                    new_spans: line.spans.clone(),
                    new_style: DiffSpanStyle::Added,
                });
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    rows
}
