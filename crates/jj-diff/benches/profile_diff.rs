use std::hint::black_box;
use std::time::Instant;

use jj_diff::{
    ContextExpansion, DiffSpanStyle, ExpandableDiff, FileDiff, SyntaxToken,
    build_side_by_side_rows, compute_file_diff, compute_file_diff_full, wrap_diff_lines,
    wrap_sbs_rows,
};

#[cfg(feature = "hotpath-alloc")]
#[global_allocator]
static ALLOCATOR: hotpath::CountingAllocator = hotpath::CountingAllocator::new();

fn main() {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    let scenario = match args.as_slice() {
        [] => "cold",
        [scenario]
            if matches!(
                scenario.as_str(),
                "cold" | "warm" | "expand" | "wrap" | "sbs"
            ) =>
        {
            scenario.as_str()
        }
        _ => panic!("usage: profile_diff [cold|warm|expand|wrap|sbs]"),
    };

    let mut lines: Vec<_> = (0..6000)
        .map(|i| {
            format!(
                "let value_{i}: u32 = {i} + 1; // a long comment to exercise wrapping at eighty display columns"
            )
        })
        .collect();
    let old = lines.join("\n") + "\n";
    lines[3000] = "let value_3000: u32 = 9999 + 2; // changed".to_owned();
    let new = lines.join("\n") + "\n";
    drop(lines);

    // Prepare warm state before collecting measurements; cold runs start in a fresh process.
    let prepared = match scenario {
        "cold" => None,
        "wrap" | "sbs" => Some(compute_file_diff_full("large.rs", &old, &new, false)),
        _ => Some(compute_file_diff("large.rs", &old, &new, false)),
    };
    if let Some(diff) = &prepared {
        check_diff(diff);
    }

    #[cfg(feature = "hotpath")]
    let _profile = hotpath::HotpathGuardBuilder::new("profile_diff")
        .percentiles(&[50.0, 95.0])
        .functions_limit(0)
        .build();

    let started = Instant::now();
    match scenario {
        "cold" => check_diff(&black_box(compute_file_diff("large.rs", &old, &new, false))),
        "warm" => {
            for _ in 0..100 {
                black_box(compute_file_diff("large.rs", &old, &new, false));
            }
        }
        "expand" => {
            let diff = prepared.unwrap();
            let initial_len = diff.lines.len();
            let region = diff
                .lines
                .iter()
                .filter_map(|line| line.context_region)
                .max_by_key(|region| region.line_count)
                .expect("fixture has collapsed context");
            assert!(region.line_count > 1000);
            let mut expandable = ExpandableDiff::new(diff, old, new);
            for step in 1..=100 {
                let result = expandable
                    .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
                    .unwrap();
                assert_eq!(result.inserted.count, 10);
                assert_eq!(result.diff.lines.len(), initial_len + step * 10);
                black_box(result);
            }
        }
        "wrap" => {
            let diff = prepared.unwrap();
            assert_eq!(diff.lines.len(), 6001);
            for _ in 0..100 {
                let wrapped = wrap_diff_lines(&diff.lines, 80);
                assert!(wrapped.len() > diff.lines.len());
                black_box(wrapped);
            }
        }
        "sbs" => {
            let diff = prepared.unwrap();
            assert_eq!(diff.lines.len(), 6001);
            for cols in (40..=89).chain((40..=89).rev()) {
                let rows = build_side_by_side_rows(&diff.lines);
                assert_eq!(rows.len(), 6000);
                let wrapped = wrap_sbs_rows(&rows, cols, cols);
                assert!(wrapped.len() > rows.len());
                black_box(wrapped);
            }
        }
        _ => unreachable!(),
    }
    eprintln!("{scenario}: workload elapsed {:?}", started.elapsed());
}

fn check_diff(diff: &FileDiff) {
    assert_eq!(
        diff.lines
            .iter()
            .filter(|line| matches!(line.style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
            .count(),
        2
    );
    assert!(
        diff.lines
            .iter()
            .flat_map(|line| &line.spans)
            .any(|span| span.token != SyntaxToken::Plain)
    );
}
