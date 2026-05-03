//! Phosphor icon glyphs (regular weight). Bundled at build time from
//! `assets/fonts/Phosphor.ttf` — registered with GPUI's text system at startup.
//!
//! Use `icon(glyph)` to render an icon span; pass `glyph::COPY` etc.
//! Codepoint reference: https://phosphoricons.com/ — values pulled from the
//! `@phosphor-icons/web` regular-weight CSS.

use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};

pub const FONT: &str = "Phosphor";

#[allow(dead_code)]
pub mod glyph {
    pub const ARROW_CLOCKWISE: &str = "\u{e036}";
    pub const ARROW_DOWN: &str = "\u{e03e}";
    pub const ARROW_RIGHT: &str = "\u{e06c}";
    pub const ARROW_UP: &str = "\u{e08e}";
    pub const BOOKMARK: &str = "\u{e0ea}";
    pub const CARET_DOWN: &str = "\u{e136}";
    pub const CARET_RIGHT: &str = "\u{e13a}";
    pub const CHECK: &str = "\u{e182}";
    pub const COLUMNS: &str = "\u{e546}";
    pub const COPY: &str = "\u{e1ca}";
    pub const DOT: &str = "\u{ecde}";
    pub const FILE_CODE: &str = "\u{e914}";
    pub const FOLDER: &str = "\u{e24a}";
    pub const FOLDER_SIMPLE: &str = "\u{e25a}";
    pub const FUNNEL: &str = "\u{e266}";
    pub const PACKAGE: &str = "\u{e29e}";
    pub const HARD_DRIVE: &str = "\u{e390}";
    pub const PLUS_CIRCLE: &str = "\u{e3d6}";
    pub const MINUS_CIRCLE: &str = "\u{e32c}";
    pub const PENCIL_CIRCLE: &str = "\u{e3b0}";
    pub const ARROW_CIRCLE_RIGHT: &str = "\u{e02e}";
    pub const GEAR: &str = "\u{e270}";
    pub const GIT_BRANCH: &str = "\u{e278}";
    pub const GIT_MERGE: &str = "\u{e280}";
    pub const INFO: &str = "\u{e2ce}";
    pub const MAGIC_WAND: &str = "\u{e6b6}";
    pub const SEARCH: &str = "\u{e30c}";
    pub const ROWS: &str = "\u{e5a2}";
    pub const SIDEBAR: &str = "\u{eab6}";
    pub const SPARKLE: &str = "\u{e6a2}";
    pub const WHITESPACE: &str = "\u{e6ee}";
    pub const WARNING: &str = "\u{e4e0}";
    pub const X: &str = "\u{e4f6}";
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
