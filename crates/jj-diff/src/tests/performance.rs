use super::*;

#[test]
fn large_highlighted_file_single_line_change_is_fast() {
    // A .rs file so highlighting runs and exercises the per-line scan that was
    // O(lines x spans); many spans per line would dominate if it were quadratic.
    let mut lines: Vec<String> = (0..5000)
        .map(|i| format!("let value_{i}: u32 = {i} + 1; // comment {i}"))
        .collect();
    let old = lines.join("\n") + "\n";
    lines[2500] = "let value_2500: u32 = 9999 + 2; // changed".to_string();
    let new = lines.join("\n") + "\n";

    let start = std::time::Instant::now();
    let diff = compute_file_diff("big.rs", &old, &new, false);
    let elapsed = start.elapsed();

    // Both sides are parsed whole, which an unoptimized tree-sitter takes about 1.4 s for on the Windows runner; the quadratic per-line scan this guards against takes far longer.
    assert!(
        elapsed.as_millis() < 3_000,
        "5000-line highlighted diff with 1 change took {}ms (limit 3000ms)",
        elapsed.as_millis()
    );

    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert_eq!(changed.len(), 2, "should have 1 removed + 1 added line");

    // Highlighting must actually have run, otherwise the perf path is not covered.
    let highlighted = diff
        .lines
        .iter()
        .flat_map(|l| &l.spans)
        .any(|s| s.token != SyntaxToken::Plain);
    assert!(highlighted, "expected syntax tokens on a .rs diff");
}

#[test]
fn large_context_show_more_is_bounded() {
    let mut lines: Vec<String> = (0..6000)
        .map(|i| format!("let value_{i}: u32 = {i};"))
        .collect();
    let old = lines.join("\n") + "\n";
    lines[3000] = "let value_3000: u32 = 9999;".to_owned();
    let new = lines.join("\n") + "\n";
    let diff = compute_file_diff("large.rs", &old, &new, false);
    let initial_len = diff.lines.len();
    let region = diff
        .lines
        .iter()
        .filter_map(|line| line.context_region)
        .max_by_key(|region| region.line_count)
        .unwrap();

    let mut expandable = ExpandableDiff::new(diff, old, new);
    let first = expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    assert_eq!(first.diff.lines.len(), initial_len + 10);
    assert_eq!(first.inserted.count, 10);

    // Repeated reveals retain their highlights even if the shared render cache evicts this source.
    let start = std::time::Instant::now();
    let second = expandable
        .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let elapsed = start.elapsed();

    assert_eq!(second.diff.lines.len(), initial_len + 20);
    assert!(
        elapsed.as_millis() < 500,
        "6000-line cached Show More took {}ms (limit 500ms)",
        elapsed.as_millis()
    );
}
