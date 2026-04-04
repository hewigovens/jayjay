use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxToken {
    Plain,
    Keyword,
    StringLit,
    Comment,
    Number,
    Type,
    Function,
    Variable,
    Operator,
    Punctuation,
    Attribute,
}

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",             // 0
    "comment",               // 1
    "constant",              // 2
    "constant.builtin",      // 3
    "constructor",           // 4
    "function",              // 5
    "function.builtin",      // 6
    "function.method",       // 7
    "function.macro",        // 8
    "keyword",               // 9
    "number",                // 10
    "operator",              // 11
    "property",              // 12
    "punctuation",           // 13
    "punctuation.bracket",   // 14
    "punctuation.delimiter", // 15
    "string",                // 16
    "string.special",        // 17
    "type",                  // 18
    "type.builtin",          // 19
    "variable",              // 20
    "variable.builtin",      // 21
    "variable.parameter",    // 22
];

fn index_to_token(idx: usize) -> SyntaxToken {
    match idx {
        0 => SyntaxToken::Attribute,
        1 => SyntaxToken::Comment,
        2 | 3 => SyntaxToken::Number, // constants
        4 => SyntaxToken::Type,       // constructor
        5..=8 => SyntaxToken::Function,
        9 => SyntaxToken::Keyword,
        10 => SyntaxToken::Number,
        11 => SyntaxToken::Operator,
        12 => SyntaxToken::Variable, // property
        13..=15 => SyntaxToken::Punctuation,
        16 | 17 => SyntaxToken::StringLit,
        18 | 19 => SyntaxToken::Type,
        20..=22 => SyntaxToken::Variable,
        _ => SyntaxToken::Plain,
    }
}

/// A span of highlighted text.
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub token: SyntaxToken,
}

/// Highlight source code and return spans with token types.
pub fn highlight(source: &str, language: &str) -> Vec<HighlightSpan> {
    let config = match make_config(language) {
        Some(c) => c,
        None => return vec![],
    };

    let mut highlighter = Highlighter::new();
    let highlights = match highlighter.highlight(&config, source.as_bytes(), None, |_| None) {
        Ok(h) => h,
        Err(_) => return vec![],
    };

    let mut spans = Vec::new();
    let mut stack: Vec<SyntaxToken> = Vec::new();

    for event in highlights.flatten() {
        match event {
            HighlightEvent::HighlightStart(h) => {
                stack.push(index_to_token(h.0));
            }
            HighlightEvent::HighlightEnd => {
                stack.pop();
            }
            HighlightEvent::Source { start, end } => {
                let token = stack.last().copied().unwrap_or(SyntaxToken::Plain);
                spans.push(HighlightSpan { start, end, token });
            }
        }
    }

    spans
}

fn make_config(language: &str) -> Option<HighlightConfiguration> {
    let (lang_fn, highlights_query) = match language {
        "rust" => (
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
        ),
        "javascript" | "jsx" => (
            tree_sitter_javascript::LANGUAGE,
            tree_sitter_javascript::HIGHLIGHT_QUERY,
        ),
        "typescript" => (
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        ),
        "tsx" => (
            tree_sitter_typescript::LANGUAGE_TSX,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        ),
        "python" => (
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY,
        ),
        "go" => (tree_sitter_go::LANGUAGE, tree_sitter_go::HIGHLIGHTS_QUERY),
        "c" => (tree_sitter_c::LANGUAGE, tree_sitter_c::HIGHLIGHT_QUERY),
        "cpp" => (tree_sitter_cpp::LANGUAGE, tree_sitter_cpp::HIGHLIGHT_QUERY),
        "json" => (
            tree_sitter_json::LANGUAGE,
            tree_sitter_json::HIGHLIGHTS_QUERY,
        ),
        "toml" => (
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
        ),
        "ruby" => (
            tree_sitter_ruby::LANGUAGE,
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
        ),
        "java" => (
            tree_sitter_java::LANGUAGE,
            tree_sitter_java::HIGHLIGHTS_QUERY,
        ),
        "markdown" => (
            tree_sitter_md::LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
        ),
        "css" => (tree_sitter_css::LANGUAGE, tree_sitter_css::HIGHLIGHTS_QUERY),
        "html" => (
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY,
        ),
        "shell" | "bash" => (
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY,
        ),
        "yaml" => (
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
        ),
        "swift" => (
            tree_sitter_swift::LANGUAGE,
            tree_sitter_swift::HIGHLIGHTS_QUERY,
        ),
        _ => return None,
    };

    let mut config =
        HighlightConfiguration::new(lang_fn.into(), language, highlights_query, "", "").ok()?;
    config.configure(HIGHLIGHT_NAMES);
    Some(config)
}

pub fn language_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "swift" => "swift",
        "js" | "cjs" | "mjs" | "jsx" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "rb" => "ruby",
        "sh" | "bash" | "zsh" => "shell",
        "html" | "htm" => "html",
        "css" | "scss" => "css",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "sql" => "sql",
        "xml" => "xml",
        _ => "plaintext",
    }
}
