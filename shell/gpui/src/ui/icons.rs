//! Codicon glyphs from `assets/fonts/Codicons.ttf`, registered with GPUI's text system at startup.
//!
//! Use `icon(glyph)` to render an icon span; pass `glyph::COPY` etc.

use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};

const FONT: &str = "codicon";
/// Codicons are drawn on this grid; at multiples of it their strokes land on whole pixels.
pub(crate) const CODICON_GRID: f32 = 16.;
pub(crate) const SETI_FONT: &str = "seti";
pub const ARROW_DOWN_SVG: &str = "icons/arrow-down.svg";
pub const ARROW_UP_SVG: &str = "icons/arrow-up.svg";
pub const EXPAND_VERTICAL_SVG: &str = "icons/expand-vertical.svg";
pub const COLLAPSE_VERTICAL_SVG: &str = "icons/collapse-vertical.svg";
pub const CIRCLE_SVG: &str = "icons/circle.svg";
pub const CHECK_SVG: &str = "icons/check.svg";
pub const CIRCLE_HALF_SVG: &str = "icons/circle-half.svg";
/// Full color, drawn with `img` rather than tinted with `svg`.
pub const HEART_CIRCLE_SVG: &str = "icons/heart-circle.svg";

/// Every SVG the shell renders, paired with its bytes, so registering an icon is one entry here rather than a constant plus a match arm in the asset source.
pub const SVG_ASSETS: &[(&str, &[u8])] = &[
    (
        ARROW_DOWN_SVG,
        include_bytes!("../../assets/icons/arrow-down.svg"),
    ),
    (
        ARROW_UP_SVG,
        include_bytes!("../../assets/icons/arrow-up.svg"),
    ),
    (
        EXPAND_VERTICAL_SVG,
        include_bytes!("../../assets/icons/expand-vertical.svg"),
    ),
    (
        COLLAPSE_VERTICAL_SVG,
        include_bytes!("../../assets/icons/collapse-vertical.svg"),
    ),
    (CIRCLE_SVG, include_bytes!("../../assets/icons/circle.svg")),
    (CHECK_SVG, include_bytes!("../../assets/icons/check.svg")),
    (
        CIRCLE_HALF_SVG,
        include_bytes!("../../assets/icons/circle-half.svg"),
    ),
    (
        HEART_CIRCLE_SVG,
        include_bytes!("../../assets/icons/heart-circle.svg"),
    ),
    (
        REFRESH_SVG,
        include_bytes!("../../assets/icons/refresh.svg"),
    ),
    (
        SIDEBAR_LEADING_SVG,
        include_bytes!("../../assets/icons/sidebar-leading.svg"),
    ),
];
pub const REFRESH_SVG: &str = "icons/refresh.svg";
pub const SIDEBAR_LEADING_SVG: &str = "icons/sidebar-leading.svg";

pub mod glyph {
    pub const APP_WINDOW: &str = "\u{eb7f}";
    pub const ARROW_CIRCLE_RIGHT: &str = "\u{ebfe}";
    pub const ARROW_CLOCKWISE: &str = "\u{eb37}";
    pub const ARROW_DOWN: &str = "\u{ea9a}";
    pub const ARROW_RIGHT: &str = "\u{ea9c}";
    pub const ARROW_UP: &str = "\u{eaa1}";
    pub const ARROW_UP_DOWN: &str = "\u{e37d}";
    pub const ARROW_UTURN_BACK: &str = "\u{eae2}";
    pub const ARROWS_LEFT_RIGHT: &str = "\u{ebcb}";
    pub const A_LARGE_SMALL: &str = "\u{eb69}";
    pub const BOOKMARK: &str = "\u{eaa5}";
    pub const BRACES: &str = "\u{ea8b}";
    pub const CARET_DOWN: &str = "\u{eab4}";
    pub const CARET_RIGHT: &str = "\u{eab6}";
    pub const CASE_SENSITIVE: &str = "\u{eab1}";
    pub const CHART_BAR: &str = "\u{eb03}";
    pub const CHECK: &str = "\u{eab2}";
    pub const CLOUD: &str = "\u{ebaa}";
    pub const CLOUD_OFF: &str = "\u{e08d}";
    pub const COLUMNS: &str = "\u{eb56}";
    pub const CONTRAST: &str = "\u{eac6}";
    pub const COPY: &str = "\u{ebcc}";
    pub const DATABASE: &str = "\u{eace}";
    pub const DIFF: &str = "\u{eae1}";
    pub const DIFF_ADDED: &str = "\u{eadc}";
    pub const DIFF_MODIFIED: &str = "\u{eade}";
    pub const DIFF_REMOVED: &str = "\u{eadf}";
    pub const DOT: &str = "\u{eb8a}";
    pub const EDIT_CODE: &str = "\u{ec68}";
    pub const EXTERNAL_LINK: &str = "\u{eb14}";
    pub const EYE: &str = "\u{ea70}";
    pub const EYE_OFF: &str = "\u{eae7}";
    pub const FILE_CODE: &str = "\u{eae9}";
    pub const FILTER: &str = "\u{eaf1}";
    pub const FOLDER: &str = "\u{ea83}";
    pub const FOLDER_MINUS: &str = "\u{e0d8}";
    pub const GEAR: &str = "\u{eaf8}";
    pub const GIT_BRANCH: &str = "\u{ea68}";
    pub const GIT_MERGE: &str = "\u{eafe}";
    pub const INFO: &str = "\u{ea74}";
    pub const LIGATURE: &str = "\u{eb7e}";
    pub const LIST: &str = "\u{eb17}";
    pub const LIST_CHECKS: &str = "\u{eab3}";
    pub const LIST_TREE: &str = "\u{eb86}";
    pub const MOON: &str = "\u{e11e}";
    pub const PACKAGE: &str = "\u{eb29}";
    pub const PALETTE: &str = "\u{eb5c}";
    pub const PANEL_LEFT: &str = "\u{ebf3}";
    pub const PENCIL: &str = "\u{ea73}";
    pub const PIN: &str = "\u{eb2b}";
    pub const PIN_OFF: &str = "\u{ec9e}";
    pub const PLUS: &str = "\u{ea60}";
    pub const ROWS: &str = "\u{eb57}";
    pub const SEARCH: &str = "\u{ea6d}";
    pub const SHIELD: &str = "\u{eb53}";
    pub const SLIDERS_HORIZONTAL: &str = "\u{eb52}";
    pub const SPARKLE: &str = "\u{ec10}";
    pub const SUBMODULE: &str = "\u{eaec}";
    pub const SYNC: &str = "\u{ea77}";
    pub const SUN: &str = "\u{e178}";
    pub const TAG: &str = "\u{ea66}";
    pub const TERMINAL: &str = "\u{ea85}";
    pub const TRASH: &str = "\u{ea81}";
    pub const UNFOLD_VERTICAL: &str = "\u{eb73}";
    pub const WARNING: &str = "\u{ea6c}";
    pub const WHITESPACE: &str = "\u{eb7d}";
    pub const WRENCH: &str = "\u{eb6d}";
    pub const X: &str = "\u{ea76}";
    pub const X_CIRCLE: &str = "\u{ea87}";
    pub const ZOOM_IN: &str = "\u{eb81}";
    pub const ZOOM_OUT: &str = "\u{eb82}";
}

/// Render an icon glyph at the given size, using a passed text color.
pub(crate) fn icon(glyph_str: &'static str, size: f32, color: u32) -> Div {
    div()
        .flex_none()
        .font_family(FONT)
        .text_size(px(size))
        .text_color(rgb(color))
        .child(SharedString::from(glyph_str))
}
