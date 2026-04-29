use std::sync::OnceLock;

use font_kit::source::SystemSource;

const MONO_CANDIDATES: &[&str] = &[
    "SF Mono",
    "JetBrains Mono",
    "Fira Code",
    "Cascadia Code",
    "Menlo",
    "Consolas",
    "Liberation Mono",
    "DejaVu Sans Mono",
];

static MONO: OnceLock<String> = OnceLock::new();

pub fn mono() -> &'static str {
    MONO.get_or_init(|| pick(MONO_CANDIDATES, "monospace"))
        .as_str()
}

fn pick(candidates: &[&str], fallback: &str) -> String {
    let source = SystemSource::new();
    for name in candidates {
        if source.select_family_by_name(name).is_ok() {
            return (*name).to_owned();
        }
    }
    fallback.to_owned()
}
