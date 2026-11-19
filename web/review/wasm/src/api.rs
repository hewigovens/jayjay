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
pub fn language_for(path: &str, source: &str) -> String {
    jj_diff::syntax::language_for(path, source).to_owned()
}

/// The function table a grammar loader grows and links into; wasm-bindgen drops the linker's own table export.
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn function_table() -> JsValue {
    wasm_bindgen::function_table()
}

// wasm-bindgen's `__wbg_reset_state`, which the worker uses after a trap, calls the start function unconditionally.
#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
fn start() {}

/// The instance's raw exports, which a grammar loader links against; they change when wasm-bindgen resets the instance.
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn instance_exports() -> JsValue {
    wasm_bindgen::exports()
}

/// Registers a grammar linked into this module; `language_fn` must be the nonzero table index of its `tree_sitter_*` function.
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn register_grammar(language: &str, language_fn: u32, query: &str) -> Result<(), JsError> {
    let index = std::num::NonZeroUsize::new(language_fn as usize)
        .ok_or_else(|| JsError::new("A grammar's language function cannot be table index 0"))?;
    // SAFETY: a wasm32 function pointer is a nonzero table index, and calling an index of the wrong type or out of range traps.
    let function = unsafe {
        std::mem::transmute::<std::num::NonZeroUsize, unsafe extern "C" fn() -> *const ()>(index)
    };
    let grammar =
        tree_sitter::Language::new(unsafe { tree_sitter_language::LanguageFn::from_raw(function) });
    jj_diff::syntax::register_grammar(language, grammar, query)
        .map_err(|error| JsError::new(&error))
}
