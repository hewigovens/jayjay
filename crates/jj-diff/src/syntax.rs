#[cfg_attr(target_family = "wasm", path = "syntax/wasm.rs")]
mod cache;
#[cfg(feature = "lang-markdown")]
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

#[derive(Debug, Clone)]
pub(crate) struct HighlightSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) token: SyntaxToken,
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub(crate) fn highlight(source: &str, language: &str) -> Vec<HighlightSpan> {
    let Some(spans) =
        with_config_for_language(language, |config| highlight_with_config(source, config))
    else {
        return vec![];
    };
    #[cfg(feature = "lang-markdown")]
    if language == "markdown" {
        return markdown::merge_block_and_inline(source, spans);
    }
    spans
}

fn highlight_with_config(source: &str, config: &HighlightConfiguration) -> Vec<HighlightSpan> {
    let mut highlighter = Highlighter::new();
    let highlights = match highlighter.highlight(config, source.as_bytes(), None, None, |_| None) {
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

// tree-sitter-solidity 1.2.13 anchors a capture inside a grouped pattern, which tree-sitter 0.27 rejects, so the whole query failed to compile and Solidity was never highlighted.
#[cfg(feature = "lang-solidity")]
static SOLIDITY_HIGHLIGHT_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    tree_sitter_solidity::HIGHLIGHT_QUERY.replace(
        "((expression(identifier)) @type .)",
        "(expression (identifier)) @type",
    )
});

fn with_config_for_language<R>(
    language: &str,
    use_config: impl FnOnce(&HighlightConfiguration) -> R,
) -> Option<R> {
    macro_rules! cached_config {
        ($name:literal, $grammar:ident, $query:ident) => {
            cached_config!($name, $grammar::LANGUAGE, $grammar::$query)
        };
        ($name:literal, $grammar:expr, $query:expr $(,)?) => {{
            fn build_config() -> Option<HighlightConfiguration> {
                let mut config =
                    HighlightConfiguration::new($grammar.into(), $name, $query, "", "").ok()?;
                config.configure(&HIGHLIGHT_NAMES);
                Some(config)
            }
            cache::with_config!(build_config, use_config)
        }};
    }

    match language {
        #[cfg(feature = "lang-rust")]
        "rust" => cached_config!("rust", tree_sitter_rust, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-javascript")]
        "javascript" | "jsx" => {
            cached_config!("javascript", tree_sitter_javascript, HIGHLIGHT_QUERY)
        }
        #[cfg(feature = "lang-typescript")]
        "typescript" => cached_config!(
            "typescript",
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-typescript")]
        "tsx" => cached_config!(
            "tsx",
            tree_sitter_typescript::LANGUAGE_TSX,
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-python")]
        "python" => cached_config!("python", tree_sitter_python, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-go")]
        "go" => cached_config!("go", tree_sitter_go, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-c")]
        "c" => cached_config!("c", tree_sitter_c, HIGHLIGHT_QUERY),
        #[cfg(feature = "lang-csharp")]
        "csharp" => cached_config!("csharp", tree_sitter_c_sharp, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-cpp")]
        "cpp" => cached_config!("cpp", tree_sitter_cpp, HIGHLIGHT_QUERY),
        #[cfg(feature = "lang-json")]
        "json" => cached_config!("json", tree_sitter_json, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-toml")]
        "toml" => cached_config!("toml", tree_sitter_toml_ng, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-ruby")]
        "ruby" => cached_config!("ruby", tree_sitter_ruby, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-java")]
        "java" => cached_config!("java", tree_sitter_java, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-kotlin")]
        "kotlin" => cached_config!("kotlin", tree_sitter_kotlin_sg, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-php")]
        "php" => cached_config!(
            "php",
            tree_sitter_php::LANGUAGE_PHP,
            tree_sitter_php::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-markdown")]
        "markdown" => cached_config!("markdown", tree_sitter_md, HIGHLIGHT_QUERY_BLOCK),
        #[cfg(feature = "lang-markdown")]
        "markdown_inline" => cached_config!(
            "markdown_inline",
            tree_sitter_md::INLINE_LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
        ),
        #[cfg(feature = "lang-css")]
        "css" => cached_config!("css", tree_sitter_css, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-html")]
        "html" => cached_config!("html", tree_sitter_html, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-bash")]
        "shell" | "bash" => cached_config!("shell", tree_sitter_bash, HIGHLIGHT_QUERY),
        #[cfg(feature = "lang-fish")]
        "fish" => cached_config!(
            "fish",
            tree_sitter_fish::language(),
            tree_sitter_fish::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-yaml")]
        "yaml" => cached_config!("yaml", tree_sitter_yaml, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-swift")]
        "swift" => cached_config!("swift", tree_sitter_swift, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-solidity")]
        "solidity" => cached_config!(
            "solidity",
            tree_sitter_solidity::LANGUAGE,
            SOLIDITY_HIGHLIGHT_QUERY.as_str(),
        ),
        #[cfg(feature = "lang-sql")]
        "sql" => cached_config!("sql", tree_sitter_sequel, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-xml")]
        "xml" => cached_config!(
            "xml",
            tree_sitter_xml::LANGUAGE_XML,
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
        ),
        #[cfg(feature = "lang-zig")]
        "zig" => cached_config!("zig", tree_sitter_zig, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-nix")]
        "nix" => cached_config!("nix", tree_sitter_nix, HIGHLIGHTS_QUERY),
        #[cfg(feature = "lang-make")]
        "make" => cached_config!("make", tree_sitter_make, HIGHLIGHTS_QUERY),
        _ => None,
    }
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
        "fish" => "fish",
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

#[cfg(test)]
mod tests {
    use std::sync::Barrier;

    use super::*;

    #[test]
    fn configurations_are_shared_while_parallel_highlights_keep_their_own_state() {
        let barrier = Barrier::new(2);
        let configs = std::thread::scope(|scope| {
            let workers = [
                ("let value = \"hello\";\n", SyntaxToken::StringLit),
                ("let value = 42;\n", SyntaxToken::Number),
            ]
            .map(|(source, token)| {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    let config = with_config_for_language("swift", |config| {
                        std::ptr::from_ref(config) as usize
                    })
                    .unwrap();
                    let spans = highlight(source, "swift");
                    assert!(spans.iter().any(|span| span.token == token));
                    config
                })
            });
            workers.map(|worker| worker.join().unwrap())
        });
        assert_eq!(configs[0], configs[1]);
    }
}
