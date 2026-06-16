use unicode_segmentation::UnicodeSegmentation;

use super::chunks::{SpanChunk, grapheme_cells, side_chunks, spans_char_len};
use super::types::{WrappedSbsRow, WrappedSide};
use crate::conflicts::{build_diff_display_items, build_diff_display_lines};
use crate::side_by_side::{RowSide, SideBySideRow};
use crate::types::{DiffDisplayItem, DiffLine, DiffSpan, DiffSpanStyle};

/// Wrap SBS rows, padding the shorter side so both panes advance in lock-step.
pub fn wrap_sbs_rows(rows: &[SideBySideRow], old_cols: u32, new_cols: u32) -> Vec<WrappedSbsRow> {
    let old_cols = (old_cols.max(1)) as usize;
    let new_cols = (new_cols.max(1)) as usize;
    let mut wrapped = Vec::new();

    for (row_ix, row) in rows.iter().enumerate() {
        let old_len = spans_char_len(&row.old.spans);
        let new_len = spans_char_len(&row.new.spans);
        if row.old.style == DiffSpanStyle::Separator {
            wrapped.push(WrappedSbsRow {
                row_ix: row_ix as u32,
                old: whole_side(old_len),
                new: whole_side(new_len),
                row: row.clone(),
            });
            continue;
        }

        if row.full_width {
            wrap_full_width_row(row_ix as u32, row, old_cols, new_cols, &mut wrapped);
            continue;
        }

        let old_chunks = side_chunks(&row.old.spans, old_cols);
        let new_chunks = side_chunks(&row.new.spans, new_cols);
        let visual_count = old_chunks.len().max(new_chunks.len()).max(1);
        for visual_ix in 0..visual_count {
            let old = old_chunks.get(visual_ix).cloned().unwrap_or_default();
            let new = new_chunks.get(visual_ix).cloned().unwrap_or_default();
            wrapped.push(WrappedSbsRow {
                row_ix: row_ix as u32,
                old: chunk_side(old_len, &old),
                new: chunk_side(new_len, &new),
                row: SideBySideRow {
                    old: continuation_side(&row.old, visual_ix, old.spans),
                    new: continuation_side(&row.new, visual_ix, new.spans),
                    full_width: false,
                },
            });
        }
    }

    wrapped
}

/// First wrapped visual position for `row_ix`; falls back to `row_ix` if absent.
/// O(log N) via `partition_point` — `wrap_sbs_rows` emits non-decreasing `row_ix`.
pub fn visual_index_for_sbs_row(wrapped: &[WrappedSbsRow], row_ix: u32) -> u32 {
    let pos = wrapped.partition_point(|row| row.row_ix < row_ix);
    if pos < wrapped.len() && wrapped[pos].row_ix == row_ix {
        pos as u32
    } else {
        row_ix
    }
}

/// Map each `DiffLine` index to the `SideBySideRow` index that consumes it —
/// mirrors `build_side_by_side_rows`' Removed/Added pairing.
pub fn sbs_line_to_row(lines: &[DiffLine]) -> Vec<u32> {
    let display_lines = build_diff_display_lines(lines);
    let lines = display_lines.as_slice();
    let mut map = vec![0u32; lines.len()];
    let mut row_ix: u32 = 0;
    for item in build_diff_display_items(lines) {
        match item {
            DiffDisplayItem::Lines {
                line_start,
                line_end,
            } => map_regular_lines(
                lines,
                line_start as usize,
                line_end as usize,
                &mut map,
                &mut row_ix,
            ),
            DiffDisplayItem::ConflictBlock { block } => {
                for line_ix in block.line_start..block.line_end {
                    map[line_ix as usize] = row_ix;
                    row_ix += 1;
                }
            }
        }
    }
    map
}

fn map_regular_lines(
    lines: &[DiffLine],
    start: usize,
    end: usize,
    map: &mut [u32],
    row_ix: &mut u32,
) {
    let mut i = start;
    while i < end {
        if lines[i].conflict_kind != crate::types::ConflictLineKind::None {
            map[i] = *row_ix;
            *row_ix += 1;
            i += 1;
            continue;
        }
        match lines[i].style {
            DiffSpanStyle::Context | DiffSpanStyle::Separator => {
                map[i] = *row_ix;
                *row_ix += 1;
                i += 1;
            }
            // Mirrors the wildcard skip in `build_side_by_side_rows`.
            DiffSpanStyle::Unchanged => {
                i += 1;
            }
            DiffSpanStyle::Removed => {
                let rem_start = i;
                while i < end && lines[i].style == DiffSpanStyle::Removed {
                    i += 1;
                }
                let rem_end = i;
                let add_start = i;
                while i < end && lines[i].style == DiffSpanStyle::Added {
                    i += 1;
                }
                let add_end = i;
                let rem_count = rem_end - rem_start;
                let add_count = add_end - add_start;
                let pair_count = rem_count.max(add_count);
                for j in 0..rem_count {
                    map[rem_start + j] = *row_ix + j as u32;
                }
                for j in 0..add_count {
                    map[add_start + j] = *row_ix + j as u32;
                }
                *row_ix += pair_count as u32;
            }
            DiffSpanStyle::Added => {
                map[i] = *row_ix;
                *row_ix += 1;
                i += 1;
            }
        }
    }
}

/// Per-side payload for one visual row: keep `line_no` only on the first row;
/// continuation rows get an empty number but the same style.
fn continuation_side(
    source: &RowSide,
    visual_ix: usize,
    spans: Vec<crate::types::DiffSpan>,
) -> RowSide {
    RowSide {
        line_no: if visual_ix == 0 {
            source.line_no.clone()
        } else {
            String::new()
        },
        spans,
        style: source.style,
        conflict_kind: source.conflict_kind,
    }
}

fn wrap_full_width_row(
    row_ix: u32,
    row: &SideBySideRow,
    old_cols: usize,
    new_cols: usize,
    wrapped: &mut Vec<WrappedSbsRow>,
) {
    let source = full_width_source(row);
    let len = spans_char_len(&source.spans);
    let cols = old_cols.saturating_add(new_cols).max(1);
    let chunks = side_chunks(&source.spans, cols);
    for chunk in chunks {
        let split = chunk.start.saturating_add(old_cols).min(chunk.end);
        let old_spans = spans_range(&source.spans, chunk.start, split);
        let new_spans = spans_range(&source.spans, split, chunk.end);
        wrapped.push(WrappedSbsRow {
            row_ix,
            old: range_side(len, chunk.start, split),
            new: range_side(len, split, chunk.end),
            row: SideBySideRow {
                old: full_width_side(source, old_spans),
                new: full_width_side(source, new_spans),
                full_width: true,
            },
        });
    }
}

fn full_width_source(row: &SideBySideRow) -> &RowSide {
    if row.new.conflict_kind != crate::types::ConflictLineKind::None || !row.new.spans.is_empty() {
        &row.new
    } else {
        &row.old
    }
}

fn full_width_side(source: &RowSide, spans: Vec<DiffSpan>) -> RowSide {
    RowSide {
        line_no: String::new(),
        spans,
        style: source.style,
        conflict_kind: source.conflict_kind,
    }
}

/// Slice `spans` to display-cell range `[start, end)`, keeping clusters whole: a
/// straddling glyph is assigned by its first cell so the two sides never split it.
fn spans_range(spans: &[DiffSpan], start: usize, end: usize) -> Vec<DiffSpan> {
    if start >= end {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut pos = 0usize;
    for span in spans {
        let mut text = String::new();
        for g in span.text.graphemes(true) {
            let w = grapheme_cells(g);
            if pos >= start && pos < end {
                text.push_str(g);
            }
            pos += w;
        }
        if !text.is_empty() {
            out.push(DiffSpan {
                text,
                style: span.style,
                token: span.token,
            });
        }
    }
    out
}

fn whole_side(len: usize) -> WrappedSide {
    WrappedSide {
        line_len: len as u32,
        col_start: 0,
        col_end: len as u32,
    }
}

fn range_side(len: usize, start: usize, end: usize) -> WrappedSide {
    WrappedSide {
        line_len: len as u32,
        col_start: start as u32,
        col_end: end as u32,
    }
}

fn chunk_side(len: usize, chunk: &SpanChunk) -> WrappedSide {
    WrappedSide {
        line_len: len as u32,
        col_start: chunk.start as u32,
        col_end: chunk.end as u32,
    }
}
