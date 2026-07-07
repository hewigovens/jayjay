use std::path::Path;

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::{DiffHunk, DiffPreview, HunkType};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::primitives::capsule;

pub fn hunk_is_image(hunk: &DiffHunk) -> bool {
    matches!(hunk.old.preview, Some(DiffPreview::Image { .. }))
        || matches!(hunk.new.preview, Some(DiffPreview::Image { .. }))
}

pub fn image_diff_view(hunk: &DiffHunk, t: &Theme) -> AnyElement {
    let old_path = image_path(hunk.old.preview.as_ref());
    let new_path = image_path(hunk.new.preview.as_ref());

    match hunk.hunk_type {
        HunkType::Added => single_pane_layout(new_path, "Added", t.tag_added_bg, t.tag_added_fg, t),
        HunkType::Removed => {
            single_pane_layout(old_path, "Removed", t.tag_removed_bg, t.tag_removed_fg, t)
        }
        HunkType::Renamed => {
            single_pane_layout(new_path, "Renamed", t.tag_renamed_bg, t.tag_renamed_fg, t)
        }
        HunkType::Modified => div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .gap(px(12.))
            .px(px(16.))
            .py(px(16.))
            .bg(rgb(t.detail_bg))
            .child(pane(
                old_path,
                "Before",
                t.tag_removed_bg,
                t.tag_removed_fg,
                t,
            ))
            .child(pane(new_path, "After", t.tag_added_bg, t.tag_added_fg, t))
            .into_any_element(),
    }
}

fn single_pane_layout(
    path: Option<String>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    t: &Theme,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_h_0()
        .px(px(16.))
        .py(px(16.))
        .bg(rgb(t.detail_bg))
        .child(pane(path, label, label_bg, label_fg, t))
        .into_any_element()
}

fn pane(
    path: Option<String>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    t: &Theme,
) -> AnyElement {
    let meta = metadata_line(path.as_deref(), t);
    let viewer = image_viewer(path, t);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .items_center()
        .gap(px(8.))
        .child(capsule(label, label_bg, label_fg, 11.))
        .child(viewer)
        .child(meta)
        .into_any_element()
}

fn image_viewer(path: Option<String>, t: &Theme) -> AnyElement {
    let frame = div()
        .flex()
        .flex_1()
        .w_full()
        .min_h_0()
        .items_center()
        .justify_center()
        .rounded_md()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(if t.is_dark { 0x14171c } else { 0xeef0f3 }));

    match path {
        Some(p) if Path::new(&p).exists() => frame
            .child(
                gpui::img(std::path::PathBuf::from(p))
                    .max_w_full()
                    .max_h_full(),
            )
            .into_any_element(),
        Some(_) | None => frame
            .text_color(rgb(t.fg_dim))
            .child(SharedString::from(if path_was_some(&path) {
                "(file unavailable)"
            } else {
                "—"
            }))
            .into_any_element(),
    }
}

fn path_was_some(p: &Option<String>) -> bool {
    p.is_some()
}

fn metadata_line(path: Option<&str>, t: &Theme) -> AnyElement {
    let label = match path {
        Some(p) => match std::fs::metadata(p) {
            Ok(meta) => format_size(meta.len()),
            Err(_) => String::from(" "),
        },
        None => String::from(" "),
    };
    div()
        .font_family(fonts::mono())
        .text_size(px(10.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(label))
        .into_any_element()
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let b = bytes as f64;
    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{:.1} MB", b / MB)
    }
}

fn image_path(preview: Option<&DiffPreview>) -> Option<String> {
    match preview? {
        DiffPreview::Image { path } => Some(path.clone()),
    }
}
