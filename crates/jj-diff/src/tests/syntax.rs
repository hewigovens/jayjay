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
