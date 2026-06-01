use super::*;

#[test]
fn large_file_single_line_change_is_fast() {
    let mut lines: Vec<String> = (0..2000).map(|i| format!("line {i}")).collect();
    let old = lines.join("\n") + "\n";
    lines[1000] = "CHANGED".to_string();
    let new = lines.join("\n") + "\n";

    let start = std::time::Instant::now();
    let diff = compute_file_diff("Cargo.lock", &old, &new, false);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 1000,
        "2000-line diff with 1 change took {}ms (should be <1s)",
        elapsed.as_millis()
    );
    let changed: Vec<_> = diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert_eq!(changed.len(), 2, "should have 1 removed + 1 added line");
}
