//! Author avatar fetcher / cache (GPUI shell only).
//!
//! Strategy mirrors `shell/mac/Sources/JayJay/Shared/GitHubAvatar.swift`:
//!   1. If email is `<id>+<user>@users.noreply.github.com`, try
//!      `https://github.com/<user>.png?size=N` (preserves account avatar).
//!   2. Fallback to Gravatar: `https://gravatar.com/avatar/<md5>?s=N&d=retro`.
//!
//! Disk cache: `$HOME/.cache/jayjay/avatars/<email-hash>.png`. Once on disk,
//! the GPUI `img(path)` element renders it directly from the file without any
//! network call.

use std::path::PathBuf;

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, img, px, rgb};
use md5::{Digest, Md5};

const PIXEL_SIZE: u32 = 96; // 2x for ~24pt slot
const AVATAR_BYTE_CAP: u64 = 2 * 1024 * 1024; // hard cap 2MB

pub fn email_md5(email: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(email.trim().to_lowercase().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn cache_path(email: &str) -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    let cache_root = dirs
        .home_dir()
        .join(".cache")
        .join("jayjay")
        .join("avatars");
    Some(cache_root.join(format!("{}.png", email_md5(email))))
}

fn url_for_email(email: &str) -> Option<String> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(local) = trimmed.strip_suffix("@users.noreply.github.com") {
        let user = local.rsplit('+').next().unwrap_or(local);
        if !user.is_empty() {
            return Some(format!("https://github.com/{user}.png?size={PIXEL_SIZE}"));
        }
    }
    Some(format!(
        "https://gravatar.com/avatar/{}?s={PIXEL_SIZE}&d=retro",
        email_md5(email)
    ))
}

/// Blocking fetch — meant to run on a background executor.
/// Writes the PNG bytes to the cache path on success.
pub fn fetch_blocking(email: &str) -> bool {
    let Some(path) = cache_path(email) else {
        return false;
    };
    if path.exists() {
        return true;
    }
    let Some(url) = url_for_email(email) else {
        return false;
    };

    let Ok(bytes) =
        jayjay_network::get_bytes(&url, AVATAR_BYTE_CAP, &jayjay_network::Auth::default())
    else {
        return false;
    };
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    std::fs::write(&path, &bytes).is_ok()
}

/// Stable index into `INITIAL_PALETTE` derived from the email — same email
/// always gets the same fallback color.
pub fn initial_color(email: &str) -> u32 {
    const PALETTE: &[u32] = &[
        0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2,
    ];
    let h = email_md5(email);
    let byte = u8::from_str_radix(&h[..2], 16).unwrap_or(0) as usize;
    PALETTE[byte % PALETTE.len()]
}

pub fn initial(name: &str) -> char {
    name.chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().next().unwrap_or('?'))
        .unwrap_or('?')
}

/// A circular author avatar: the cached image if present, else a colored
/// monogram. Cached on disk by the loaders, so this never hits the network.
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
