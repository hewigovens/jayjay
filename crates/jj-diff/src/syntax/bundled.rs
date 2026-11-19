#[cfg(wasm_grammars)]
use super::wasm_grammar::WasmGrammar;

// `just wasm-grammars` regenerates these files from the grammar crates in Cargo.lock.
macro_rules! bundled {
    ($($feature:literal: $grammar:ident, $query:ident = $file:literal, $linked:path;)*) => {$(
        #[cfg(all(feature = $feature, wasm_grammars))]
        pub(super) static $grammar: WasmGrammar =
            WasmGrammar::new($file, include_bytes!(concat!("../../grammars/wasm/", $file, ".wasm.gz")));
        #[cfg(all(feature = $feature, target_family = "wasm"))]
        pub(super) use $linked as $grammar;
        #[cfg(feature = $feature)]
        pub(super) const $query: &str = include_str!(concat!("../../grammars/wasm/", $file, ".scm"));
    )*};
}

bundled! {
    "lang-csharp": CSHARP, CSHARP_QUERY = "c_sharp", tree_sitter_c_sharp::LANGUAGE;
    "lang-kotlin": KOTLIN, KOTLIN_QUERY = "kotlin", tree_sitter_kotlin_sg::LANGUAGE;
    "lang-php": PHP, PHP_QUERY = "php", tree_sitter_php::LANGUAGE_PHP;
    "lang-ruby": RUBY, RUBY_QUERY = "ruby", tree_sitter_ruby::LANGUAGE;
    "lang-sql": SQL, SQL_QUERY = "sql", tree_sitter_sequel::LANGUAGE;
}
