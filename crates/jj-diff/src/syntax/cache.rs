macro_rules! with_config {
    ($build:expr, $use_config:expr) => {{
        static CONFIG: std::sync::LazyLock<Option<tree_sitter_highlight::HighlightConfiguration>> =
            std::sync::LazyLock::new($build);
        CONFIG.as_ref().map($use_config)
    }};
}

pub(super) use with_config;
