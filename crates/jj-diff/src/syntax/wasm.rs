macro_rules! with_config {
    ($build:expr, $use_config:expr) => {{
        // Tree-sitter's WASM language handles are neither Send nor Sync.
        thread_local! {
            static CONFIG: std::cell::LazyCell<Option<tree_sitter_highlight::HighlightConfiguration>> =
                std::cell::LazyCell::new($build);
        }
        CONFIG.with(|config| config.as_ref().map($use_config))
    }};
}

pub(super) use with_config;
