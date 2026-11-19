#[cfg(wasm_grammars)]
use super::wasm_grammar::WasmGrammar;

// `just wasm-grammars` regenerates these files from the grammar sources it pins.
macro_rules! bundled {
    ($($feature:literal: $grammar:ident, $query:ident = $file:literal $(, $linked:path)?;)*) => {$(
        #[cfg(all(feature = $feature, wasm_grammars))]
        pub(super) static $grammar: WasmGrammar =
            WasmGrammar::new($file, include_bytes!(concat!("../../grammars/wasm/", $file, ".wasm.gz")));
        #[cfg(all(feature = $feature, target_family = "wasm"))]
        linked!($grammar $(, $linked)?);
        #[cfg(feature = $feature)]
        pub(super) const $query: &str = include_str!(concat!("../../grammars/wasm/", $file, ".scm"));
    )*};
}

#[cfg(target_family = "wasm")]
macro_rules! linked {
    ($grammar:ident, $linked:path) => {
        pub(super) use $linked as $grammar;
    };
    ($grammar:ident) => {
        pub(super) const $grammar: unlinked::Unlinked = unlinked::Unlinked;
    };
}

/// A grammar the web build does not link, so its files show as plain text there.
#[cfg(all(
    target_family = "wasm",
    any(
        feature = "lang-dart",
        feature = "lang-dockerfile",
        feature = "lang-just",
        feature = "lang-proto",
        feature = "lang-r"
    )
))]
mod unlinked {
    pub(crate) struct Unlinked;

    impl crate::syntax::Grammar for Unlinked {
        fn language(&self) -> Option<tree_sitter::Language> {
            None
        }
    }
}

bundled! {
    "lang-csharp": CSHARP, CSHARP_QUERY = "c_sharp", tree_sitter_c_sharp::LANGUAGE;
    "lang-dart": DART, DART_QUERY = "dart";
    "lang-dockerfile": DOCKERFILE, DOCKERFILE_QUERY = "containerfile";
    "lang-just": JUST, JUST_QUERY = "just";
    "lang-kotlin": KOTLIN, KOTLIN_QUERY = "kotlin", tree_sitter_kotlin_sg::LANGUAGE;
    "lang-php": PHP, PHP_QUERY = "php", tree_sitter_php::LANGUAGE_PHP;
    "lang-proto": PROTO, PROTO_QUERY = "proto";
    "lang-r": R, R_QUERY = "r";
    "lang-ruby": RUBY, RUBY_QUERY = "ruby", tree_sitter_ruby::LANGUAGE;
    "lang-sql": SQL, SQL_QUERY = "sql", tree_sitter_sequel::LANGUAGE;
}
