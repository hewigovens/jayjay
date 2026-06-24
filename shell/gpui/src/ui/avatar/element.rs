//! The circular avatar element and its fallback monogram.

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, img, px, rgb};

use super::cache::{cache_path, email_md5};

/// Stable palette index from the email — same email, same fallback color.
fn initial_color(email: &str) -> u32 {
    const PALETTE: &[u32] = &[
        0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2,
    ];
    let h = email_md5(email);
    let byte = u8::from_str_radix(&h[..2], 16).unwrap_or(0) as usize;
    PALETTE[byte % PALETTE.len()]
}

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
        .bg(rgb(initial_color(email)))
        .text_size(px(size * 0.55))
        .text_color(rgb(0xffffff))
        .child(SharedString::from(initial(name).to_string()))
        .into_any_element()
}
