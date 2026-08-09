use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use gpui::{AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, svg};
use jayjay_core::HunkType;

use crate::app::theme::Theme;
use crate::diff::media_diff::{format_size, media_diff_layout, media_frame, media_pane};
use jayjay_core::diff::DiffSide;

#[derive(Clone, Copy)]
pub(crate) struct SvgDiffContent<'a> {
    pub(crate) old: Option<&'a str>,
    pub(crate) new: Option<&'a str>,
}

pub(crate) fn svg_diff_view(
    content: SvgDiffContent<'_>,
    hunk_type: HunkType,
    t: &Theme,
) -> AnyElement {
    media_diff_layout(
        hunk_type,
        t,
        |side, label, label_bg, label_fg, show_label, t| {
            let content = match side {
                DiffSide::Old => content.old,
                DiffSide::New => content.new,
            };
            pane(content, label, label_bg, label_fg, show_label, t)
        },
    )
}

fn pane(
    content: Option<&str>,
    label: &'static str,
    label_bg: u32,
    label_fg: u32,
    show_label: bool,
    t: &Theme,
) -> AnyElement {
    let svg_path = content.and_then(cached_svg_file);
    let meta = metadata_line(content, t);
    let viewer = svg_viewer(svg_path, content.is_some(), t);
    media_pane(label, label_bg, label_fg, show_label, viewer, Some(meta))
}

fn svg_viewer(path: Option<PathBuf>, had_content: bool, t: &Theme) -> AnyElement {
    let frame = media_frame(t).debug_selector(|| "svg-preview-pane".to_owned());

    match path {
        Some(path) => frame
            .child(
                svg()
                    .external_path(path.to_string_lossy().into_owned())
                    .size_full()
                    .text_color(gpui::rgb(t.fg)),
            )
            .into_any_element(),
        None => {
            let label = if had_content {
                "(preview unavailable)"
            } else {
                "—"
            };
            frame
                .text_color(gpui::rgb(t.fg_dim))
                .child(SharedString::from(label))
                .into_any_element()
        }
    }
}

fn cached_svg_file(content: &str) -> Option<PathBuf> {
    if content.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    let cache_dir = std::env::temp_dir().join("jayjay-svg-previews");
    std::fs::create_dir_all(&cache_dir).ok()?;
    let path = cache_dir.join(format!("{:016x}.svg", hasher.finish()));
    if !path.exists() {
        std::fs::write(&path, content).ok()?;
    }
    Some(path)
}

fn metadata_line(content: Option<&str>, t: &Theme) -> AnyElement {
    let label = content
        .map(|content| format_size(content.len() as u64))
        .unwrap_or_else(|| " ".to_owned());
    crate::diff::media_diff::metadata_line(SharedString::from(label), t)
}
