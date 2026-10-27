fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_LANG_FLUENT").is_none() {
        return;
    }
    let src = std::path::Path::new("grammars/fluent/src");
    println!("cargo:rerun-if-changed={}", src.display());
    let mut build = cc::Build::new();
    build
        .std("c11")
        .include(src)
        .file(src.join("parser.c"))
        .file(src.join("scanner.c"));
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag("-utf-8");
    }
    build.compile("tree-sitter-fluent");
}
