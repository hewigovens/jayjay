use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

use super::HIGHLIGHT_NAMES;

// Tree-sitter's language and query handles are neither Send nor Sync, and a browser engine runs on one thread; a configuration is shared out so no borrow spans grammar code, which can trap.
thread_local! {
    static GRAMMARS: RefCell<HashMap<String, Rc<HighlightConfiguration>>> = RefCell::new(HashMap::new());
}

/// Registers a grammar the host linked into this module with the finished query `web_grammars` exported for it.
pub fn register_grammar(language: &str, grammar: Language, query: &str) -> Result<(), String> {
    let mut config = HighlightConfiguration::new(grammar, language, query, "", "")
        .map_err(|error| format!("{language} highlight query: {error}"))?;
    config.configure(&HIGHLIGHT_NAMES);
    GRAMMARS.with(|grammars| {
        grammars
            .borrow_mut()
            .insert(language.to_owned(), Rc::new(config))
    });
    Ok(())
}

fn config(language: &str) -> Option<Rc<HighlightConfiguration>> {
    GRAMMARS.with(|grammars| grammars.borrow().get(language).cloned())
}

pub(crate) fn grammar(language: &str) -> Option<Language> {
    Some(config(language)?.language.clone())
}

pub(crate) fn with_config_for_language<R>(
    language: &str,
    use_config: impl FnOnce(&HighlightConfiguration) -> R,
) -> Option<R> {
    config(language).map(|config| use_config(&config))
}
