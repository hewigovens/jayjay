#[cfg(feature = "wasm-grammars")]
mod bundled;
#[cfg(feature = "syntax")]
#[cfg_attr(target_family = "wasm", path = "syntax/wasm.rs")]
mod cache;
#[cfg(feature = "lang-fluent")]
mod fluent;
#[cfg(feature = "lang-markdown")]
mod markdown;
#[cfg(wasm_grammars)]
mod wasm_grammar;

use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[cfg(wasm_grammars)]
pub(crate) use wasm_grammar::with_wasm_store;

#[cfg(not(wasm_grammars))]
pub(crate) fn with_wasm_store<T, R>(
    owner: &mut T,
    _language: &Language,
    run: impl FnOnce(&mut T) -> R,
) -> R {
    run(owner)
}

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
    ("tag.attribute", SyntaxToken::Attribute),
    ("field", SyntaxToken::Variable),
    ("method", SyntaxToken::Function),
];

#[cfg(feature = "syntax")]
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
    with_wasm_store(&mut Highlighter::new(), &config.language, |highlighter| {
        highlight_spans(highlighter, source, config)
    })
}

fn highlight_spans(
    highlighter: &mut Highlighter,
    source: &str,
    config: &HighlightConfiguration,
) -> Vec<HighlightSpan> {
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

#[cfg(feature = "syntax")]
trait Grammar {
    fn language(&self) -> Option<Language>;
}

#[cfg(feature = "syntax")]
impl<T: Clone + Into<Language>> Grammar for T {
    fn language(&self) -> Option<Language> {
        Some(self.clone().into())
    }
}

// tree-sitter-solidity 1.2.13 anchors a capture inside a grouped pattern, which tree-sitter 0.27 rejects, so the whole query failed to compile and Solidity was never highlighted.
#[cfg(feature = "lang-solidity")]
static SOLIDITY_HIGHLIGHT_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    tree_sitter_solidity::HIGHLIGHT_QUERY.replace(
        "((expression(identifier)) @type .)",
        "(expression (identifier)) @type",
    )
});

// Some grammars ship Neovim-dialect queries: tree-sitter-highlight keeps the last of `@comment @spell` and ignores `#lua-match?`, which would apply its capture to every node.
#[cfg(feature = "syntax")]
fn without_neovim_extensions(query: &str) -> String {
    query
        .replace(" @nospell", "")
        .replace(" @spell", "")
        .replace("#lua-match?", "#match?")
        .replace("%u", "A-Z")
        .replace("%d", "\\d")
}

/// One table drives both highlighting and the syntax matcher, so a language supported by one is supported by the other.
macro_rules! languages {
    ($($feature:literal: $name:literal $(| $alias:literal)* => $grammar:expr, $query:expr;)*) => {
        pub(crate) fn grammar(language: &str) -> Option<Language> {
            match language {
                $(#[cfg(feature = $feature)]
                $name $(| $alias)* => $grammar.language(),)*
                _ => None,
            }
        }

        fn with_config_for_language<R>(
            language: &str,
            _use_config: impl FnOnce(&HighlightConfiguration) -> R,
        ) -> Option<R> {
            match language {
                $(#[cfg(feature = $feature)]
                $name $(| $alias)* => {
                    fn build_config() -> Option<HighlightConfiguration> {
                        let mut config =
                            HighlightConfiguration::new($grammar.language()?, $name, &without_neovim_extensions($query), "", "").ok()?;
                        config.configure(&HIGHLIGHT_NAMES);
                        Some(config)
                    }
                    cache::with_config!(build_config, _use_config)
                })*
                _ => None,
            }
        }
    };
}

languages! {
    "lang-rust": "rust" => tree_sitter_rust::LANGUAGE, tree_sitter_rust::HIGHLIGHTS_QUERY;
    "lang-javascript": "javascript" | "jsx" => tree_sitter_javascript::LANGUAGE, tree_sitter_javascript::HIGHLIGHT_QUERY;
    // TypeScript's query extends JavaScript's; TSX also needs the JSX captures.
    // Capture the void token itself so JavaScript's nested keyword capture cannot override its enclosing type.
    "lang-typescript": "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT, &[
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
        "(predefined_type \"void\" @type.builtin)",
    ].join("\n");
    "lang-typescript": "tsx" => tree_sitter_typescript::LANGUAGE_TSX, &[
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
        "(predefined_type \"void\" @type.builtin)",
    ].join("\n");
    "lang-python": "python" => tree_sitter_python::LANGUAGE, tree_sitter_python::HIGHLIGHTS_QUERY;
    "lang-go": "go" => tree_sitter_go::LANGUAGE, tree_sitter_go::HIGHLIGHTS_QUERY;
    "lang-c": "c" => tree_sitter_c::LANGUAGE, tree_sitter_c::HIGHLIGHT_QUERY;
    "lang-csharp": "csharp" => bundled::CSHARP, bundled::CSHARP_QUERY;
    "lang-cpp": "cpp" => tree_sitter_cpp::LANGUAGE, &[
        tree_sitter_c::HIGHLIGHT_QUERY,
        tree_sitter_cpp::HIGHLIGHT_QUERY,
    ].join("\n");
    "lang-json": "json" => tree_sitter_json::LANGUAGE, tree_sitter_json::HIGHLIGHTS_QUERY;
    "lang-toml": "toml" => tree_sitter_toml_ng::LANGUAGE, tree_sitter_toml_ng::HIGHLIGHTS_QUERY;
    "lang-ruby": "ruby" => bundled::RUBY, bundled::RUBY_QUERY;
    "lang-java": "java" => tree_sitter_java::LANGUAGE, tree_sitter_java::HIGHLIGHTS_QUERY;
    "lang-kotlin": "kotlin" => bundled::KOTLIN, bundled::KOTLIN_QUERY;
    "lang-php": "php" => bundled::PHP, bundled::PHP_QUERY;
    "lang-markdown": "markdown" => tree_sitter_md::LANGUAGE, tree_sitter_md::HIGHLIGHT_QUERY_BLOCK;
    "lang-markdown": "markdown_inline" => tree_sitter_md::INLINE_LANGUAGE, tree_sitter_md::HIGHLIGHT_QUERY_INLINE;
    "lang-css": "css" => tree_sitter_css::LANGUAGE, tree_sitter_css::HIGHLIGHTS_QUERY;
    "lang-html": "html" => tree_sitter_html::LANGUAGE, tree_sitter_html::HIGHLIGHTS_QUERY;
    "lang-bash": "shell" | "bash" => tree_sitter_bash::LANGUAGE, tree_sitter_bash::HIGHLIGHT_QUERY;
    "lang-batch": "batch" => tree_sitter_batch::LANGUAGE, tree_sitter_batch::HIGHLIGHTS_QUERY;
    "lang-fish": "fish" => tree_sitter_fish::language(), tree_sitter_fish::HIGHLIGHTS_QUERY;
    "lang-fluent": "fluent" => fluent::LANGUAGE, fluent::HIGHLIGHTS_QUERY;
    "lang-powershell": "powershell" => tree_sitter_powershell::LANGUAGE, tree_sitter_powershell::HIGHLIGHTS_QUERY;
    "lang-yaml": "yaml" => tree_sitter_yaml::LANGUAGE, tree_sitter_yaml::HIGHLIGHTS_QUERY;
    "lang-swift": "swift" => tree_sitter_swift::LANGUAGE, tree_sitter_swift::HIGHLIGHTS_QUERY;
    "lang-solidity": "solidity" => tree_sitter_solidity::LANGUAGE, SOLIDITY_HIGHLIGHT_QUERY.as_str();
    "lang-sql": "sql" => bundled::SQL, bundled::SQL_QUERY;
    "lang-xml": "xml" => tree_sitter_xml::LANGUAGE_XML, tree_sitter_xml::XML_HIGHLIGHT_QUERY;
    "lang-zig": "zig" => tree_sitter_zig::LANGUAGE, tree_sitter_zig::HIGHLIGHTS_QUERY;
    "lang-nix": "nix" => tree_sitter_nix::LANGUAGE, tree_sitter_nix::HIGHLIGHTS_QUERY;
    "lang-make": "make" => tree_sitter_make::LANGUAGE, tree_sitter_make::HIGHLIGHTS_QUERY;
    "lang-cmake": "cmake" => tree_sitter_cmake::LANGUAGE, tree_sitter_cmake::HIGHLIGHTS_QUERY;
    "lang-ini": "ini" => tree_sitter_ini::LANGUAGE, tree_sitter_ini::HIGHLIGHTS_QUERY;
    "lang-lua": "lua" => tree_sitter_lua::LANGUAGE, tree_sitter_lua::HIGHLIGHTS_QUERY;
}

/// The extension decides; a file it does not place is read by its shebang.
pub fn language_for(path: &str, source: &str) -> &'static str {
    match language_for_path(path) {
        "plaintext" => shebang_language(source)
            .or_else(|| language_for_ambiguous_name(path, source))
            .unwrap_or("plaintext"),
        language => language,
    }
}

/// Names other formats share, so the content has to agree before they are highlighted.
fn language_for_ambiguous_name(path: &str, source: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next()?.to_ascii_lowercase();
    if name.starts_with(".env.") {
        return Some("shell");
    }
    let (_, ext) = name.rsplit_once('.')?;
    let starts_with_section = || {
        source
            .lines()
            .map(str::trim_start)
            .find(|line| !line.is_empty() && !line.starts_with(['#', ';']))
            .is_some_and(|line| {
                let line = line.trim_end();
                line.len() > 2 && line.starts_with('[') && line.ends_with(']')
            })
    };
    (matches!(ext, "service" | "socket" | "timer" | "desktop") && starts_with_section())
        .then_some("ini")
}

fn shebang_language(source: &str) -> Option<&'static str> {
    let mut words = source
        .strip_prefix("#!")?
        .lines()
        .next()?
        .split_whitespace();
    let mut program = words.next()?;
    if program.rsplit('/').next() == Some("env") {
        program = loop {
            match words.next()? {
                "-u" | "-C" | "-P" | "--unset" | "--chdir" => {
                    words.next();
                }
                word if word.starts_with('-') || word.contains('=') => {}
                word => break word,
            }
        };
    }
    match program.rsplit('/').next()? {
        // The body is written for the interpreter a `#! nix-shell -i` line names, bash when none does.
        "nix-shell" => match nix_shell_interpreter(source) {
            Some(interpreter) => interpreter_language(interpreter?),
            None => Some("shell"),
        },
        "uv" => (words.next() == Some("run")
            && words
                .take_while(|word| word.starts_with('-') && *word != "--")
                .any(|word| matches!(word, "--script" | "-s")))
        .then_some("python"),
        interpreter => interpreter_language(interpreter),
    }
}

/// The `-i` operand on the `#! nix-shell` lines, which Nix reads from anywhere in the file; the inner option is empty when the flag has no operand.
fn nix_shell_interpreter(source: &str) -> Option<Option<&str>> {
    let mut arguments = source.lines().skip(1).flat_map(|line| {
        let mut words = line.strip_prefix("#!").unwrap_or("").split_whitespace();
        (words.next() == Some("nix-shell"))
            .then_some(words)
            .into_iter()
            .flatten()
    });
    arguments.find(|word| *word == "-i")?;
    Some(arguments.next().map(|word| word.trim_matches(['\'', '"'])))
}

fn interpreter_language(program: &str) -> Option<&'static str> {
    let interpreter = program
        .rsplit('/')
        .next()?
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    Some(match interpreter {
        "sh" | "bash" | "zsh" | "dash" | "ksh" | "ash" | "bats" => "shell",
        "fish" => "fish",
        "python" | "pypy" => "python",
        "ruby" => "ruby",
        "node" | "nodejs" | "zx" => "javascript",
        "deno" | "bun" | "ts-node" | "tsx" => "typescript",
        "lua" | "luajit" => "lua",
        "php" => "php",
        "pwsh" | "powershell" => "powershell",
        "make" => "make",
        "swift" => "swift",
        "cargo" => "rust",
        _ => return None,
    })
}

fn language_for_path(path: &str) -> &'static str {
    let basename = path.rsplit('/').next().unwrap_or(path);
    match basename {
        "Makefile" | "makefile" | "GNUmakefile" | "Makefile.am" => return "make",
        "CMakeLists.txt" => return "cmake",
        "Gemfile" | "Rakefile" | "Podfile" | "Fastfile" | "Appfile" | "Matchfile" | "Brewfile"
        | "Vagrantfile" | "Guardfile" | "Dangerfile" => return "ruby",
        "PKGBUILD" | "APKBUILD" | ".env" | ".envrc" | ".profile" | ".bashrc" | ".bash_profile"
        | ".bash_aliases" | ".bash_logout" | ".zshrc" | ".zshenv" | ".zprofile" => return "shell",
        ".npmrc" | ".gitmodules" | ".pylintrc" | ".flake8" | ".hgrc" | "setup.cfg" => return "ini",
        // YAML is a superset of JSON, so it covers the rc files that may hold either.
        ".clang-format" | ".clang-tidy" | ".yamllint" | ".gemrc" | ".prettierrc" | ".eslintrc"
        | ".babelrc" | ".stylelintrc" => return "yaml",
        "BUILD" | "WORKSPACE" | "SConstruct" | "SConscript" | "Tiltfile" => return "python",
        "Cargo.lock" | "Pipfile" | "poetry.lock" | "uv.lock" | "pdm.lock" | "Gopkg.lock" => {
            return "toml";
        }
        "flake.lock" | "Pipfile.lock" | "composer.lock" | "deno.lock" | "Package.resolved" => {
            return "json";
        }
        "Podfile.lock" | "pubspec.lock" | "pixi.lock" | "CITATION.cff" => return "yaml",
        _ => {}
    }
    let name = basename.to_ascii_lowercase();
    if let Some(stem) = name.strip_suffix(".config")
        && (matches!(stem, "web" | "app" | "nuget" | "packages")
            || stem.starts_with("web.")
            || stem.ends_with(".exe")
            || stem.ends_with(".dll"))
    {
        return "xml";
    }
    let ext = name.rsplit_once('.').map_or("", |(_, ext)| ext);
    match ext {
        "rs" => "rust",
        "swift" | "swiftinterface" => "swift",
        "js" | "cjs" | "mjs" | "jsx" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "py" | "pyi" | "pyw" | "bzl" | "bazel" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cs" | "csx" => "csharp",
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hxx" | "hh" | "h++" | "inl" | "ixx" | "ipp"
        | "tpp" | "ino" | "cu" | "cuh" => "cpp",
        "rb" | "gemspec" | "podspec" | "rake" | "ru" => "ruby",
        "sh" | "bash" | "zsh" | "ksh" | "bats" | "ebuild" | "env" => "shell",
        "fish" => "fish",
        "ftl" => "fluent",
        "html" | "htm" | "vue" | "svelte" => "html",
        "css" | "scss" | "pcss" | "postcss" => "css",
        "json" | "jsonc" | "jsonl" | "ndjson" | "geojson" | "webmanifest" | "xcstrings"
        | "tfstate" | "code-workspace" => "json",
        "php" | "phtml" => "php",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" | "mdx" | "mdown" | "mkd" | "mdc" | "rmd" | "qmd" => "markdown",
        "sql" | "psql" | "pgsql" | "mysql" | "ddl" | "dml" => "sql",
        "xml" | "xsd" | "xsl" | "xslt" | "svg" | "plist" | "entitlements" | "xib"
        | "storyboard" | "xcscheme" | "xcworkspacedata" | "xaml" | "csproj" | "fsproj"
        | "vbproj" | "vcxproj" | "props" | "targets" | "resx" | "nuspec" | "slnx" | "manifest"
        | "appxmanifest" | "wxs" | "wxi" | "filters" | "ruleset" | "runsettings" | "xhtml"
        | "rss" | "atom" | "kml" | "gpx" | "xlf" | "xliff" | "wsdl" | "pom" | "stringsdict"
        | "xcprivacy" | "iml" | "fxml" | "qrc" | "glade" | "tmx" | "ps1xml" | "vsixmanifest" => {
            "xml"
        }
        "ps1" | "psm1" | "psd1" => "powershell",
        "bat" | "cmd" => "batch",
        "ini" | "editorconfig" | "gitconfig" | "properties" => "ini",
        "cmake" => "cmake",
        "sol" => "solidity",
        "zig" | "zon" => "zig",
        "nix" => "nix",
        "lua" | "rockspec" => "lua",
        "mk" | "mak" => "make",
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
