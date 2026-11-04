use std::collections::HashMap;

use jj_diff::{DiffSide, DiffSpan, DiffSpanStyle, RowSide};
use serde_json::{Value, json};

pub(super) fn diff(
    path: &str,
    before: &str,
    after: &str,
    ignore_whitespace: bool,
    show_all: bool,
    split: bool,
) -> Value {
    let compute = if show_all {
        jj_diff::compute_file_diff_full
    } else {
        jj_diff::compute_file_diff
    };
    let diff = compute(path, before, after, ignore_whitespace);
    let display = jj_diff::build_diff_display_lines(&diff.lines);
    let groups = jj_diff::change_groups(&display);
    let mut group_by_line = HashMap::new();
    for group in &groups {
        for line in &display[group.start_line as usize - 1..group.end_line as usize] {
            if let Some(anchor) = jj_diff::anchor_side_and_number(line) {
                group_by_line.insert(anchor, group.index);
            }
        }
    }
    let (added, removed) = jj_diff::count_changed_lines(before, after, ignore_whitespace);
    let eof = |line_no: fn(&jj_diff::DiffLine) -> Option<u32>| {
        diff.lines
            .iter()
            .filter(|line| line.no_eof_newline)
            .find_map(line_no)
            .map(|n| n.to_string())
    };
    let old_eof = eof(|line| line.old_line_no);
    let new_eof = eof(|line| line.new_line_no);
    let rows: Vec<Value> = if split {
        jj_diff::build_side_by_side_rows(&diff.lines)
            .iter()
            .map(|row| {
                json!({
                    "cells": [side(&row.old, old_eof.as_deref(), &group_by_line), side(&row.new, new_eof.as_deref(), &group_by_line)],
                    "fullWidth": row.full_width,
                })
            })
            .collect()
    } else {
        display.iter()
            .map(|line| {
                json!({"cells": [{
                    "oldLine": line.old_line_no,
                    "newLine": line.new_line_no,
                    "style": style(line.style),
                    "spans": spans(&line.spans),
                    "noEofNewline": line.no_eof_newline,
                    "anchor": jj_diff::anchor_side_and_number(line).map(|(side, number)| anchor(side, number, &group_by_line)),
                }], "fullWidth": false})
            })
            .collect()
    };
    json!({
        "language": diff.language,
        "added": added,
        "removed": removed,
        "whitespaceOnlyHidden": diff.whitespace_only_hidden,
        "rows": rows,
    })
}

fn side(side: &RowSide, eof_line: Option<&str>, groups: &HashMap<(DiffSide, u32), u32>) -> Value {
    // Full-width conflict rows put removed lines in the new cell, so the anchor side follows the line's style, not its cell.
    let source = match side.style {
        DiffSpanStyle::Removed => Some(DiffSide::Old),
        DiffSpanStyle::Added => Some(DiffSide::New),
        _ => None,
    };
    json!({
        "oldLine": side.line_no,
        "newLine": null,
        "style": style(side.style),
        "spans": spans(&side.spans),
        "noEofNewline": eof_line == Some(side.line_no.as_str()),
        "anchor": source.zip(side.line_no.parse::<u32>().ok()).map(|(source, number)| anchor(source, number, groups)),
    })
}

fn anchor(side: DiffSide, line: u32, groups: &HashMap<(DiffSide, u32), u32>) -> Value {
    json!({ "side": if side == DiffSide::Old { "old" } else { "new" }, "line": line, "group": groups.get(&(side, line)) })
}

fn spans(spans: &[DiffSpan]) -> Vec<Value> {
    spans
        .iter()
        .map(|span| {
            json!({
                "text": span.text,
                "style": style(span.style),
                "token": format!("{:?}", span.token),
                "tooltip": span.tooltip,
            })
        })
        .collect()
}

fn style(style: DiffSpanStyle) -> &'static str {
    match style {
        DiffSpanStyle::Context => "context",
        DiffSpanStyle::Added => "added",
        DiffSpanStyle::Removed => "removed",
        DiffSpanStyle::Unchanged => "unchanged",
        DiffSpanStyle::Separator => "separator",
    }
}

#[cfg(test)]
mod tests {
    use super::diff;

    #[test]
    fn split_padding_and_eof_survive_the_browser_boundary() {
        let view = diff("a.txt", "old\n", "new\nextra", false, true, true);
        let rows = view["rows"].as_array().unwrap();
        assert_eq!(view["added"], 2);
        assert_eq!(view["removed"], 1);
        assert_eq!(rows[1]["cells"][0]["oldLine"], "");
        assert_eq!(rows[1]["cells"][1]["noEofNewline"], true);
    }

    #[test]
    fn deleted_conflict_lines_anchor_to_the_old_side_in_split_view() {
        let conflict = "<<<<<<< conflict 1 of 1\n%%%%%%% diff from base to side #1\n-a\n+b\n+++++++ side #2\nc\n>>>>>>> conflict 1 of 1 ends\n";
        let view = diff("a.txt", conflict, "", false, true, true);
        let anchors: Vec<_> = view["rows"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|row| {
                row["cells"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|cell| cell["anchor"].clone())
            })
            .filter(|anchor| !anchor.is_null())
            .collect();
        assert!(
            view["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["fullWidth"] == true)
        );
        assert!(!anchors.is_empty());
        assert!(
            anchors
                .iter()
                .all(|anchor| anchor["side"] == "old" && !anchor["group"].is_null())
        );
    }
}
