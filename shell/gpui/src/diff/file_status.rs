use gpui::{Div, FontWeight, ParentElement, Styled, div, px, rgb, rgba};
use jayjay_core::diff::placeholders::{is_git_lfs, is_git_submodule};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::ui::icons::{self, glyph};

pub(crate) fn color(hunk: &DiffHunk, theme: &Theme) -> u32 {
    if is_submodule(hunk) {
        return theme.file_renamed_color;
    }
    if is_lfs(hunk) {
        return theme.file_lfs_color;
    }
    color_for_hunk_type(hunk.hunk_type, theme)
}

pub(crate) fn color_for_hunk_type(hunk_type: HunkType, theme: &Theme) -> u32 {
    match hunk_type {
        HunkType::Added => theme.file_added_color,
        HunkType::Removed => theme.file_removed_color,
        HunkType::Modified => theme.file_modified_color,
        HunkType::Renamed => theme.file_renamed_color,
    }
}

pub(crate) fn label(hunk_type: HunkType) -> &'static str {
    match hunk_type {
        HunkType::Added => "Added",
        HunkType::Removed => "Removed",
        HunkType::Modified => "Modified",
        HunkType::Renamed => "Renamed",
    }
}

pub(crate) fn badge(hunk_type: HunkType, theme: &Theme) -> Div {
    let symbol = match hunk_type {
        HunkType::Added => "+",
        HunkType::Removed => "\u{2212}",
        HunkType::Modified => "~",
        HunkType::Renamed => "\u{2192}",
    };
    let fg = match hunk_type {
        HunkType::Added => theme.file_badge_added_fg,
        HunkType::Removed => theme.file_badge_removed_fg,
        HunkType::Modified | HunkType::Renamed => theme.file_badge_modified_fg,
    };
    let bg_alpha = if theme.is_dark { 0x24 } else { 0x26 };
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(theme.scaled_font_size(18.)))
        .rounded(px(4.))
        .bg(rgba(with_alpha(
            color_for_hunk_type(hunk_type, theme),
            bg_alpha,
        )))
        .text_size(ui_font_size(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(fg))
        .child(symbol)
}

/// Matches SwiftUI's `*.circle.fill` file-status symbols.
pub(crate) fn disc(hunk_type: HunkType, color: u32, size: f32) -> Div {
    let disc = div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded_full()
        .bg(rgb(color));
    let glyph_str = match hunk_type {
        HunkType::Added => Some(glyph::PLUS),
        HunkType::Removed => Some(glyph::MINUS),
        HunkType::Modified => None,
        HunkType::Renamed => Some(glyph::ARROW_RIGHT),
    };
    match glyph_str {
        Some(glyph_str) => disc.child(icons::icon(glyph_str, size * 0.6, 0xffffff)),
        None => disc.child(
            div()
                .text_size(px(size * 0.7))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child("~"),
        ),
    }
}

pub(crate) fn is_submodule(hunk: &DiffHunk) -> bool {
    is_git_submodule(hunk.old.content.as_deref()) || is_git_submodule(hunk.new.content.as_deref())
}

pub(crate) fn is_lfs(hunk: &DiffHunk) -> bool {
    is_git_lfs(hunk.old.content.as_deref()) || is_git_lfs(hunk.new.content.as_deref())
}
