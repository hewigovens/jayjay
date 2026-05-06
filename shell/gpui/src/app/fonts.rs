use std::sync::OnceLock;

use font_kit::source::SystemSource;
use gpui::{App, Pixels, font, px};

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

// Falls back to ~7.2 px (SF Mono / Menlo at 12 px) on measurement failure.
pub fn mono_advance(cx: &App, size: Pixels) -> Pixels {
    let font_id = cx.text_system().resolve_font(&font(mono()));
    cx.text_system()
        .ch_advance(font_id, size)
        .unwrap_or(px(7.2))
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
