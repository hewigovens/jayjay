mod markdown;

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

// Tree-sitter returns indices into this list, so names and tokens must stay aligned.
const HIGHLIGHTS: &[(&str, SyntaxToken)] = &[
    ("attribute", SyntaxToken::Attribute),
    ("comment", SyntaxToken::Comment),
    ("constant", SyntaxToken::Number),
    ("constant.builtin", SyntaxToken::Number),
    ("constructor", SyntaxToken::Type),
    ("function", SyntaxToken::Function),
    ("function.builtin", SyntaxToken::Function),
    ("function.method", SyntaxToken::Function),
    ("function.macro", SyntaxToken::Function),
    ("keyword", SyntaxToken::Keyword),
    ("number", SyntaxToken::Number),
    ("operator", SyntaxToken::Operator),
    ("property", SyntaxToken::Variable),
    ("punctuation", SyntaxToken::Punctuation),
    ("punctuation.bracket", SyntaxToken::Punctuation),
    ("punctuation.delimiter", SyntaxToken::Punctuation),
    ("string", SyntaxToken::StringLit),
    ("string.special", SyntaxToken::StringLit),
    ("type", SyntaxToken::Type),
    ("type.builtin", SyntaxToken::Type),
    ("variable", SyntaxToken::Variable),
    ("variable.builtin", SyntaxToken::Variable),
    ("variable.parameter", SyntaxToken::Variable),
    ("boolean", SyntaxToken::Number),
    ("character", SyntaxToken::StringLit),
    ("conditional", SyntaxToken::Keyword),
    ("exception", SyntaxToken::Keyword),
    ("float", SyntaxToken::Number),
    ("include", SyntaxToken::Keyword),
    ("keyword.function", SyntaxToken::Keyword),
    ("keyword.return", SyntaxToken::Keyword),
    ("label", SyntaxToken::Attribute),
    ("namespace", SyntaxToken::Type),
    ("none", SyntaxToken::Plain),
    ("parameter", SyntaxToken::Variable),
    ("punctuation.special", SyntaxToken::Punctuation),
    ("repeat", SyntaxToken::Keyword),
    ("string.escape", SyntaxToken::StringLit),
    ("string.regex", SyntaxToken::StringLit),
    ("module", SyntaxToken::Type),
    ("module.builtin", SyntaxToken::Type),
    ("property.definition", SyntaxToken::Variable),
    ("tag", SyntaxToken::Type),
    ("text.emphasis", SyntaxToken::Attribute),
    ("text.literal", SyntaxToken::StringLit),
    ("text.reference", SyntaxToken::Attribute),
    ("text.strong", SyntaxToken::Keyword),
    ("text.title", SyntaxToken::Type),
    ("text.uri", SyntaxToken::StringLit),
];

static HIGHLIGHT_NAMES: std::sync::LazyLock<Vec<&str>> =
    std::sync::LazyLock::new(|| HIGHLIGHTS.iter().map(|(name, _)| *name).collect());

fn index_to_token(idx: usize) -> SyntaxToken {
    HIGHLIGHTS
        .get(idx)
        .map_or(SyntaxToken::Plain, |(_, token)| *token)
}

/// A span of highlighted text.
#[derive(Debug, Clone)]
pub(crate) struct HighlightSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) token: SyntaxToken,
}

/// Highlight source code and return spans with token types.
pub(crate) fn highlight(source: &str, language: &str) -> Vec<HighlightSpan> {
    let config = match make_config(language) {
        Some(c) => c,
        None => return vec![],
    };
    let spans = highlight_with_config(source, &config);
    if language != "markdown" {
        return spans;
    }
    markdown::merge_block_and_inline(source, spans)
}

fn highlight_with_config(source: &str, config: &HighlightConfiguration) -> Vec<HighlightSpan> {
    let mut highlighter = Highlighter::new();
    let highlights = match highlighter.highlight(config, source.as_bytes(), None, |_| None) {
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
        "csharp" => (
            tree_sitter_c_sharp::LANGUAGE,
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
        ),
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
        "kotlin" => (
            tree_sitter_kotlin_sg::LANGUAGE,
            tree_sitter_kotlin_sg::HIGHLIGHTS_QUERY,
        ),
        "php" => (
            tree_sitter_php::LANGUAGE_PHP,
            tree_sitter_php::HIGHLIGHTS_QUERY,
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
        "solidity" => (
            tree_sitter_solidity::LANGUAGE,
            tree_sitter_solidity::HIGHLIGHT_QUERY,
        ),
        "sql" => (
            tree_sitter_sequel::LANGUAGE,
            tree_sitter_sequel::HIGHLIGHTS_QUERY,
        ),
        "xml" => (
            tree_sitter_xml::LANGUAGE_XML,
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
        ),
        "zig" => (tree_sitter_zig::LANGUAGE, tree_sitter_zig::HIGHLIGHTS_QUERY),
        "nix" => (tree_sitter_nix::LANGUAGE, tree_sitter_nix::HIGHLIGHTS_QUERY),
        "make" => (
            tree_sitter_make::LANGUAGE,
            tree_sitter_make::HIGHLIGHTS_QUERY,
        ),
        _ => return None,
    };

    let mut config =
        HighlightConfiguration::new(lang_fn.into(), language, highlights_query, "", "").ok()?;
    config.configure(&HIGHLIGHT_NAMES);
    Some(config)
}

pub(crate) fn language_for_path(path: &str) -> &'static str {
    let basename = path.rsplit('/').next().unwrap_or(path);
    if matches!(basename, "Makefile" | "makefile" | "GNUmakefile") {
        return "make";
    }
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
        "cs" => "csharp",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "rb" => "ruby",
        "sh" | "bash" | "zsh" => "shell",
        "html" | "htm" => "html",
        "css" | "scss" => "css",
        "json" => "json",
        "php" | "phtml" => "php",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "sql" => "sql",
        "xml" => "xml",
        "sol" => "solidity",
        "zig" => "zig",
        "nix" => "nix",
        "mk" => "make",
        _ => "plaintext",
    }
}
