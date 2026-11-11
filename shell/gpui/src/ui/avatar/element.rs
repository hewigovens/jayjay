//! The circular avatar element and its fallback monogram.

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, img, px, rgb};

use jayjay_core::{CommitAuthor, avatar};

use super::cache::cache_path;

/// Circular avatar: the cached image if present, else a colored monogram (never hits the network).
pub fn element(author: &CommitAuthor, size: f32) -> AnyElement {
    if let Some(path) = cache_path(&author.email)
        && path.exists()
    {
        return img(path)
            .w(px(size))
            .h(px(size))
            .rounded_full()
            .into_any_element();
    }
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(size))
        .h(px(size))
        .rounded_full()
        .bg(rgb(avatar::monogram_color(&author.email)))
        .text_size(px(size * 0.55))
        .text_color(rgb(0xffffff))
        .child(SharedString::from(author.monogram_initial().to_string()))
        .into_any_element()
}
