use super::*;

fn assert_highlighted(path: &str, language: &str, source: &str) {
    let diff = compute_file_diff_full(path, "", source, false);
    assert_eq!(diff.language, language);

    let spans: Vec<_> = diff.lines.iter().flat_map(|line| &line.spans).collect();
    for token in [
        SyntaxToken::Comment,
        SyntaxToken::Keyword,
        SyntaxToken::Type,
        SyntaxToken::StringLit,
    ] {
        assert!(
            spans.iter().any(|span| span.token == token),
            "{language} should highlight {token:?} for {path}"
        );
    }
}

#[test]
fn kotlin_source_is_detected_and_highlighted() {
    let source = r#"// Greets a Kotlin user
fun greet(name: String): String = "Hello, $name"
"#;

    for path in ["src/Main.kt", "build.gradle.kts"] {
        assert_highlighted(path, "kotlin", source);
    }
}

#[test]
fn csharp_source_is_detected_and_highlighted() {
    let source = r#"// Greets a C# user
public class Greeter {
    public string Greet(string name) => $"Hello, {name}";
}
"#;

    assert_highlighted("src/Greeter.cs", "csharp", source);
}

#[test]
fn php_source_is_detected_and_highlighted() {
    let source = r#"<?php
// Greets a PHP user
function greet(string $name): string {
    return "Hello, $name";
}
"#;

    for path in ["src/greet.php", "templates/greet.phtml"] {
        assert_highlighted(path, "php", source);
    }
}

#[test]
fn markdown_block_and_inline_structure_is_highlighted() {
    let source = "# JayJay\nRead **carefully**, open [the guide](https://example.com), and run `jj st`.\n\n```text\n**not emphasis**\n```\n";
    let lines = highlight_file("README.md", source);
    let spans: Vec<_> = lines.iter().flatten().collect();

    for (text, token) in [
        ("JayJay", SyntaxToken::Type),
        ("carefully", SyntaxToken::Keyword),
        ("the guide", SyntaxToken::Attribute),
        ("https://example.com", SyntaxToken::StringLit),
        ("jj st", SyntaxToken::StringLit),
    ] {
        assert!(
            spans
                .iter()
                .any(|span| span.text == text && span.token == token),
            "Markdown should highlight {text:?} as {token:?}: {spans:?}"
        );
    }
    assert!(
        lines[4].iter().all(|span| span.token == SyntaxToken::Plain),
        "fenced code content should not be parsed as Markdown inline syntax: {:?}",
        lines[4]
    );
}

#[test]
fn materialized_conflict_keeps_source_syntax_highlighting() {
    let source = r#"<<<<<<< Conflict 1 of 1
%%%%%%% Changes from base to side #1
-fn old_name() { println!("old"); }
+fn new_name() { println!("new"); }
+++++++ Contents of side #2
fn other_name() { println!("others"); }
>>>>>>> Conflict 1 of 1 ends
"#;

    let lines = highlight_file("src/main.rs", source);
    for line in [2, 3, 5] {
        assert!(
            lines[line]
                .iter()
                .any(|span| span.text == "fn" && span.token == SyntaxToken::Keyword),
            "conflict source line {line} should retain Rust syntax highlighting"
        );
    }
}

#[test]
fn base_relative_highlights_follow_the_displayed_source_lines() {
    let base = "fn greeting() {\n    print(\"base\")\n}\n";
    let source = "fn greeting() {\n    print(\"left\")\n    print(\"added\")\n}\n";

    let lines = highlight_file_against_base("conflict.swift", base, source);

    assert_eq!(lines.len(), source.lines().count());
    assert_eq!(
        lines
            .iter()
            .map(|line| line.new_line_no)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4),]
    );
    assert_eq!(
        lines.iter().map(|line| line.style).collect::<Vec<_>>(),
        vec![
            DiffSpanStyle::Context,
            DiffSpanStyle::Added,
            DiffSpanStyle::Added,
            DiffSpanStyle::Context,
        ]
    );
    assert!(
        lines[1]
            .spans
            .iter()
            .any(|span| span.text.contains("left") && span.style == DiffSpanStyle::Added),
        "modified words should keep their stronger Added highlight"
    );
    assert!(
        lines
            .iter()
            .flat_map(|line| &line.spans)
            .any(|span| span.token != SyntaxToken::Plain),
        "source diff highlighting should retain syntax tokens"
    );
}

#[test]
fn highlighted_lines_reassemble_to_source_text() {
    // The windowed highlight scan must not drop or duplicate bytes across the binary-search/take-while boundaries: each line's spans must rejoin to source.
    let src = "fn alpha() -> u32 { 1 }\nfn beta() -> u32 { 2 }\nfn gamma() -> u32 { 3 }\n";
    let diff = compute_file_diff("sample.rs", src, src, false);

    let src_lines: Vec<&str> = src.lines().collect();
    let context: Vec<&DiffLine> = diff
        .lines
        .iter()
        .filter(|line| line.style == DiffSpanStyle::Context)
        .collect();
    assert_eq!(context.len(), src_lines.len());

    for (line, expected) in context.iter().zip(src_lines.iter()) {
        let joined: String = line.spans.iter().map(|span| span.text.as_str()).collect();
        assert_eq!(
            &joined, expected,
            "spans must reassemble to the source line"
        );
        assert!(
            line.spans
                .iter()
                .any(|span| span.token != SyntaxToken::Plain),
            "rust source should carry syntax tokens"
        );
    }
}

#[test]
fn plain_full_diff_matches_highlighted_line_structure() {
    let old = "fn main() {\n    let x = 1;\n}\n";
    let new = "fn main() {\n    let x = 2;\n}\n";
    let highlighted = compute_file_diff_full("t.rs", old, new, false);
    let plain = compute_file_diff_full_plain("t.rs", old, new, false);
    assert_eq!(plain.lines.len(), highlighted.lines.len());
    for (plain_line, highlighted_line) in plain.lines.iter().zip(&highlighted.lines) {
        assert_eq!(
            (
                plain_line.old_line_no,
                plain_line.new_line_no,
                plain_line.style
            ),
            (
                highlighted_line.old_line_no,
                highlighted_line.new_line_no,
                highlighted_line.style
            )
        );
    }
    assert!(plain.lines.iter().all(|line| {
        line.spans
            .iter()
            .all(|span| span.token == SyntaxToken::Plain)
    }));
    assert!(highlighted.lines.iter().any(|line| {
        line.spans
            .iter()
            .any(|span| span.token != SyntaxToken::Plain)
    }));
}
