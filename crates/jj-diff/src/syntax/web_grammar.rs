/// A language as the web build links it: the grammar file to load, its finished highlight query, and the other languages it highlights with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebGrammar {
    pub language: &'static str,
    pub grammar: &'static str,
    pub query: String,
    pub requires: Vec<&'static str>,
}

impl WebGrammar {
    pub(super) fn new(language: &'static str, grammar: &'static str, query: &str) -> Self {
        let requires = match language {
            #[cfg(feature = "lang-markdown")]
            "markdown" => vec![super::markdown::INLINE],
            _ => vec![],
        };
        Self {
            language,
            grammar,
            query: query.to_owned(),
            requires,
        }
    }
}
