use super::wasm_grammar::WasmGrammar;

// `just wasm-grammars` regenerates these files from the grammar sources it pins.
macro_rules! bundled {
    ($($feature:literal: $grammar:ident, $query:ident = $file:literal;)*) => {$(
        #[cfg(feature = $feature)]
        pub(super) static $grammar: WasmGrammar =
            WasmGrammar::new($file, include_bytes!(concat!("../../grammars/wasm/", $file, ".wasm.gz")));
        #[cfg(feature = $feature)]
        pub(super) const $query: &str = include_str!(concat!("../../grammars/wasm/", $file, ".scm"));
    )*};
}

bundled! {
    "lang-csharp": CSHARP, CSHARP_QUERY = "c_sharp";
    "lang-dart": DART, DART_QUERY = "dart";
    "lang-dockerfile": DOCKERFILE, DOCKERFILE_QUERY = "containerfile";
    "lang-just": JUST, JUST_QUERY = "just";
    "lang-kotlin": KOTLIN, KOTLIN_QUERY = "kotlin";
    "lang-php": PHP, PHP_QUERY = "php";
    "lang-proto": PROTO, PROTO_QUERY = "proto";
    "lang-r": R, R_QUERY = "r";
    "lang-ruby": RUBY, RUBY_QUERY = "ruby";
    "lang-sql": SQL, SQL_QUERY = "sql";
}
