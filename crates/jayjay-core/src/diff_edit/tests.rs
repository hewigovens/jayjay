use jayjay_primitives::{DiffEditDestination, FileDiffStats};

use crate::mock::{diff_edit_file, file_diff_stats, strings};

use super::*;

#[test]
fn line_hunk_and_file_toggles_only_move_changed_rows() {
    let mut session = DiffEditSession::default();
    session.load(diff_edit_file("a", &[2, 3, 7]));

    session.toggle_line("a", 2);
    session.toggle_line("a", 4);
    assert_eq!(session.selected_lines("a"), [2], "row 4 never changed");
    assert_eq!(session.checkbox("a"), DiffEditCheckbox::Some);

    session.select_lines("a", &[3, 4, 5]);
    assert_eq!(session.selected_lines("a"), [2, 3]);

    session.toggle_file("a");
    assert_eq!(session.selected_lines("a"), [2, 3, 7]);
    assert_eq!(session.checkbox("a"), DiffEditCheckbox::All);
    assert_eq!(
        session.file_counts("a"),
        DiffEditFileCounts {
            selected: 3,
            changed: 3
        }
    );

    session.toggle_file("a");
    assert!(session.selected_lines("a").is_empty());
    assert_eq!(session.checkbox("a"), DiffEditCheckbox::None);
}

#[test]
fn reloading_a_file_keeps_only_the_rows_that_still_changed() {
    let mut session = DiffEditSession::default();
    session.load(diff_edit_file("a", &[2, 3, 7]));
    session.select_file("a");

    session.load(diff_edit_file("a", &[3, 9]));

    assert_eq!(
        session.selected_lines("a"),
        [3],
        "rows the reload dropped must not reach apply"
    );
}

#[test]
fn select_all_claims_files_as_they_load_until_deselected() {
    let mut session = DiffEditSession::default();
    session.load(diff_edit_file("a", &[1]));

    let pending = session.toggle_all(&strings(&["a", "b", "c"]));

    assert_eq!(pending, strings(&["b", "c"]));
    assert!(session.is_selecting_all());
    assert_eq!(session.selected_lines("a"), [1], "loaded files select now");

    session.load(diff_edit_file("b", &[4, 5]));
    assert_eq!(session.selected_lines("b"), [4, 5]);
    session.skip("c");
    assert!(
        !session.is_selecting_all(),
        "an unsupported card must not hold Select All open"
    );

    assert!(session.should_deselect());
    assert!(session.toggle_all(&strings(&["a", "b"])).is_empty());
    assert!(!session.has_selection());
}

#[test]
fn summary_counts_only_files_with_selected_rows() {
    let mut session = DiffEditSession::default();
    session.load(diff_edit_file("a", &[2, 3]));
    session.load(diff_edit_file("b", &[5]));
    assert_eq!(
        session.summary_text(),
        "Select files, hunks, or line ranges to edit"
    );

    session.toggle_line("a", 2);
    assert_eq!(
        session.summary(),
        DiffEditSelectionSummary { files: 1, lines: 1 }
    );
    assert_eq!(session.summary_text(), "1 file, 1 line selected");

    session.select_file("b");
    session.toggle_line("a", 3);
    assert_eq!(
        session.summary(),
        DiffEditSelectionSummary { files: 2, lines: 3 }
    );
    assert_eq!(session.summary_text(), "2 files, 3 lines selected");
}

#[test]
fn apply_builds_card_ordered_ranges_and_inverts_for_remove_from_source() {
    let mut session = DiffEditSession::default();
    session.load(diff_edit_file("a", &[2, 3, 4, 9]));
    session.load(diff_edit_file("b", &[6]));
    session.select_lines("a", &[2, 3, 4]);

    let ordered = strings(&["b", "a"]);
    let selections = session.selections(&ordered, DiffEditDestination::NewChild);
    assert_eq!(
        selections
            .iter()
            .map(|s| s.path.as_str())
            .collect::<Vec<_>>(),
        ["a"],
        "a file with nothing selected is left out"
    );
    assert_eq!(
        selections[0]
            .line_ranges
            .iter()
            .map(|range| (range.start_line, range.end_line))
            .collect::<Vec<_>>(),
        [(2, 4)]
    );

    let inverse = session.selections(&ordered, DiffEditDestination::RemoveFromSource);
    assert_eq!(
        inverse.iter().map(|s| s.path.as_str()).collect::<Vec<_>>(),
        ["b", "a"],
        "keeping the unselected rows makes every loaded file a target"
    );
    assert_eq!(
        inverse[1]
            .line_ranges
            .iter()
            .map(|range| (range.start_line, range.end_line))
            .collect::<Vec<_>>(),
        [(9, 9)]
    );
}

#[test]
fn collapse_seeds_from_the_aggregate_then_the_per_file_pass_replaces_it() {
    let cards = strings(
        &(1..=40)
            .map(|ix| format!("f{ix}"))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let mut session = DiffEditSession::default();

    session.seed_collapse(&cards, None);
    assert!(
        session.is_collapsed("f1"),
        "stats-pending seed folds everything"
    );

    let stats: Vec<FileDiffStats> = cards
        .iter()
        .map(|path| file_diff_stats(path, if path == "f1" { 400 } else { 1 }))
        .collect();
    session.apply_stats(&cards, &stats);
    assert!(
        session.is_collapsed("f1"),
        "the one large file stays folded"
    );
    assert!(!session.is_collapsed("f2"), "small files open again");
}

#[test]
fn the_per_file_pass_counts_cards_the_tree_diff_never_reported() {
    let cards = strings(&["a", "b", "c"]);
    let mut session = DiffEditSession::default();

    session.apply_stats(&cards, &[file_diff_stats("a", 2000)]);
    assert!(
        !session.is_collapsed("a"),
        "three cards stay open however large one of them is"
    );

    session.toggle_collapse("b");
    session.apply_stats(&cards, &[file_diff_stats("a", 2000)]);
    assert!(
        session.is_collapsed("b") && !session.is_collapsed("a"),
        "a hand-folded card freezes the automatic policy"
    );
}

#[test]
fn setting_a_collapse_state_reports_whether_it_moved() {
    let mut session = DiffEditSession::default();

    assert!(session.set_collapsed("a", true));
    assert!(!session.set_collapsed("a", true));
    assert!(session.set_collapsed("a", false));

    session.collapse_all(&strings(&["a", "b"]));
    assert!(session.is_collapsed("a") && session.is_collapsed("b"));
    session.expand_all();
    assert!(!session.is_collapsed("a"));
}

#[test]
fn focus_enters_from_the_near_end_and_clamps_at_the_far_one() {
    let cards = strings(&["a", "b", "c"]);
    let mut session = DiffEditSession::default();

    assert_eq!(session.move_focus(&cards, true), Some("a"));
    assert_eq!(session.move_focus(&cards, true), Some("b"));
    assert_eq!(session.move_focus(&cards, false), Some("a"));
    assert_eq!(session.move_focus(&cards, false), Some("a"));

    session.set_focused("c");
    assert_eq!(session.move_focus(&cards, true), Some("c"));

    session.set_focused("gone");
    assert_eq!(
        session.move_focus(&cards, false),
        Some("c"),
        "a card that disappeared re-enters from the end"
    );

    session.prune_focus(&strings(&["a"]));
    assert_eq!(session.focused(), None);
}

#[test]
fn computing_a_card_reports_changed_rows_of_the_uncollapsed_diff() {
    let old: String = (1..=60).map(|line| format!("line {line}\n")).collect();
    let new = old.replace("line 30\n", "changed 30\n");

    let computed = DiffEditFileDiff::compute("sample.txt", &old, &new, false, false);

    assert_eq!(computed.changed_lines, [30, 31]);
    assert!(
        computed.display.lines.len() < 60,
        "the rendered diff is collapsed"
    );
    let changed_display: Vec<u32> = computed
        .display_to_full
        .iter()
        .filter(|mapping| computed.changed_lines.contains(&mapping.full_line))
        .map(|mapping| mapping.display_line)
        .collect();
    assert_eq!(
        changed_display.len(),
        2,
        "every changed row stays reachable from a display row"
    );
}
