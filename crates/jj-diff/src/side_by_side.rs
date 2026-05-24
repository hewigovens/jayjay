use crate::types::{DiffLine, DiffSpan, DiffSpanStyle};

/// One side (old or new) of a side-by-side row.
#[derive(Debug, Clone)]
pub struct RowSide {
    pub line_no: String,
    pub spans: Vec<DiffSpan>,
    pub style: DiffSpanStyle,
}

impl Default for RowSide {
    fn default() -> Self {
        Self {
            line_no: String::new(),
            spans: Vec::new(),
            // Empty/padded sides render as context background.
            style: DiffSpanStyle::Context,
        }
    }
}

/// A row in a side-by-side diff view, pairing old and new sides.
#[derive(Debug, Clone)]
pub struct SideBySideRow {
    pub old: RowSide,
    pub new: RowSide,
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
                    old: RowSide {
                        line_no: line_no.clone(),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Context,
                    },
                    new: RowSide {
                        line_no: stringify_line_no(line.new_line_no),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Context,
                    },
                });
                i += 1;
            }
            DiffSpanStyle::Separator => {
                let separator = RowSide {
                    line_no: String::new(),
                    spans: line.spans.clone(),
                    style: DiffSpanStyle::Separator,
                };
                rows.push(SideBySideRow {
                    old: separator.clone(),
                    new: separator,
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
                    rows.push(SideBySideRow {
                        old: row_side_for(removed.get(j).copied(), DiffSpanStyle::Removed, |l| {
                            l.old_line_no
                        }),
                        new: row_side_for(added.get(j).copied(), DiffSpanStyle::Added, |l| {
                            l.new_line_no
                        }),
                    });
                }
            }
            DiffSpanStyle::Added => {
                rows.push(SideBySideRow {
                    old: RowSide::default(),
                    new: RowSide {
                        line_no: stringify_line_no(line.new_line_no),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Added,
                    },
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

fn stringify_line_no(n: Option<u32>) -> String {
    n.map(|n| n.to_string()).unwrap_or_default()
}

fn row_side_for(
    line: Option<&DiffLine>,
    style: DiffSpanStyle,
    pick_no: impl FnOnce(&DiffLine) -> Option<u32>,
) -> RowSide {
    match line {
        Some(l) => RowSide {
            line_no: stringify_line_no(pick_no(l)),
            spans: l.spans.clone(),
            style,
        },
        None => RowSide::default(),
    }
}
