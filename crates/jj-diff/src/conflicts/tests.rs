use super::*;
use crate::syntax::SyntaxToken;
use crate::types::{DiffLine, DiffSpan, DiffSpanStyle};

#[test]
fn annotates_jj_diff_style_conflict_lines() {
    let mut lines = [
        line("line1", DiffSpanStyle::Context),
        line("<<<<<<< conflict 1 of 1", DiffSpanStyle::Added),
        line("%%%%%%% diff from: base", DiffSpanStyle::Added),
        line(
            "\\\\\\\\\\\\\\        to: destination",
            DiffSpanStyle::Added,
        ),
        line("-old", DiffSpanStyle::Added),
        line("+new", DiffSpanStyle::Added),
        line("+++++++ side", DiffSpanStyle::Added),
        line("side content", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 1 ends", DiffSpanStyle::Added),
        line("line3", DiffSpanStyle::Context),
    ];

    annotate_conflict_lines(&mut lines);

    let kinds = lines
        .iter()
        .map(|line| line.conflict_kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            ConflictLineKind::None,
            ConflictLineKind::Start,
            ConflictLineKind::Section,
            ConflictLineKind::Section,
            ConflictLineKind::Removed,
            ConflictLineKind::Added,
            ConflictLineKind::Section,
            ConflictLineKind::Content,
            ConflictLineKind::End,
            ConflictLineKind::None,
        ]
    );
}

#[test]
fn builds_clean_conflict_header_labels() {
    assert_eq!(
        conflict_display_text(ConflictLineKind::Start, "<<<<<<< conflict 1 of 2"),
        Some("Conflict 1 of 2".to_owned())
    );
    assert_eq!(
        conflict_display_text(ConflictLineKind::Section, "%%%%%%% diff from: base"),
        Some("Base".to_owned())
    );
    assert_eq!(
        conflict_display_text(
            ConflictLineKind::Section,
            "%%%%%%% diff from: rytwxylk 82cae1de \"base\" (parents of rebased revision)"
        ),
        Some("Base: base".to_owned())
    );
    assert_eq!(
        conflict_display_text(ConflictLineKind::Section, "\\\\\\\\\\\\\\ to: destination"),
        Some("Destination".to_owned())
    );
    assert_eq!(
        conflict_display_text(
            ConflictLineKind::Section,
            "\\\\\\\\\\\\\\ to: qwuknwyk e862f4b9 \"main: conflicting edits\" (rebase destination)"
        ),
        Some("Destination: main: conflicting edits".to_owned())
    );
    assert_eq!(
        conflict_display_text(
            ConflictLineKind::Section,
            "+++++++ npoqsuqs 54aee1ed \"feature: conflicting edits\" (rebased revision)"
        ),
        Some("Rebased: feature: conflicting edits".to_owned())
    );
    assert_eq!(
        conflict_display_text(ConflictLineKind::End, ">>>>>>> conflict 1 of 2 ends"),
        Some("End Conflict 1 of 2".to_owned())
    );
    assert_eq!(
        conflict_display_text(ConflictLineKind::Content, "plain"),
        None
    );
}

#[test]
fn builds_first_class_conflict_blocks() {
    let mut lines = [
        line("line1", DiffSpanStyle::Context),
        line("<<<<<<< conflict 1 of 1", DiffSpanStyle::Added),
        line("%%%%%%% diff from: base", DiffSpanStyle::Added),
        line("-old", DiffSpanStyle::Added),
        line("+new", DiffSpanStyle::Added),
        line("+++++++ side", DiffSpanStyle::Added),
        line("side content", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 1 ends", DiffSpanStyle::Added),
        line("line3", DiffSpanStyle::Context),
    ];
    annotate_conflict_lines(&mut lines);

    let items = build_diff_display_items(&lines);

    assert_eq!(items.len(), 3);
    assert_eq!(
        items[0],
        DiffDisplayItem::Lines {
            line_start: 0,
            line_end: 1
        }
    );
    let DiffDisplayItem::ConflictBlock { block } = &items[1] else {
        panic!("expected conflict block");
    };
    assert_eq!(block.title, "Conflict 1 of 1");
    assert_eq!((block.line_start, block.line_end), (1, 8));
    let labels = block
        .sections
        .iter()
        .map(|section| section.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec!["Conflict 1 of 1", "Base", "Side", "End Conflict 1 of 1"]
    );
    assert_eq!(
        items[2],
        DiffDisplayItem::Lines {
            line_start: 8,
            line_end: 9
        }
    );
}

#[test]
fn builds_compact_conflict_display_lines() {
    let mut lines = [
        line("line1", DiffSpanStyle::Context),
        line("<<<<<<< conflict 1 of 2", DiffSpanStyle::Added),
        line(
            "%%%%%%% diff from: rytwxylk 82cae1de \"base\" (parents of rebased revision)",
            DiffSpanStyle::Added,
        ),
        line("-value = base", DiffSpanStyle::Removed),
        line(
            "\\\\\\\\\\\\\\ to: qwuknwyk e862f4b9 \"main: conflicting edits\" (rebase destination)",
            DiffSpanStyle::Added,
        ),
        line("+value = main", DiffSpanStyle::Added),
        line(
            "+++++++ npoqsuqs 54aee1ed \"feature: conflicting edits\" (rebased revision)",
            DiffSpanStyle::Added,
        ),
        line("value = feature", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 2 ends", DiffSpanStyle::Added),
        line("line3", DiffSpanStyle::Context),
    ];
    annotate_conflict_lines(&mut lines);

    let display = build_diff_display_lines(&lines);
    let texts = display.iter().map(DiffLine::text).collect::<Vec<_>>();

    assert_eq!(
        texts,
        vec![
            "line1",
            "Conflict 1 of 2 · ◇ base · → main: conflicting edits · ← feature: conflicting edits",
            "◇ │ -value = base",
            "→ │ +value = main",
            "← │ value = feature",
            "line3"
        ]
    );
    assert_eq!(display[1].conflict_kind, ConflictLineKind::Start);
    assert_eq!(display[1].old_line_no, None);
    assert_eq!(display[1].new_line_no, None);
}

#[test]
fn truncates_long_conflict_summary_sources() {
    let mut lines = [
        line("<<<<<<< conflict 1 of 1", DiffSpanStyle::Added),
        line(
            "\\\\\\\\\\\\\\ to: qwuknwyk e862f4b9 \"this is a very long commit message that should be truncated before it makes the conflict row hard to scan\" (rebase destination)",
            DiffSpanStyle::Added,
        ),
        line("+value = main", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 1 ends", DiffSpanStyle::Added),
    ];
    annotate_conflict_lines(&mut lines);

    let display = build_diff_display_lines(&lines);
    let summary = display[0].text();

    assert!(summary.starts_with("Conflict 1 of 1 · → this is a very long commit message"));
    assert!(summary.ends_with('…'));
    assert!(!summary.contains("hard to scan"));
}

#[test]
fn prefixes_conflict_content_lines_with_section_source() {
    let mut lines = [
        line("<<<<<<< conflict 1 of 1", DiffSpanStyle::Added),
        line("%%%%%%% diff from: base", DiffSpanStyle::Added),
        line("-base", DiffSpanStyle::Removed),
        line("\\\\\\\\\\\\\\ to: destination", DiffSpanStyle::Added),
        line("+destination", DiffSpanStyle::Added),
        line("+++++++ side #2", DiffSpanStyle::Added),
        line("side", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 1 ends", DiffSpanStyle::Added),
    ];
    annotate_conflict_lines(&mut lines);

    let texts = build_diff_display_lines(&lines)
        .iter()
        .map(DiffLine::text)
        .collect::<Vec<_>>();

    assert_eq!(
        texts,
        vec![
            "Conflict 1 of 1 · ◇ base · → destination · ◆ Side #2",
            "◇ │ -base",
            "→ │ +destination",
            "◆ │ side"
        ]
    );
}

#[test]
fn prefixes_combined_base_destination_diff_rows_by_line_kind() {
    let mut lines = [
        line("<<<<<<< conflict 1 of 1", DiffSpanStyle::Added),
        line("%%%%%%% diff from: base", DiffSpanStyle::Added),
        line("\\\\\\\\\\\\\\ to: destination", DiffSpanStyle::Added),
        line("-base", DiffSpanStyle::Added),
        line("+destination", DiffSpanStyle::Added),
        line(">>>>>>> conflict 1 of 1 ends", DiffSpanStyle::Added),
    ];
    annotate_conflict_lines(&mut lines);

    let texts = build_diff_display_lines(&lines)
        .iter()
        .map(DiffLine::text)
        .collect::<Vec<_>>();

    assert_eq!(
        texts,
        vec![
            "Conflict 1 of 1 · ◇ base · → destination",
            "◇ │ -base",
            "→ │ +destination"
        ]
    );
}

#[test]
fn marker_text_requires_raw_marker_prefix() {
    assert_eq!(
        conflict_display_text(ConflictLineKind::Start, "Conflict 1 of 2"),
        None
    );
}

fn line(text: &str, style: DiffSpanStyle) -> DiffLine {
    DiffLine {
        old_line_no: None,
        new_line_no: Some(1),
        style,
        spans: vec![DiffSpan {
            text: text.to_owned(),
            style,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
        context_region: None,
    }
}
