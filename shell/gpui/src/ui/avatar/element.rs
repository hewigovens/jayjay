//! The circular avatar element and its fallback monogram.

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, img, px, rgb};

use jayjay_core::avatar;

use super::cache::cache_path;

fn initial(name: &str) -> char {
    name.chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().next().unwrap_or('?'))
        .unwrap_or('?')
}

/// Circular avatar: the cached image if present, else a colored monogram (never hits the network).
pub fn element(email: &str, name: &str, size: f32) -> AnyElement {
    if let Some(path) = cache_path(email)
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
        .bg(rgb(avatar::monogram_color(email)))
        .text_size(px(size * 0.55))
        .text_color(rgb(0xffffff))
        .child(SharedString::from(initial(name).to_string()))
        .into_any_element()
}
