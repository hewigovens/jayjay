use crate::conflicts::build_diff_display_lines;
use crate::types::{ConflictLineKind, ContextRegion, DiffLine, DiffSpan, DiffSpanStyle};

/// One side (old or new) of a side-by-side row.
#[derive(Debug, Clone)]
pub struct RowSide {
    pub line_no: String,
    pub spans: Vec<DiffSpan>,
    pub style: DiffSpanStyle,
    pub conflict_kind: ConflictLineKind,
}

impl Default for RowSide {
    fn default() -> Self {
        Self {
            line_no: String::new(),
            spans: Vec::new(),
            // Empty/padded sides render as context background.
            style: DiffSpanStyle::Context,
            conflict_kind: ConflictLineKind::None,
        }
    }
}

impl RowSide {
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

/// A row in a side-by-side diff view, pairing old and new sides.
#[derive(Debug, Clone)]
pub struct SideBySideRow {
    pub old: RowSide,
    pub new: RowSide,
    pub full_width: bool,
    pub context_region: Option<ContextRegion>,
}

/// Build side-by-side rows from a unified diff's lines.
/// Pairs consecutive removed+added lines together.
pub fn build_side_by_side_rows(lines: &[DiffLine]) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();
    let display_lines = build_diff_display_lines(lines);
    build_regular_rows(&display_lines, &mut rows);
    rows
}

fn build_regular_rows(lines: &[DiffLine], rows: &mut Vec<SideBySideRow>) {
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if line.conflict_kind != ConflictLineKind::None {
            rows.push(full_width_conflict_row(line));
            i += 1;
            continue;
        }
        match line.style {
            DiffSpanStyle::Context => {
                let line_no = line
                    .old_line_no
                    .or(line.new_line_no)
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                rows.push(side_by_side_row(
                    RowSide {
                        line_no: line_no.clone(),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Context,
                        conflict_kind: line.conflict_kind,
                    },
                    RowSide {
                        line_no: stringify_line_no(line.new_line_no),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Context,
                        conflict_kind: line.conflict_kind,
                    },
                ));
                i += 1;
            }
            DiffSpanStyle::Separator => {
                let separator = RowSide {
                    line_no: String::new(),
                    spans: line.spans.clone(),
                    style: DiffSpanStyle::Separator,
                    conflict_kind: ConflictLineKind::None,
                };
                rows.push(SideBySideRow {
                    old: separator.clone(),
                    new: separator,
                    full_width: false,
                    context_region: line.context_region,
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
                    rows.push(side_by_side_row(
                        row_side_for(removed.get(j).copied(), DiffSpanStyle::Removed, |l| {
                            l.old_line_no
                        }),
                        row_side_for(added.get(j).copied(), DiffSpanStyle::Added, |l| {
                            l.new_line_no
                        }),
                    ));
                }
            }
            DiffSpanStyle::Added => {
                rows.push(side_by_side_row(
                    RowSide::default(),
                    RowSide {
                        line_no: stringify_line_no(line.new_line_no),
                        spans: line.spans.clone(),
                        style: DiffSpanStyle::Added,
                        conflict_kind: line.conflict_kind,
                    },
                ));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
}

fn side_by_side_row(old: RowSide, new: RowSide) -> SideBySideRow {
    let full_width = is_conflict_side(&old) || is_conflict_side(&new);
    SideBySideRow {
        old,
        new,
        full_width,
        context_region: None,
    }
}

fn is_conflict_side(side: &RowSide) -> bool {
    side.conflict_kind != ConflictLineKind::None
}

fn full_width_conflict_row(line: &DiffLine) -> SideBySideRow {
    side_by_side_row(
        RowSide::default(),
        RowSide {
            line_no: stringify_line_no(line.new_line_no.or(line.old_line_no)),
            spans: line.spans.clone(),
            style: line.style,
            conflict_kind: line.conflict_kind,
        },
    )
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
            conflict_kind: l.conflict_kind,
        },
        None => RowSide::default(),
    }
}
