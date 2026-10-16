use super::moved_lines::MovedLines;
use crate::{DiffSpanStyle, FileDiff, MovedRow, compute_file_diff};

const SMALL: &str = "fn small() {\n    x(1);\n}\n";
const BIG: &str = "fn big() {\n    let a = compute(1, 2);\n    let b = compute(3, 4);\n    let c = compute(5, 6);\n}\n";

fn long_big() -> String {
    let body: String = (0..12)
        .map(|i| format!("    let v{i} = compute({i});\n"))
        .collect();
    format!("fn big() {{\n{body}}}\n")
}

fn detected(path: &str, old: &str, new: &str, ignore_whitespace: bool) -> FileDiff {
    let mut diff = compute_file_diff(path, old, new, ignore_whitespace);
    diff.detect_moves(old, new);
    diff
}

/// Changed rows that carry a move, as (sign, own line, counterpart line, starts block).
fn moved_rows(diff: &FileDiff) -> Vec<(char, u32, u32, bool)> {
    diff.lines
        .iter()
        .filter_map(|line| {
            let MovedRow {
                counterpart_line,
                starts_block,
            } = line.moved?;
            let (sign, own) = if line.style == DiffSpanStyle::Removed {
                ('-', line.old_line_no?)
            } else {
                ('+', line.new_line_no?)
            };
            Some((sign, own, counterpart_line, starts_block))
        })
        .collect()
}

#[test]
fn moved_function_rows_link_to_each_other() {
    let diff = detected(
        "moved.rs",
        &format!("{SMALL}{BIG}"),
        &format!("{BIG}{SMALL}"),
        false,
    );
    assert_eq!(
        moved_rows(&diff),
        [
            ('-', 1, 6, true),
            ('-', 2, 6, false),
            ('-', 3, 6, false),
            ('+', 6, 1, true),
            ('+', 7, 1, false),
            ('+', 8, 1, false)
        ]
    );
}

#[test]
fn moves_follow_the_rows_the_line_diff_shows() {
    let edited = detected(
        "moved.rs",
        &format!("{SMALL}{BIG}"),
        &format!("{}{SMALL}", BIG.replace('4', "9")),
        false,
    );
    assert_eq!(
        moved_rows(&edited),
        [
            ('+', 2, 5, true),
            ('+', 4, 7, true),
            ('-', 5, 2, true),
            ('-', 7, 4, true)
        ]
    );
}

#[test]
fn a_move_whose_closing_brace_the_line_diff_pairs_still_links() {
    let big = long_big();
    let tail = "fn tail() {\n    finish(1);\n}\n";
    let diff = detected(
        "moved.rs",
        &format!("{SMALL}{big}{tail}"),
        &format!("{big}{}{SMALL}", tail.replace('1', "2")),
        false,
    );
    assert_eq!(
        moved_rows(&diff),
        [
            ('-', 1, 18, true),
            ('-', 2, 18, false),
            ('-', 3, 18, false),
            ('+', 18, 1, true),
            ('+', 19, 1, false)
        ]
    );
}

#[test]
fn moves_follow_rows_shown_with_ignore_whitespace() {
    let small = "fn small() {\n    x(1);\n    y(2);\n}\n";
    let other = "fn other() {\n    z(9);\n}\n";
    let old = format!("{small}{BIG}{other}");
    let new = format!("{}{other}{small}", BIG.replace("    ", "\t"));
    for (ignore_whitespace, removed_start, counterpart) in [(false, 5, 1), (true, 1, 9)] {
        let rows = moved_rows(&detected("moved.rs", &old, &new, ignore_whitespace));
        assert!(
            rows.contains(&('-', removed_start, counterpart, true)),
            "ignore_whitespace={ignore_whitespace}: {rows:?}"
        );
    }
}

#[test]
fn moves_are_found_across_grammars() {
    for (path, block, rest) in [
        (
            "moved.py",
            "def small():\n    x(1)\n",
            "def big():\n    a = 1\n    b = 2\n    c = 3\n",
        ),
        (
            "moved.json",
            "  {\"id\": 1, \"name\": \"small\"},\n",
            "  {\"id\": 2, \"name\": \"big\"},\n  {\"id\": 3, \"name\": \"tail\"},\n  {\"id\": 4, \"name\": \"end\"},\n",
        ),
        (
            "moved.swift",
            "func small() {\n    x(1)\n}\n",
            "func big() {\n    let a = 1\n    let b = 2\n    let c = 3\n}\n",
        ),
    ] {
        let (old, new) = if path.ends_with(".json") {
            (
                format!("[\n{block}{rest}  {{}}\n]\n"),
                format!("[\n{rest}{block}  {{}}\n]\n"),
            )
        } else {
            (format!("{block}{rest}"), format!("{rest}{block}"))
        };
        let rows = moved_rows(&detected(path, &old, &new, false));
        assert!(
            rows.iter()
                .any(|&(sign, own, _, starts)| sign == '-' && own <= 2 && starts),
            "{path}: {rows:?}"
        );
    }
}

#[test]
fn nearby_reorders_have_no_moves() {
    let old = "fn f() {\n    alpha(1);\n    beta(2);\n}\n";
    let new = "fn f() {\n    beta(2);\n    alpha(1);\n}\n";
    assert!(compute_file_diff("swap.rs", old, new, false).may_have_moves());
    assert!(moved_rows(&detected("swap.rs", old, new, false)).is_empty());
}

#[test]
fn only_diffs_sharing_text_across_hunks_are_analyzed() {
    let big = long_big();
    for (new, expected) in [
        (format!("{big}{SMALL}"), true),
        (format!("{SMALL}{}", big.replace("v3", "w3")), false),
        (format!("{SMALL}{}", big.replace("    let", "\tlet")), false),
    ] {
        let diff = compute_file_diff("moved.rs", &format!("{SMALL}{big}"), &new, false);
        assert_eq!(diff.may_have_moves(), expected, "{new}");
    }
}

#[test]
fn unanalyzable_inputs_return_quickly_without_moves() {
    use std::time::{Duration, Instant};

    let line_count = 3_000_000;
    let huge = "x\n".repeat(line_count);
    let huge_new = format!("{huge}tail\n");
    let sample = "x\n".repeat(8);
    let mut collapsed = compute_file_diff("large.rs", &sample, &format!("{sample}tail\n"), false);
    for line in &mut collapsed.lines {
        if let Some(region) = &mut line.context_region {
            region.line_count = line_count as u32 - 3;
        } else {
            line.old_line_no = line.old_line_no.map(|n| n + line_count as u32 - 8);
            line.new_line_no = line.new_line_no.map(|n| n + line_count as u32 - 8);
        }
    }
    let nested = format!("{}0{}", "[".repeat(7_900), "]".repeat(7_900));
    let (nested_old, nested_new) = (format!("[{nested}, 1]\n"), format!("[{nested}, 2]\n"));
    let big = long_big();
    let (moved_old, broken_new) = (
        format!("{SMALL}{big}"),
        format!("{big}{SMALL}fn broken( {{}}\n"),
    );
    let others = [
        ("notes.txt", "a\nb\nc\n", "c\nb\na\n"),
        ("broken.rs", moved_old.as_str(), broken_new.as_str()),
        ("nested.json", nested_old.as_str(), nested_new.as_str()),
    ];
    let mut cases = vec![(collapsed, huge.as_str(), huge_new.as_str())];
    cases.extend(others.map(|(path, old, new)| {
        (
            crate::compute_file_diff_full_plain(path, old, new, false),
            old,
            new,
        )
    }));
    for (diff, old, new) in cases {
        let start = Instant::now();
        let found = MovedLines::find(&diff, old, new);
        assert!(
            start.elapsed() < Duration::from_millis(500),
            "{}",
            diff.path
        );
        assert!(found.unwrap_or_default().is_empty(), "{}", diff.path);
    }
}
