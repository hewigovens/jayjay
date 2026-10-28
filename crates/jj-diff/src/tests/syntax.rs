use super::*;

#[test]
fn fish_source_is_detected_and_highlighted() {
    let source =
        "# Greeting\nfunction greet\n    set name (whoami)\n    echo \"Hello\" $name\nend\n";
    let diff = compute_file_diff_full("conf.d/greet.fish", "", source, false);
    let spans: Vec<_> = diff.lines.iter().flat_map(|line| &line.spans).collect();

    for (text, token) in [
        ("# Greeting", SyntaxToken::Comment),
        ("function", SyntaxToken::Keyword),
        ("greet", SyntaxToken::Function),
        ("whoami", SyntaxToken::Function),
        ("\"Hello\"", SyntaxToken::StringLit),
        ("end", SyntaxToken::Keyword),
    ] {
        assert!(
            spans
                .iter()
                .any(|span| span.text == text && span.token == token),
            "Fish should highlight {text:?} as {token:?}: {spans:?}"
        );
    }
    assert_eq!(diff.language, "fish");
}

#[cfg(feature = "lang-typescript")]
#[test]
fn typescript_and_tsx_keep_javascript_highlights() {
    let source = "// Greeting\nfunction greet(name: string): string {\n    const count: number = 42;\n    return \"Hello\" + name + count;\n}\nfunction notify(): void {}\nvoid notify();\n";
    for path in ["greeting.ts", "greeting.tsx"] {
        let lines = highlight_file(path, source);
        let spans: Vec<_> = lines.iter().flatten().collect();
        for (text, token) in [
            ("// Greeting", SyntaxToken::Comment),
            ("const", SyntaxToken::Keyword),
            ("greet", SyntaxToken::Function),
            ("string", SyntaxToken::Type),
            ("42", SyntaxToken::Number),
            ("\"Hello\"", SyntaxToken::StringLit),
            ("return", SyntaxToken::Keyword),
        ] {
            assert!(
                spans
                    .iter()
                    .any(|span| span.text == text && span.token == token),
                "{path} should highlight {text:?} as {token:?}: {spans:?}"
            );
        }
        let void_tokens: Vec<_> = spans
            .iter()
            .filter(|span| span.text == "void")
            .map(|span| span.token)
            .collect();
        assert_eq!(
            void_tokens,
            [SyntaxToken::Type, SyntaxToken::Keyword],
            "{path}"
        );
    }
}

#[cfg(feature = "lang-typescript")]
#[test]
fn tsx_highlights_jsx_tags_and_attributes() {
    let lines = highlight_file(
        "view.tsx",
        "const view = <section title=\"Greeting\"><Widget /></section>;\n",
    );
    let spans: Vec<_> = lines.iter().flatten().collect();
    for (text, token) in [
        ("section", SyntaxToken::Type),
        ("title", SyntaxToken::Attribute),
        ("\"Greeting\"", SyntaxToken::StringLit),
        ("Widget", SyntaxToken::Type),
    ] {
        assert!(
            spans
                .iter()
                .any(|span| span.text == text && span.token == token),
            "TSX should highlight {text:?} as {token:?}: {spans:?}"
        );
    }
}

#[test]
fn windows_build_and_script_sources_are_detected_and_highlighted() {
    for (path, language, source, tokens) in [
        (
            "scripts/greet.ps1",
            "powershell",
            "# Greeting\nfunction Get-Greeting {\n    param([string]$Name)\n    Write-Output \"Hello $Name\"\n}\n",
            &[
                SyntaxToken::Comment,
                SyntaxToken::Keyword,
                SyntaxToken::Variable,
                SyntaxToken::StringLit,
            ][..],
        ),
        (
            "BUILD.CMD",
            "batch",
            "@echo off\nREM Build\nset NAME=jayjay\nif exist out goto done\necho \"%NAME%\"\n:done\n",
            &[
                SyntaxToken::Comment,
                SyntaxToken::Keyword,
                SyntaxToken::Variable,
            ],
        ),
        (
            "setup.ini",
            "ini",
            "[core]\n; Defaults\neditor = vim\n",
            &[
                SyntaxToken::Comment,
                SyntaxToken::Type,
                SyntaxToken::Variable,
            ],
        ),
        (
            "src/CMakeLists.txt",
            "cmake",
            "# Targets\nif(WIN32)\n  add_executable(app main.cpp)\n  set(NAME \"app\")\nendif()\n",
            &[
                SyntaxToken::Comment,
                SyntaxToken::Keyword,
                SyntaxToken::Function,
                SyntaxToken::StringLit,
            ],
        ),
    ] {
        let diff = compute_file_diff_full(path, "", source, false);
        assert_eq!(diff.language, language, "{path}");
        let spans: Vec<_> = diff.lines.iter().flat_map(|line| &line.spans).collect();
        for token in tokens {
            assert!(
                spans.iter().any(|span| span.token == *token),
                "{language} should highlight {token:?}: {spans:?}"
            );
        }
    }
}

#[test]
fn spell_marked_comments_keep_the_comment_color() {
    for (path, source) in [
        ("App.swift", "// note\n"),
        ("query.sql", "-- note\n"),
        ("setup.ini", "; note\n"),
        ("app.cmake", "# note\n"),
    ] {
        let lines = highlight_file(path, source);
        assert!(
            lines[0]
                .iter()
                .any(|span| span.token == SyntaxToken::Comment),
            "{path}: {:?}",
            lines[0]
        );
    }
}

#[test]
fn lua_match_predicates_still_filter_captures() {
    let lines = highlight_file("main.zig", "const max_len = MAX;\n");
    let token = |text: &str| {
        lines[0]
            .iter()
            .find(|span| span.text == text)
            .map(|span| span.token)
    };
    assert_eq!(token("max_len"), Some(SyntaxToken::Variable));
    assert_eq!(token("MAX"), Some(SyntaxToken::Number));
}

#[test]
fn xml_family_extensions_use_the_xml_grammar() {
    for path in [
        "App/App.csproj",
        "Base.lproj/Main.storyboard",
        "Info.plist",
        "Views/MainWindow.xaml",
        "Web.config",
        "Web.Release.config",
        "bin/MyApp.exe.config",
    ] {
        let diff =
            compute_file_diff_full(path, "", "<root attr=\"1\"><!-- note --></root>\n", false);
        assert_eq!(diff.language, "xml", "{path}");
        assert!(
            diff.lines[0]
                .spans
                .iter()
                .any(|span| span.token == SyntaxToken::Comment),
            "{path}"
        );
    }
    for path in ["config", "kernel/configs/tiny.config"] {
        assert_eq!(
            compute_file_diff_full(path, "", "CONFIG_FOO=y\n", false).language,
            "plaintext",
            "{path}"
        );
    }
}

#[test]
fn solidity_source_is_detected_and_highlighted() {
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Greeter {
    string public greeting = "Hello";
    function greet() external view returns (string memory) {
        return greeting;
    }
}
"#;
    let diff = compute_file_diff_full("contracts/Greeter.sol", "", source, false);
    assert_eq!(diff.language, "solidity");
    let spans: Vec<_> = diff.lines.iter().flat_map(|line| &line.spans).collect();
    for token in [
        SyntaxToken::Comment,
        SyntaxToken::Keyword,
        SyntaxToken::Type,
        SyntaxToken::StringLit,
    ] {
        assert!(
            spans.iter().any(|span| span.token == token),
            "solidity should highlight {token:?}"
        );
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

#[test]
fn fluent_messages_terms_and_selectors_are_highlighted() {
    let source = "## Repository\n-brand = JayJay\nrepo-count = { $count ->\n        [one] One change\n       *[other] { NUMBER($count) } changes\n    }\n    .title = Open { -brand }\n";
    let diff = compute_file_diff_full("locales/en-US/repo.ftl", "", source, false);
    assert_eq!(diff.language, "fluent");
    let spans: Vec<_> = diff.lines.iter().flat_map(|line| &line.spans).collect();
    for (text, token) in [
        ("## Repository", SyntaxToken::Comment),
        ("-brand", SyntaxToken::Function),
        ("repo-count", SyntaxToken::Type),
        ("$count", SyntaxToken::Variable),
        ("other", SyntaxToken::Variable),
        ("NUMBER", SyntaxToken::Function),
        ("title", SyntaxToken::Attribute),
    ] {
        assert!(
            spans
                .iter()
                .any(|span| span.text == text && span.token == token),
            "Fluent should highlight {text:?} as {token:?}: {spans:?}"
        );
    }
}
