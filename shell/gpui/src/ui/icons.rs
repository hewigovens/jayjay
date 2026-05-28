//! Lucide icon glyphs. Bundled at build time from
//! `assets/fonts/Lucide.ttf` — registered with GPUI's text system at startup.
//!
//! Use `icon(glyph)` to render an icon span; pass `glyph::COPY` etc.
//! Codepoints are pulled from `lucide-static`'s `font/codepoints.json`.

use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};

pub const FONT: &str = "lucide";

#[allow(dead_code)]
pub mod glyph {
    pub const ARROW_CLOCKWISE: &str = "\u{e145}";
    pub const ARROW_DOWN: &str = "\u{e042}";
    pub const ARROW_RIGHT: &str = "\u{e049}";
    pub const ARROW_UP: &str = "\u{e04a}";
    pub const ARROWS_LEFT_RIGHT: &str = "\u{e24a}";
    pub const BOOKMARK: &str = "\u{e060}";
    pub const CARET_DOWN: &str = "\u{e06d}";
    pub const CARET_RIGHT: &str = "\u{e06f}";
    pub const CHECK: &str = "\u{e06c}";
    pub const COLUMNS: &str = "\u{e098}";
    pub const COPY: &str = "\u{e09e}";
    pub const DOT: &str = "\u{e44f}";
    pub const FILE_CODE: &str = "\u{e0c3}";
    pub const FILTER: &str = "\u{e0dc}";
    pub const FOLDER: &str = "\u{e0d7}";
    pub const FOLDER_SIMPLE: &str = "\u{e0d7}";
    pub const FUNNEL: &str = "\u{e0dc}";
    pub const PACKAGE: &str = "\u{e129}";
    pub const HARD_DRIVE: &str = "\u{e0ed}";
    pub const PLUS_CIRCLE: &str = "\u{e081}";
    pub const MINUS_CIRCLE: &str = "\u{e07e}";
    pub const PENCIL_CIRCLE: &str = "\u{e1f9}";
    pub const ARROW_CIRCLE_RIGHT: &str = "\u{e07a}";
    pub const GEAR: &str = "\u{e154}";
    pub const GIT_BRANCH: &str = "\u{e0e2}";
    pub const GIT_MERGE: &str = "\u{e0e4}";
    pub const INFO: &str = "\u{e0f9}";
    pub const MAGIC_WAND: &str = "\u{e357}";
    pub const SEARCH: &str = "\u{e151}";
    pub const ROWS: &str = "\u{e58a}";
    pub const SIDEBAR: &str = "\u{e12a}";
    pub const SPARKLE: &str = "\u{e47e}";
    pub const WHITESPACE: &str = "\u{e3a3}";
    pub const WARNING: &str = "\u{e193}";
    pub const X: &str = "\u{e1b2}";
    pub const X_CIRCLE: &str = "\u{e084}";
}

/// Render an icon glyph at the given size, using a passed text color.
pub fn icon(glyph_str: &'static str, size: f32, color: u32) -> Div {
    div()
        .flex_none()
        .font_family(FONT)
        .text_size(px(size))
        .text_color(rgb(color))
        .child(SharedString::from(glyph_str))
}
