fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(wasm_grammars)");
    if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("wasm") {
        return;
    }
    if std::env::var_os("CARGO_FEATURE_WASM_GRAMMARS").is_some() {
        println!("cargo::rustc-cfg=wasm_grammars");
    }
}
