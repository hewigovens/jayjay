use crate::{labels, render};

use wasm_bindgen::prelude::*;

/// Presentation data only; diffing, syntax, and row pairing belong to jj-diff.
#[wasm_bindgen]
pub fn render_diff(
    path: &str,
    before: &str,
    after: &str,
    ignore_whitespace: bool,
    show_all: bool,
    split: bool,
) -> String {
    render::diff(path, before, after, ignore_whitespace, show_all, split).to_string()
}

#[wasm_bindgen]
pub fn ui_labels(languages: &str) -> String {
    labels::labels(languages).to_string()
}

#[wasm_bindgen]
pub fn language_pack(path: &str) -> String {
    match jj_diff::syntax::language_for_path(path) {
        "rust" => "rust",
        "javascript" | "typescript" | "tsx" | "html" | "css" => "web",
        "python" => "python",
        "swift" => "swift",
        "shell" | "fish" | "powershell" | "batch" => "shell",
        "fluent" | "json" | "toml" | "yaml" | "ini" | "xml" | "markdown" => "data",
        "c" | "cpp" | "csharp" | "go" | "zig" => "systems",
        "java" | "kotlin" => "jvm",
        "ruby" | "php" | "nix" | "make" | "cmake" | "solidity" | "sql" => "other",
        _ => "base",
    }
    .to_owned()
}
