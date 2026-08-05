use super::fixtures::{regions, three_gap_diff};
use super::*;

#[test]
fn expanding_context_highlights_newly_visible_lines() {
    let (diff, old, new) = three_gap_diff();
    let region = regions(&diff)[1];
    let before_len = diff.lines.len();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    assert_eq!(expanded.diff.lines.len(), before_len + 10);
    let inserted = &expanded.diff.lines[expanded.inserted.start as usize
        ..(expanded.inserted.start + expanded.inserted.count) as usize];
    assert!(inserted.iter().all(|line| {
        line.style == DiffSpanStyle::Context
            && line.context_region.is_none()
            && line.old_line_no.is_some()
            && line.new_line_no.is_some()
    }));
}

#[test]
fn expansion_highlights_lines_inside_a_construct_whose_opener_stays_hidden() {
    let mut old_lines = vec![
        "let changed = 1;".to_owned(),
        "let a = 2;".to_owned(),
        "let b = 3;".to_owned(),
        "let c = 4;".to_owned(),
        "/* hidden opener".to_owned(),
    ];
    old_lines.extend((1..=38).map(|line| format!("   comment line {line}")));
    old_lines.extend([
        "*/".to_owned(),
        "let d = 5;".to_owned(),
        "let e = 6;".to_owned(),
    ]);
    old_lines.push("let tail = 7;".to_owned());
    let mut new_lines = old_lines.clone();
    new_lines[0] = "let changed = 2;".to_owned();
    let last = new_lines.len() - 1;
    new_lines[last] = "let tail = 8;".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let diff = compute_file_diff("hidden.rs", &old, &new, false);
    let region = regions(&diff)
        .into_iter()
        .max_by_key(|region| region.line_count)
        .unwrap();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let inserted = &expanded.diff.lines[expanded.inserted.start as usize
        ..(expanded.inserted.start + expanded.inserted.count) as usize];

    let comment_lines: Vec<_> = inserted
        .iter()
        .filter(|line| line.text().contains("comment line"))
        .collect();
    assert!(
        !comment_lines.is_empty(),
        "revealed suffix reaches the comment body"
    );
    assert!(comment_lines.iter().all(|line| {
        line.spans
            .iter()
            .filter(|span| !span.text.trim().is_empty())
            .all(|span| span.token == SyntaxToken::Comment)
    }));
}

#[test]
fn expansion_rehighlights_changed_lines_once_their_hidden_opener_is_revealed() {
    let mut old_lines: Vec<String> = (1..=5)
        .map(|line| format!("let v{line} = {line};"))
        .collect();
    old_lines.push("/* hidden opener".to_owned());
    old_lines.extend((1..=12).map(|line| format!("   body {line}")));
    old_lines.push("   changed inside comment".to_owned());
    old_lines.extend((13..=43).map(|line| format!("   body {line}")));
    old_lines.extend(["*/".to_owned(), "let tail = 2;".to_owned()]);
    let mut new_lines = old_lines.clone();
    new_lines[0] = "let v1 = 9;".to_owned();
    let changed_ix = 18;
    assert_eq!(old_lines[changed_ix], "   changed inside comment");
    new_lines[changed_ix] = "   rewritten inside comment".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let diff = compute_file_diff("stale.rs", &old, &new, false);
    let changed_spans_are_comment = |diff: &FileDiff| {
        diff.lines
            .iter()
            .filter(|line| line.text().contains("inside comment"))
            .all(|line| {
                line.spans
                    .iter()
                    .filter(|span| !span.text.trim().is_empty())
                    .all(|span| span.token == SyntaxToken::Comment)
            })
    };
    assert!(
        !changed_spans_are_comment(&diff),
        "the collapsed render parses the changed pair without its hidden opener"
    );
    let region = regions(&diff)
        .into_iter()
        .min_by_key(|region| region.new_start_line)
        .unwrap();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(region.id, ContextExpansion::ShowAll)
        .unwrap();

    assert!(
        changed_spans_are_comment(&expanded.diff),
        "revealing the opener re-highlights the visible changed pair as comment"
    );
}

#[test]
fn repeated_expansion_preserves_multiline_syntax_across_visible_chunks() {
    let mut old_lines = vec![
        "fn main() {".to_owned(),
        "    let changed = 1;".to_owned(),
        "    /* visible comment opener".to_owned(),
    ];
    old_lines.extend((1..=40).map(|line| format!("       comment line {line}")));
    old_lines.extend(["    */".to_owned(), "}".to_owned()]);
    let mut new_lines = old_lines.clone();
    new_lines[1] = "    let changed = 2;".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let diff = compute_file_diff("multiline.rs", &old, &new, false);
    let region = regions(&diff)
        .into_iter()
        .max_by_key(|region| region.line_count)
        .unwrap();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let second = expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let inserted = &second.diff.lines
        [second.inserted.start as usize..(second.inserted.start + second.inserted.count) as usize];

    assert!(inserted.iter().all(|line| {
        line.spans
            .iter()
            .filter(|span| !span.text.trim().is_empty())
            .all(|span| span.token == SyntaxToken::Comment)
    }));
}
