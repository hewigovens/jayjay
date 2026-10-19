use gpui::{Div, ParentElement, Styled, Svg, div, px, rgb, rgba, svg};
use jayjay_core::diff::placeholders::{is_git_lfs, is_git_submodule};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::theme::{Theme, with_alpha};
use crate::ui::icons;

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
        HunkType::Copied => theme.file_added_color,
    }
}

pub(crate) fn label(hunk_type: HunkType) -> &'static str {
    match hunk_type {
        HunkType::Added => "Added",
        HunkType::Removed => "Removed",
        HunkType::Modified => "Modified",
        HunkType::Renamed => "Renamed",
        HunkType::Copied => "Copied",
    }
}

/// Matches SwiftUI's file-status SF Symbols.
fn status_icon(hunk_type: HunkType, size: f32, color: u32) -> Svg {
    let path = match hunk_type {
        HunkType::Added => icons::PLUS_SVG,
        HunkType::Removed => icons::MINUS_SVG,
        HunkType::Modified => icons::PLUS_MINUS_SVG,
        HunkType::Renamed => icons::ARROW_RIGHT_SVG,
        HunkType::Copied => icons::COPY_SVG,
    };
    svg()
        .path(path)
        .flex_none()
        .size(px(size))
        .text_color(rgb(color))
}

pub(crate) fn badge(hunk_type: HunkType, theme: &Theme) -> Div {
    let fg = match hunk_type {
        HunkType::Added | HunkType::Copied => theme.file_badge_added_fg,
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
        .child(status_icon(hunk_type, theme.scaled_font_size(12.), fg))
}

pub(crate) fn disc(hunk_type: HunkType, color: u32, size: f32) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded_full()
        .bg(rgb(color))
        .child(status_icon(hunk_type, size * 0.6, 0xffffff))
}

pub(crate) fn is_submodule(hunk: &DiffHunk) -> bool {
    is_git_submodule(hunk.old.content.as_deref()) || is_git_submodule(hunk.new.content.as_deref())
}

pub(crate) fn is_lfs(hunk: &DiffHunk) -> bool {
    is_git_lfs(hunk.old.content.as_deref()) || is_git_lfs(hunk.new.content.as_deref())
}
