use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_fluent() -> *const ();
}

pub(super) const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fluent) };
pub(super) const HIGHLIGHTS_QUERY: &str = include_str!("../../grammars/fluent/highlights.scm");
