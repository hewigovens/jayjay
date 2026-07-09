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

    // The collapsed path should not syntax-highlight thousands of hidden lines;
    // this still catches the O(lines × spans) quadratic blowup this guards against.
    assert!(
        elapsed.as_millis() < 1_100,
        "5000-line highlighted diff with 1 change took {}ms (limit 1100ms)",
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
