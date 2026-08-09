use std::path::Path;

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, rgb};
use jayjay_core::{DiffHunk, DiffPreview};

use crate::app::theme::Theme;
use crate::diff::media_diff::{format_size, media_diff_layout, media_frame, media_pane};
use jayjay_core::diff::DiffSide;

pub fn hunk_is_image(hunk: &DiffHunk) -> bool {
    matches!(hunk.old.preview, Some(DiffPreview::Image { .. }))
        || matches!(hunk.new.preview, Some(DiffPreview::Image { .. }))
}

pub fn image_diff_view(hunk: &DiffHunk, t: &Theme) -> AnyElement {
    let old_path = image_path(hunk.old.preview.as_ref());
    let new_path = image_path(hunk.new.preview.as_ref());
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
    let frame = media_frame(t);

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
