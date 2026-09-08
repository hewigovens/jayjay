use std::path::Path;

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, Window,
    rgb,
};
use jayjay_core::{DiffHunk, DiffPreview, HunkType};

use super::image_diff_split::ImageDiffSplit;
use crate::app::theme::Theme;
use crate::diff::media_diff::{format_size, media_diff_layout, media_frame, media_pane};
use jayjay_core::diff::DiffSide;

pub fn hunk_is_image(hunk: &DiffHunk) -> bool {
    matches!(hunk.old.preview, Some(DiffPreview::Image { .. }))
        || matches!(hunk.new.preview, Some(DiffPreview::Image { .. }))
}

pub fn image_diff_view(
    hunk: &DiffHunk,
    t: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let old_path = image_path(hunk.old.preview.as_ref());
    let new_path = image_path(hunk.new.preview.as_ref());
    if hunk.hunk_type == HunkType::Modified {
        return ImageDiffSplit::render(
            &hunk.path,
            pane(
                old_path,
                "Before",
                t.tag_removed_bg,
                t.tag_removed_fg,
                true,
                t,
            ),
            pane(new_path, "After", t.tag_added_bg, t.tag_added_fg, true, t),
            t,
            window,
            cx,
        );
    }
    media_diff_layout(
        hunk.hunk_type,
        t,
        |side, label, label_bg, label_fg, show_label, t| {
            let path = match side {
                DiffSide::Old => old_path.clone(),
                DiffSide::New => new_path.clone(),
            };
            pane(path, label, label_bg, label_fg, show_label, t)
        },
    )
}

fn pane(
    path: Option<String>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    show_label: bool,
    t: &Theme,
) -> AnyElement {
    let meta = metadata_line(path.as_deref(), t);
    let viewer = image_viewer(path, t);
    media_pane(label, label_bg, label_fg, show_label, viewer, Some(meta))
}

fn image_viewer(path: Option<String>, t: &Theme) -> AnyElement {
    let frame = media_frame(t).debug_selector(|| "image-preview-pane".to_owned());

    match path {
        Some(p) if Path::new(&p).exists() => frame
            .child(
                gpui::img(std::path::PathBuf::from(p))
                    .max_w_full()
                    .max_h_full(),
            )
            .into_any_element(),
        Some(_) => frame
            .text_color(rgb(t.fg_dim))
            .child(SharedString::from("(file unavailable)"))
            .into_any_element(),
        None => frame
            .text_color(rgb(t.fg_dim))
            .child(SharedString::from("—"))
            .into_any_element(),
    }
}

fn metadata_line(path: Option<&str>, t: &Theme) -> AnyElement {
    let label = match path {
        Some(p) => match std::fs::metadata(p) {
            Ok(meta) => format_size(meta.len()),
            Err(_) => String::from(" "),
        },
        None => String::from(" "),
    };
    crate::diff::media_diff::metadata_line(SharedString::from(label), t)
}

fn image_path(preview: Option<&DiffPreview>) -> Option<String> {
    match preview? {
        DiffPreview::Image { path } => Some(path.clone()),
    }
}
